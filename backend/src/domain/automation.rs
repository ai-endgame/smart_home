use std::collections::HashMap;
use std::fmt;

use chrono::{NaiveTime, Timelike};
use serde_json::Value;

use crate::domain::device::DeviceState;
use crate::domain::error::DomainError;
use crate::domain::manager::SmartHome;
use crate::domain::presence::{PresenceRegistry, PresenceState};

/// Sun event kind for the Sun trigger.
#[derive(Debug, Clone, PartialEq)]
pub enum SunEvent { Sunrise, Sunset }

/// Numeric device attribute selector.
#[derive(Debug, Clone, PartialEq)]
pub enum NumericAttr { Brightness, Temperature }

/// What triggers an automation rule.
#[derive(Debug, Clone)]
pub enum Trigger {
    DeviceStateChange { device_name: String, target_state: DeviceState },
    TemperatureAbove { device_name: String, threshold: f64 },
    TemperatureBelow { device_name: String, threshold: f64 },
    Time { time: String },
    Sun { event: SunEvent, offset_minutes: i32 },
    NumericStateAbove { device_name: String, attribute: NumericAttr, threshold: f64 },
    NumericStateBelow { device_name: String, attribute: NumericAttr, threshold: f64 },
    Webhook { id: String },
    PresenceChange { person_name: String, target_state: PresenceState },
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trigger::DeviceStateChange { device_name, target_state } =>
                write!(f, "when '{}' turns {}", device_name, target_state),
            Trigger::TemperatureAbove { device_name, threshold } =>
                write!(f, "when '{}' temp > {:.1}°C", device_name, threshold),
            Trigger::TemperatureBelow { device_name, threshold } =>
                write!(f, "when '{}' temp < {:.1}°C", device_name, threshold),
            Trigger::Time { time } =>
                write!(f, "at {}", time),
            Trigger::Sun { event, offset_minutes } => {
                let evt = match event { SunEvent::Sunrise => "sunrise", SunEvent::Sunset => "sunset" };
                if *offset_minutes == 0 {
                    write!(f, "at {}", evt)
                } else {
                    write!(f, "{} {} min after {}", if *offset_minutes > 0 { "+" } else { "" }, offset_minutes, evt)
                }
            }
            Trigger::NumericStateAbove { device_name, attribute, threshold } => {
                let attr = match attribute { NumericAttr::Brightness => "brightness", NumericAttr::Temperature => "temp" };
                write!(f, "when '{}' {} > {:.1}", device_name, attr, threshold)
            }
            Trigger::NumericStateBelow { device_name, attribute, threshold } => {
                let attr = match attribute { NumericAttr::Brightness => "brightness", NumericAttr::Temperature => "temp" };
                write!(f, "when '{}' {} < {:.1}", device_name, attr, threshold)
            }
            Trigger::Webhook { id } =>
                write!(f, "webhook '{}'", id),
            Trigger::PresenceChange { person_name, target_state } =>
                write!(f, "when '{}' becomes {}", person_name, target_state),
        }
    }
}

/// Condition that must pass after a trigger fires.
#[derive(Debug, Clone)]
pub enum Condition {
    StateEquals { device_name: String, state: DeviceState },
    BrightnessAbove { device_name: String, value: f64 },
    BrightnessBelow { device_name: String, value: f64 },
    TemplateEval { expr: String },
}

/// What action to perform when a rule triggers.
#[derive(Debug, Clone)]
pub enum Action {
    DeviceState { device_name: String, state: DeviceState },
    Brightness { device_name: String, brightness: u8 },
    Temperature { device_name: String, temperature: f64 },
    Notify { message: String },
    ScriptCall { script_name: String, args: HashMap<String, Value> },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::DeviceState { device_name, state } =>
                write!(f, "set '{}' to {}", device_name, state),
            Action::Brightness { device_name, brightness } =>
                write!(f, "set '{}' brightness to {}%", device_name, brightness),
            Action::Temperature { device_name, temperature } =>
                write!(f, "set '{}' temp to {:.1}°C", device_name, temperature),
            Action::Notify { message } =>
                write!(f, "notify: {}", message),
            Action::ScriptCall { script_name, .. } =>
                write!(f, "call script '{}'", script_name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutomationRule {
    pub name: String,
    pub trigger: Trigger,
    pub action: Action,
    pub enabled: bool,
    /// When true, actuator actions (state/brightness/temperature) are suppressed.
    pub safe_mode: bool,
    /// Optional time window — if set, action only fires when current time is within range.
    pub time_range: Option<(String, String)>,
    /// All conditions must pass (AND semantics) after trigger fires.
    pub conditions: Vec<Condition>,
    /// Optional URL to POST a webhook notification when this rule fires.
    pub notify_url: Option<String>,
}

impl fmt::Display for AutomationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.enabled { "enabled" } else { "disabled" };
        write!(f, "[{}] {} → {} ({})", self.name, self.trigger, self.action, status)
    }
}

pub struct AutomationEngine {
    /// Rules keyed by lowercased name for O(1) lookups.
    pub rules: HashMap<String, AutomationRule>,
}

impl AutomationEngine {
    pub fn new() -> Self {
        AutomationEngine { rules: HashMap::new() }
    }

    pub fn add_rule(
        &mut self,
        name: &str,
        trigger: Trigger,
        action: Action,
        time_range: Option<(String, String)>,
        conditions: Vec<Condition>,
    ) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        if self.rules.contains_key(&key) {
            return Err(DomainError::AlreadyExists(format!("Rule '{}' already exists.", name)));
        }
        self.rules.insert(key, AutomationRule { name: name.to_string(), trigger, action, enabled: true, safe_mode: false, time_range, conditions, notify_url: None });
        Ok(())
    }

    pub fn remove_rule(&mut self, name: &str) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        self.rules.remove(&key)
            .map(|_| ())
            .ok_or_else(|| DomainError::NotFound(format!("Rule '{}' not found.", name)))
    }

    pub fn toggle_rule(&mut self, name: &str) -> Result<bool, DomainError> {
        let key = name.to_lowercase();
        let rule = self.rules.get_mut(&key)
            .ok_or_else(|| DomainError::NotFound(format!("Rule '{}' not found.", name)))?;
        rule.enabled = !rule.enabled;
        Ok(rule.enabled)
    }

    pub fn toggle_safe_mode(&mut self, name: &str) -> Result<bool, DomainError> {
        let key = name.to_lowercase();
        let rule = self.rules.get_mut(&key)
            .ok_or_else(|| DomainError::NotFound(format!("Rule '{}' not found.", name)))?;
        rule.safe_mode = !rule.safe_mode;
        Ok(rule.safe_mode)
    }

    fn rule_triggered(&self, rule: &AutomationRule, home: &SmartHome, presence: &PresenceRegistry, now: NaiveTime) -> bool {
        let utc_now = chrono::Utc::now();
        let triggered = match &rule.trigger {
            Trigger::DeviceStateChange { device_name, target_state } =>
                home.get_device(device_name).map(|d| &d.state == target_state).unwrap_or(false),
            Trigger::TemperatureAbove { device_name, threshold } =>
                home.get_device(device_name).and_then(|d| d.temperature).map(|t| t > *threshold).unwrap_or(false),
            Trigger::TemperatureBelow { device_name, threshold } =>
                home.get_device(device_name).and_then(|d| d.temperature).map(|t| t < *threshold).unwrap_or(false),
            Trigger::Time { time } =>
                parse_hhmm(time).map(|t| t.hour() == now.hour() && t.minute() == now.minute()).unwrap_or(false),
            Trigger::Sun { event, offset_minutes } => {
                let base = crate::infrastructure::sun::sun_event_time(event);
                let offset = chrono::Duration::minutes(*offset_minutes as i64);
                let target = base + offset;
                target.hour() == now.hour() && target.minute() == now.minute()
            }
            Trigger::NumericStateAbove { device_name, attribute, threshold } =>
                numeric_value(home, device_name, attribute).map(|v| v > *threshold).unwrap_or(false),
            Trigger::NumericStateBelow { device_name, attribute, threshold } =>
                numeric_value(home, device_name, attribute).map(|v| v < *threshold).unwrap_or(false),
            Trigger::Webhook { .. } => false,
            Trigger::PresenceChange { person_name, target_state } =>
                presence.get_by_name(person_name)
                    .map(|p| p.effective_state(utc_now) == *target_state)
                    .unwrap_or(false),
        };
        if !triggered { return false; }
        if let Some((from, to)) = &rule.time_range
            && !time_in_range(now, from, to) { return false; }
        for condition in &rule.conditions {
            if !evaluate_condition(condition, home, now) { return false; }
        }
        true
    }

    pub fn evaluate_rules(&self, home: &SmartHome, presence: &PresenceRegistry, now: NaiveTime) -> Vec<Action> {
        self.rules.values()
            .filter(|r| r.enabled)
            .filter_map(|rule| {
                if self.rule_triggered(rule, home, presence, now) { Some(rule.action.clone()) } else { None }
            })
            .collect()
    }

    /// Like `evaluate_rules` but also returns the rule name and notify_url for webhook dispatch.
    pub fn evaluate_rules_with_meta(&self, home: &SmartHome, presence: &PresenceRegistry, now: NaiveTime) -> Vec<(String, Option<String>, Action)> {
        self.rules.values()
            .filter(|r| r.enabled)
            .filter_map(|rule| {
                if self.rule_triggered(rule, home, presence, now) {
                    Some((rule.name.clone(), rule.notify_url.clone(), rule.action.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn execute_actions(actions: &[Action], home: &mut SmartHome, safe_mode: bool) -> Vec<String> {
        let mut notifications = Vec::new();
        for action in actions {
            match action {
                Action::DeviceState { device_name, state } => {
                    if safe_mode {
                        log::warn!("safe_mode: suppressing actuator action {:?}", action);
                    } else {
                        let _ = home.set_state(device_name, state.clone());
                    }
                }
                Action::Brightness { device_name, brightness } => {
                    if safe_mode {
                        log::warn!("safe_mode: suppressing actuator action {:?}", action);
                    } else {
                        let _ = home.set_brightness(device_name, *brightness);
                    }
                }
                Action::Temperature { device_name, temperature } => {
                    if safe_mode {
                        log::warn!("safe_mode: suppressing actuator action {:?}", action);
                    } else {
                        let _ = home.set_temperature(device_name, *temperature);
                    }
                }
                Action::Notify { message } => notifications.push(message.clone()),
                // ScriptCall is handled at the HTTP layer (spawns async executor); no-op here.
                Action::ScriptCall { .. } => {}
            }
        }
        notifications
    }

    pub fn list_rules(&self) -> Vec<&AutomationRule> {
        let mut rules: Vec<&AutomationRule> = self.rules.values().collect();
        rules.sort_by(|a, b| a.name.cmp(&b.name));
        rules
    }
}

impl Default for AutomationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn evaluate_condition(condition: &Condition, home: &SmartHome, now: NaiveTime) -> bool {
    match condition {
        Condition::StateEquals { device_name, state } =>
            home.get_device(device_name).map(|d| &d.state == state).unwrap_or(false),
        Condition::BrightnessAbove { device_name, value } =>
            home.get_device(device_name).map(|d| d.brightness as f64 > *value).unwrap_or(false),
        Condition::BrightnessBelow { device_name, value } =>
            home.get_device(device_name).map(|d| (d.brightness as f64) < *value).unwrap_or(false),
        Condition::TemplateEval { expr } => {
            let ctx = crate::infrastructure::template::TemplateContext {
                home,
                now_hour: now.hour(),
            };
            crate::infrastructure::template::eval_template(expr, &ctx)
                .map(|v| v.is_truthy())
                .unwrap_or(false)
        }
    }
}

fn parse_hhmm(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

fn numeric_value(home: &SmartHome, device_name: &str, attr: &NumericAttr) -> Option<f64> {
    let device = home.get_device(device_name)?;
    match attr {
        NumericAttr::Brightness => Some(device.brightness as f64),
        NumericAttr::Temperature => device.temperature,
    }
}

/// Returns true when `now` falls within [from, to). Supports overnight ranges (from > to).
pub fn time_in_range(now: NaiveTime, from: &str, to: &str) -> bool {
    let (Some(f), Some(t)) = (parse_hhmm(from), parse_hhmm(to)) else { return true };
    if f <= t {
        now >= f && now < t
    } else {
        // Overnight: e.g. 22:00–06:00
        now >= f || now < t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::device::DeviceType;

    #[test]
    fn test_add_and_toggle_rule() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "night",
            Trigger::DeviceStateChange { device_name: "sensor".into(), target_state: DeviceState::On },
            Action::DeviceState { device_name: "lamp".into(), state: DeviceState::On },
            None, vec![],
        ).unwrap();
        assert!(engine.add_rule("Night", Trigger::DeviceStateChange { device_name: "x".into(), target_state: DeviceState::On }, Action::DeviceState { device_name: "y".into(), state: DeviceState::Off }, None, vec![]).is_err());
        let enabled = engine.toggle_rule("night").unwrap();
        assert!(!enabled);
    }

    #[test]
    fn test_evaluate_and_execute() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let mut home = SmartHome::new();
        home.add_device("thermo", DeviceType::Thermostat).unwrap();
        home.set_temperature("thermo", 30.0).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "cool",
            Trigger::TemperatureAbove { device_name: "thermo".into(), threshold: 25.0 },
            Action::Temperature { device_name: "thermo".into(), temperature: 22.0 },
            None, vec![],
        ).unwrap();

        let actions = engine.evaluate_rules(&home, &PresenceRegistry::new(), now);
        assert_eq!(actions.len(), 1);
        AutomationEngine::execute_actions(&actions, &mut home, false);
        assert_eq!(home.get_device("thermo").unwrap().temperature, Some(22.0));
    }

    #[test]
    fn test_default_impl() {
        let engine = AutomationEngine::default();
        assert!(engine.rules.is_empty());
    }

    #[test]
    fn test_time_trigger_matches() {
        let now = NaiveTime::from_hms_opt(22, 0, 30).unwrap();
        let home = SmartHome::new();
        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "night_off",
            Trigger::Time { time: "22:00".into() },
            Action::Notify { message: "Night mode".into() },
            None, vec![],
        ).unwrap();
        let actions = engine.evaluate_rules(&home, &PresenceRegistry::new(), now);
        assert_eq!(actions.len(), 1);
        // Different minute — should not fire
        let other = NaiveTime::from_hms_opt(21, 59, 0).unwrap();
        assert!(engine.evaluate_rules(&home, &PresenceRegistry::new(), other).is_empty());
    }

    #[test]
    fn test_numeric_state_above() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let mut home = SmartHome::new();
        home.add_device("lamp", DeviceType::Light).unwrap();
        home.set_brightness("lamp", 90).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "bright_alert",
            Trigger::NumericStateAbove { device_name: "lamp".into(), attribute: NumericAttr::Brightness, threshold: 80.0 },
            Action::Notify { message: "Too bright".into() },
            None, vec![],
        ).unwrap();
        assert_eq!(engine.evaluate_rules(&home, &PresenceRegistry::new(), now).len(), 1);
        home.set_brightness("lamp", 70).unwrap();
        assert!(engine.evaluate_rules(&home, &PresenceRegistry::new(), now).is_empty());
    }

    #[test]
    fn test_time_in_range_normal_and_overnight() {
        let t1 = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        let t2 = NaiveTime::from_hms_opt(23, 30, 0).unwrap();
        assert!(time_in_range(t1, "08:00", "22:00"));
        assert!(!time_in_range(t2, "08:00", "22:00"));
        // Overnight
        assert!(time_in_range(t2, "22:00", "06:00"));
        assert!(!time_in_range(t1, "22:00", "06:00"));
    }

    #[test]
    fn test_notify_action_returns_message() {
        let mut home = SmartHome::new();
        let actions = vec![Action::Notify { message: "hello".into() }];
        let msgs = AutomationEngine::execute_actions(&actions, &mut home, false);
        assert_eq!(msgs, vec!["hello"]);
    }

    #[test]
    fn test_conditions_all_pass_action_fires() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let mut home = SmartHome::new();
        home.add_device("sensor", DeviceType::Light).unwrap();
        home.set_state("sensor", DeviceState::On).unwrap();
        home.add_device("lamp", DeviceType::Light).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "cond_test",
            Trigger::DeviceStateChange { device_name: "sensor".into(), target_state: DeviceState::On },
            Action::DeviceState { device_name: "lamp".into(), state: DeviceState::On },
            None,
            vec![Condition::StateEquals { device_name: "sensor".into(), state: DeviceState::On }],
        ).unwrap();
        assert_eq!(engine.evaluate_rules(&home, &PresenceRegistry::new(), now).len(), 1);
    }

    #[test]
    fn test_conditions_one_fails_action_skipped() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let mut home = SmartHome::new();
        home.add_device("sensor", DeviceType::Light).unwrap();
        home.set_state("sensor", DeviceState::On).unwrap();
        home.add_device("lamp", DeviceType::Light).unwrap();
        // lamp brightness defaults to 100 for lights; set to 0 so BrightnessAbove 50 fails
        home.set_brightness("lamp", 0).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "cond_fail",
            Trigger::DeviceStateChange { device_name: "sensor".into(), target_state: DeviceState::On },
            Action::DeviceState { device_name: "lamp".into(), state: DeviceState::On },
            None,
            vec![Condition::BrightnessAbove { device_name: "lamp".into(), value: 50.0 }],
        ).unwrap();
        assert!(engine.evaluate_rules(&home, &PresenceRegistry::new(), now).is_empty());
    }

    #[test]
    fn test_empty_conditions_always_fires() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let mut home = SmartHome::new();
        home.add_device("sensor", DeviceType::Light).unwrap();
        home.set_state("sensor", DeviceState::On).unwrap();
        home.add_device("lamp", DeviceType::Light).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "no_cond",
            Trigger::DeviceStateChange { device_name: "sensor".into(), target_state: DeviceState::On },
            Action::DeviceState { device_name: "lamp".into(), state: DeviceState::On },
            None, vec![],
        ).unwrap();
        assert_eq!(engine.evaluate_rules(&home, &PresenceRegistry::new(), now).len(), 1);
    }

    #[test]
    fn test_presence_trigger_fires_on_match() {
        use crate::domain::presence::{PersonTracker, PresenceRegistry, SourceState};
        let now_naive = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let now_utc = chrono::Utc::now();
        let home = SmartHome::new();
        let mut presence = PresenceRegistry::new();
        let mut alice = PersonTracker::new("alice", 0);
        alice.update_source("network", SourceState::Home, now_utc);
        presence.add(alice).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "welcome",
            Trigger::PresenceChange { person_name: "alice".into(), target_state: PresenceState::Home },
            Action::Notify { message: "Welcome home!".into() },
            None, vec![],
        ).unwrap();
        assert_eq!(engine.evaluate_rules(&home, &presence, now_naive).len(), 1);
    }

    #[test]
    fn test_presence_trigger_does_not_fire_on_mismatch() {
        use crate::domain::presence::{PersonTracker, PresenceRegistry, SourceState};
        let now_naive = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let now_utc = chrono::Utc::now() - chrono::Duration::seconds(200);
        let home = SmartHome::new();
        let mut presence = PresenceRegistry::new();
        let mut alice = PersonTracker::new("alice", 60); // grace = 60s, away for 200s
        alice.update_source("network", SourceState::Away, now_utc);
        presence.add(alice).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "welcome",
            Trigger::PresenceChange { person_name: "alice".into(), target_state: PresenceState::Home },
            Action::Notify { message: "Welcome home!".into() },
            None, vec![],
        ).unwrap();
        // alice is away — trigger expects Home — should not fire
        assert!(engine.evaluate_rules(&home, &presence, now_naive).is_empty());
    }

    #[test]
    fn test_presence_trigger_unknown_person_does_not_fire() {
        let now_naive = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let home = SmartHome::new();
        let presence = PresenceRegistry::new();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "ghost",
            Trigger::PresenceChange { person_name: "nobody".into(), target_state: PresenceState::Home },
            Action::Notify { message: "Ghost!".into() },
            None, vec![],
        ).unwrap();
        assert!(engine.evaluate_rules(&home, &presence, now_naive).is_empty());
    }

    #[test]
    fn safe_mode_suppresses_state_action() {
        let mut home = SmartHome::new();
        use crate::domain::device::DeviceType;
        home.add_device("lamp", DeviceType::Light).unwrap();
        let actions = vec![Action::DeviceState { device_name: "lamp".into(), state: DeviceState::On }];
        AutomationEngine::execute_actions(&actions, &mut home, true); // safe_mode = true
        // State should remain Off (default), not be changed to On
        assert_eq!(home.get_device("lamp").unwrap().state, DeviceState::Off);
    }

    #[test]
    fn safe_mode_allows_notify_action() {
        let mut home = SmartHome::new();
        let actions = vec![Action::Notify { message: "alert!".into() }];
        let msgs = AutomationEngine::execute_actions(&actions, &mut home, true); // safe_mode = true
        assert_eq!(msgs, vec!["alert!"]);
    }

    #[test]
    fn toggle_safe_mode_flips_flag() {
        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "test",
            Trigger::Time { time: "00:00".into() },
            Action::Notify { message: "hi".into() },
            None, vec![],
        ).unwrap();
        assert!(!engine.rules.get("test").unwrap().safe_mode);
        let new_val = engine.toggle_safe_mode("test").unwrap();
        assert!(new_val);
        assert!(engine.rules.get("test").unwrap().safe_mode);
        let new_val2 = engine.toggle_safe_mode("test").unwrap();
        assert!(!new_val2);
    }
}
