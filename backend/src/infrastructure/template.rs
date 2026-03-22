/// Minimal `{{ expr }}` template evaluator.
///
/// Supported syntax inside braces:
///   state("device_name")
///   brightness("device_name")
///   now_hour()
///   integer / float literals
///   arithmetic: + - * /
///   comparisons: == != > < >= <=
use crate::domain::manager::SmartHome;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateValue {
    Str(String),
    Num(f64),
    Bool(bool),
}

impl TemplateValue {
    /// Truthy: true, non-zero number, non-empty string.
    pub fn is_truthy(&self) -> bool {
        match self {
            TemplateValue::Bool(b) => *b,
            TemplateValue::Num(n) => *n != 0.0,
            TemplateValue::Str(s) => !s.is_empty(),
        }
    }
}

#[derive(Debug)]
pub struct TemplateError(pub String);

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "template error: {}", self.0)
    }
}

/// Context passed to template evaluation — snapshot of SmartHome + current hour.
pub struct TemplateContext<'a> {
    pub home: &'a SmartHome,
    pub now_hour: u32,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Evaluate a `{{ expr }}` template string.
/// If the string has no `{{` tokens it is returned verbatim as a `Str`.
/// If it has exactly one `{{ expr }}` token the result is that expression's value.
/// Multiple tokens are concatenated as strings.
pub fn eval_template(tmpl: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    // Fast path: no template tokens.
    if !tmpl.contains("{{") {
        return Ok(TemplateValue::Str(tmpl.to_string()));
    }

    let mut parts: Vec<TemplateValue> = Vec::new();
    let mut remaining = tmpl;

    while let Some(start) = remaining.find("{{") {
        let literal = &remaining[..start];
        if !literal.is_empty() {
            parts.push(TemplateValue::Str(literal.to_string()));
        }
        let after_open = &remaining[start + 2..];
        let end = after_open.find("}}").ok_or_else(|| TemplateError("unclosed '{{' in template".to_string()))?;
        let expr = after_open[..end].trim();
        parts.push(eval_expr(expr, ctx)?);
        remaining = &after_open[end + 2..];
    }
    if !remaining.is_empty() {
        parts.push(TemplateValue::Str(remaining.to_string()));
    }

    if parts.len() == 1 {
        Ok(parts.remove(0))
    } else {
        // Multiple parts — join as string.
        let joined = parts.iter().map(|p| match p {
            TemplateValue::Str(s) => s.clone(),
            TemplateValue::Num(n) => n.to_string(),
            TemplateValue::Bool(b) => b.to_string(),
        }).collect::<String>();
        Ok(TemplateValue::Str(joined))
    }
}

// ── Expression parser / evaluator ────────────────────────────────────────────

fn eval_expr(expr: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    let expr = expr.trim();
    eval_comparison(expr, ctx)
}

fn eval_comparison(expr: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    // Try two-char operators first to avoid matching '<' before '<='
    for op in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some(pos) = find_op(expr, op) {
            let lhs = eval_additive(expr[..pos].trim(), ctx)?;
            let rhs = eval_additive(expr[pos + op.len()..].trim(), ctx)?;
            let result = compare(&lhs, op, &rhs)?;
            return Ok(TemplateValue::Bool(result));
        }
    }
    eval_additive(expr, ctx)
}

fn eval_additive(expr: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    // Split on + or - (right-to-left to handle left-associativity with simple split)
    // We walk from the right looking for + or - outside function calls.
    let bytes = expr.as_bytes();
    let mut depth = 0i32;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'+' | b'-' if depth == 0 && i > 0 => {
                let op = if bytes[i] == b'+' { "+" } else { "-" };
                let lhs = eval_additive(expr[..i].trim(), ctx)?;
                let rhs = eval_multiplicative(expr[i + 1..].trim(), ctx)?;
                return numeric_op(&lhs, op, &rhs);
            }
            _ => {}
        }
    }
    eval_multiplicative(expr, ctx)
}

fn eval_multiplicative(expr: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    let bytes = expr.as_bytes();
    let mut depth = 0i32;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'*' | b'/' if depth == 0 => {
                let op = if bytes[i] == b'*' { "*" } else { "/" };
                let lhs = eval_multiplicative(expr[..i].trim(), ctx)?;
                let rhs = eval_atom(expr[i + 1..].trim(), ctx)?;
                return numeric_op(&lhs, op, &rhs);
            }
            _ => {}
        }
    }
    eval_atom(expr, ctx)
}

fn eval_atom(expr: &str, ctx: &TemplateContext<'_>) -> Result<TemplateValue, TemplateError> {
    let expr = expr.trim();

    // Parenthesised expression
    if expr.starts_with('(') && expr.ends_with(')') {
        return eval_expr(&expr[1..expr.len() - 1], ctx);
    }

    // Function calls
    if let Some(rest) = expr.strip_prefix("state(")
        && let Some(name) = extract_string_arg(rest) {
            let device = ctx.home.get_device(&name)
                .ok_or_else(|| TemplateError(format!("unknown device '{}'", name)))?;
            let state_str = match &device.state {
                crate::domain::device::DeviceState::On  => "on",
                crate::domain::device::DeviceState::Off => "off",
                crate::domain::device::DeviceState::Unknown => "unknown",
            };
            return Ok(TemplateValue::Str(state_str.to_string()));
        }
    if let Some(rest) = expr.strip_prefix("brightness(")
        && let Some(name) = extract_string_arg(rest) {
            let device = ctx.home.get_device(&name)
                .ok_or_else(|| TemplateError(format!("unknown device '{}'", name)))?;
            return Ok(TemplateValue::Num(device.brightness as f64));
        }
    if expr == "now_hour()" {
        return Ok(TemplateValue::Num(ctx.now_hour as f64));
    }

    // Boolean literals
    if expr == "true"  { return Ok(TemplateValue::Bool(true)); }
    if expr == "false" { return Ok(TemplateValue::Bool(false)); }

    // Numeric literal
    if let Ok(n) = expr.parse::<f64>() {
        return Ok(TemplateValue::Num(n));
    }

    // Quoted string literal
    if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
        return Ok(TemplateValue::Str(expr[1..expr.len() - 1].to_string()));
    }

    Err(TemplateError(format!("cannot evaluate expression: '{}'", expr)))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Find the position of a binary operator `op` in `expr`, skipping parenthesised sub-expressions.
fn find_op(expr: &str, op: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let op_bytes = op.as_bytes();
    let mut depth = 0i32;
    let len = bytes.len();
    let op_len = op_bytes.len();
    let mut i = 0usize;
    while i + op_len <= len {
        match bytes[i] {
            b'(' => { depth += 1; i += 1; }
            b')' => { depth -= 1; i += 1; }
            _ if depth == 0 && bytes[i..].starts_with(op_bytes) => return Some(i),
            _ => { i += 1; }
        }
    }
    None
}

/// Extract the string argument from `"name")` — the part after the function prefix.
fn extract_string_arg(rest: &str) -> Option<String> {
    // rest is like `"living_room_light")`
    let inner = rest.strip_suffix(')')?.trim();
    if inner.starts_with('"') && inner.ends_with('"') && inner.len() >= 2 {
        Some(inner[1..inner.len() - 1].to_string())
    } else {
        None
    }
}

fn numeric_op(lhs: &TemplateValue, op: &str, rhs: &TemplateValue) -> Result<TemplateValue, TemplateError> {
    let l = to_num(lhs)?;
    let r = to_num(rhs)?;
    let result = match op {
        "+" => l + r,
        "-" => l - r,
        "*" => l * r,
        "/" => {
            if r == 0.0 { return Err(TemplateError("division by zero".to_string())); }
            l / r
        }
        _ => return Err(TemplateError(format!("unknown operator '{}'", op))),
    };
    Ok(TemplateValue::Num(result))
}

fn compare(lhs: &TemplateValue, op: &str, rhs: &TemplateValue) -> Result<bool, TemplateError> {
    match (lhs, rhs) {
        (TemplateValue::Num(l), TemplateValue::Num(r)) => Ok(match op {
            "==" => l == r, "!=" => l != r,
            ">"  => l > r,  "<"  => l < r,
            ">=" => l >= r, "<=" => l <= r,
            _ => return Err(TemplateError(format!("unknown op '{}'", op))),
        }),
        (TemplateValue::Str(l), TemplateValue::Str(r)) => Ok(match op {
            "==" => l == r, "!=" => l != r,
            _ => return Err(TemplateError(format!("operator '{}' not supported for strings", op))),
        }),
        (TemplateValue::Bool(l), TemplateValue::Bool(r)) => Ok(match op {
            "==" => l == r, "!=" => l != r,
            _ => return Err(TemplateError(format!("operator '{}' not supported for booleans", op))),
        }),
        _ => Err(TemplateError("type mismatch in comparison".to_string())),
    }
}

fn to_num(v: &TemplateValue) -> Result<f64, TemplateError> {
    match v {
        TemplateValue::Num(n) => Ok(*n),
        _ => Err(TemplateError("expected numeric value".to_string())),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::device::DeviceType;

    fn ctx<'a>(home: &'a SmartHome, hour: u32) -> TemplateContext<'a> {
        TemplateContext { home, now_hour: hour }
    }

    #[test]
    fn state_lookup() {
        let mut home = SmartHome::new();
        home.add_device("living_room_light", DeviceType::Light).unwrap();
        home.set_state("living_room_light", crate::domain::device::DeviceState::On).unwrap();
        let c = ctx(&home, 12);
        let v = eval_template(r#"{{ state("living_room_light") }}"#, &c).unwrap();
        assert_eq!(v, TemplateValue::Str("on".to_string()));
    }

    #[test]
    fn brightness_arithmetic() {
        let mut home = SmartHome::new();
        home.add_device("desk_lamp", DeviceType::Light).unwrap();
        home.set_brightness("desk_lamp", 60).unwrap();
        let c = ctx(&home, 12);
        let v = eval_template(r#"{{ brightness("desk_lamp") + 20 }}"#, &c).unwrap();
        assert_eq!(v, TemplateValue::Num(80.0));
    }

    #[test]
    fn time_comparison() {
        let home = SmartHome::new();
        let c = ctx(&home, 23);
        let v = eval_template("{{ now_hour() >= 22 }}", &c).unwrap();
        assert_eq!(v, TemplateValue::Bool(true));
        let c2 = ctx(&home, 10);
        let v2 = eval_template("{{ now_hour() >= 22 }}", &c2).unwrap();
        assert_eq!(v2, TemplateValue::Bool(false));
    }

    #[test]
    fn unknown_device_returns_error() {
        let home = SmartHome::new();
        let c = ctx(&home, 12);
        let err = eval_template(r#"{{ state("ghost") }}"#, &c);
        assert!(err.is_err());
    }

    #[test]
    fn no_template_passthrough() {
        let home = SmartHome::new();
        let c = ctx(&home, 12);
        let v = eval_template("hello world", &c).unwrap();
        assert_eq!(v, TemplateValue::Str("hello world".to_string()));
    }
}
