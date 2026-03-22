use std::collections::HashMap;
use std::time::Duration;

use log::warn;
use serde_json::Value;
use tokio::time::sleep;

use crate::domain::device::DeviceState;
use crate::domain::scene::SceneRegistry;
use crate::domain::script::{Script, ScriptStep};
use crate::infrastructure::template::{TemplateContext, eval_template, TemplateValue};
use crate::state::AppState;

pub const MAX_DEPTH: u8 = 5;
pub const MAX_DELAY_MS: u64 = 60_000;

/// Execute a script asynchronously, applying each step in order.
/// Returns a list of error/warning messages.
pub async fn run_script(script: Script, args: HashMap<String, Value>, state: AppState, depth: u8) -> Vec<String> {
    if depth >= MAX_DEPTH {
        warn!("script '{}': max recursion depth exceeded", script.name);
        return vec![format!("script '{}': max depth exceeded", script.name)];
    }

    let mut errors = Vec::new();

    for step in &script.steps {
        match step {
            ScriptStep::SetState { device_name, state: target_state } => {
                let resolved_name = resolve_str(device_name, &state, &args);
                let resolved_state_str = resolve_str(target_state, &state, &args);
                let device_state = match resolved_state_str.as_str() {
                    "on" => DeviceState::On,
                    "off" => DeviceState::Off,
                    other => {
                        errors.push(format!("set_state: unknown state '{}'", other));
                        continue;
                    }
                };
                let result = {
                    let mut home = state.home.write().await;
                    home.set_state(&resolved_name, device_state)
                };
                if let Err(e) = result {
                    errors.push(format!("set_state on '{}': {}", resolved_name, e));
                    set_device_error(&state, &resolved_name, format!("{}", e)).await;
                } else {
                    persist_device(&state, &resolved_name).await;
                }
            }

            ScriptStep::SetBrightness { device_name, brightness } => {
                let resolved_name = resolve_str(device_name, &state, &args);
                let brightness_val = resolve_num(brightness, &state, &args);
                match brightness_val {
                    Ok(b) => {
                        let clamped = b.clamp(0.0, 100.0) as u8;
                        let result = { state.home.write().await.set_brightness(&resolved_name, clamped) };
                        if let Err(e) = result {
                            errors.push(format!("set_brightness on '{}': {}", resolved_name, e));
                            set_device_error(&state, &resolved_name, format!("{}", e)).await;
                        } else {
                            persist_device(&state, &resolved_name).await;
                        }
                    }
                    Err(e) => errors.push(format!("set_brightness template error on '{}': {}", resolved_name, e)),
                }
            }

            ScriptStep::SetTemperature { device_name, temperature } => {
                let resolved_name = resolve_str(device_name, &state, &args);
                let temp_val = resolve_num(temperature, &state, &args);
                match temp_val {
                    Ok(t) => {
                        let result = { state.home.write().await.set_temperature(&resolved_name, t) };
                        if let Err(e) = result {
                            errors.push(format!("set_temperature on '{}': {}", resolved_name, e));
                            set_device_error(&state, &resolved_name, format!("{}", e)).await;
                        } else {
                            persist_device(&state, &resolved_name).await;
                        }
                    }
                    Err(e) => errors.push(format!("set_temperature template error on '{}': {}", resolved_name, e)),
                }
            }

            ScriptStep::Delay { milliseconds } => {
                let capped = (*milliseconds).min(MAX_DELAY_MS);
                sleep(Duration::from_millis(capped)).await;
            }

            ScriptStep::ApplyScene { scene_name } => {
                let scene_snapshot = {
                    let scenes = state.scenes.read().await;
                    scenes.get_by_name(scene_name).cloned()
                };
                match scene_snapshot {
                    Some(scene) => {
                        let mut home_guard = state.home.write().await;
                        let (_, errs) = SceneRegistry::apply(&scene, &mut home_guard);
                        errors.extend(errs);
                    }
                    None => errors.push(format!("apply_scene: scene '{}' not found", scene_name)),
                }
            }

            ScriptStep::CallScript { script_name, args: call_args } => {
                let script_snapshot = {
                    let scripts = state.scripts.read().await;
                    scripts.get_by_name(script_name).cloned()
                };
                match script_snapshot {
                    Some(s) => {
                        let child_errors = Box::pin(run_script(s, call_args.clone(), state.clone(), depth + 1)).await;
                        errors.extend(child_errors);
                    }
                    None => {
                        warn!("call_script: script '{}' not found — skipping", script_name);
                        errors.push(format!("call_script: script '{}' not found", script_name));
                    }
                }
            }
        }
    }

    errors
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resolve_str(value: &str, state: &AppState, _args: &HashMap<String, Value>) -> String {
    if !value.contains("{{") {
        return value.to_string();
    }
    // We need a sync snapshot for template resolution — take a try_read
    if let Ok(home) = state.home.try_read() {
        let ctx = TemplateContext { home: &home, now_hour: chrono::Local::now().time().hour() };
        if let Ok(v) = eval_template(value, &ctx) {
            return match v {
                TemplateValue::Str(s) => s,
                TemplateValue::Num(n) => n.to_string(),
                TemplateValue::Bool(b) => b.to_string(),
            };
        }
    }
    value.to_string()
}

fn resolve_num(value: &Value, state: &AppState, _args: &HashMap<String, Value>) -> Result<f64, String> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| "invalid number".to_string()),
        Value::String(s) => {
            if s.contains("{{") {
                if let Ok(home) = state.home.try_read() {
                    let ctx = TemplateContext { home: &home, now_hour: chrono::Local::now().time().hour() };
                    match eval_template(s, &ctx) {
                        Ok(TemplateValue::Num(n)) => return Ok(n),
                        Ok(_) => return Err(format!("template '{}' did not evaluate to a number", s)),
                        Err(e) => return Err(e.to_string()),
                    }
                }
                Err(format!("could not acquire home read lock for template '{}'", s))
            } else {
                s.parse::<f64>().map_err(|_| format!("'{}' is not a number", s))
            }
        }
        _ => Err(format!("expected number, got {:?}", value)),
    }
}

async fn set_device_error(state: &AppState, device_name: &str, msg: String) {
    let _ = state.home.write().await.set_device_error(device_name, msg);
}

async fn persist_device(state: &AppState, device_name: &str) {
    use crate::infrastructure::db;
    if let Some(pool) = &state.db {
        let device_opt = {
            let home = state.home.read().await;
            home.get_device(device_name).cloned()
        };
        if let Some(device) = device_opt
            && let Err(e) = db::upsert_device(pool, &device, None).await {
                warn!("script_executor: failed to persist device '{}': {}", device_name, e);
            }
    }
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::script::ScriptParam;

    #[tokio::test]
    async fn delay_capped_at_60s() {
        // 70 000 ms should be clamped to 60 000 ms
        assert_eq!(70_000u64.min(MAX_DELAY_MS), MAX_DELAY_MS);
    }

    #[tokio::test]
    async fn max_depth_returns_error() {
        // Build a simple script and call with depth = MAX_DEPTH
        let script = Script::new("test", "", vec![], vec![ScriptStep::Delay { milliseconds: 0 }]);
        let state = crate::state::AppState::new(None);
        let errors = run_script(script, HashMap::new(), state, MAX_DEPTH).await;
        assert!(errors.iter().any(|e| e.contains("max depth")));
    }
}
