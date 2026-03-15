use std::fmt;

use crate::domain::device::DeviceState;
use crate::domain::manager::SmartHome;

/// What triggers an automation rule.
#[derive(Debug, Clone)]
pub enum Trigger {
    DeviceStateChange { device_name: String, target_state: DeviceState },
    TemperatureAbove { device_name: String, threshold: f64 },
    TemperatureBelow { device_name: String, threshold: f64 },
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
        }
    }
}

/// What action to perform when a rule triggers.
#[derive(Debug, Clone)]
pub enum Action {
    DeviceState { device_name: String, state: DeviceState },
    Brightness { device_name: String, brightness: u8 },
    Temperature { device_name: String, temperature: f64 },
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutomationRule {
    pub name: String,
    pub trigger: Trigger,
    pub action: Action,
    pub enabled: bool,
}

impl fmt::Display for AutomationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.enabled { "enabled" } else { "disabled" };
        write!(f, "[{}] {} → {} ({})", self.name, self.trigger, self.action, status)
    }
}

pub struct AutomationEngine {
    pub rules: Vec<AutomationRule>,
}

impl AutomationEngine {
    pub fn new() -> Self {
        AutomationEngine { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, name: &str, trigger: Trigger, action: Action) -> Result<(), String> {
        if self.rules.iter().any(|r| r.name.to_lowercase() == name.to_lowercase()) {
            return Err(format!("Rule '{}' already exists.", name));
        }
        self.rules.push(AutomationRule { name: name.to_string(), trigger, action, enabled: true });
        Ok(())
    }

    pub fn remove_rule(&mut self, name: &str) -> Result<(), String> {
        let idx = self.rules.iter()
            .position(|r| r.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| format!("Rule '{}' not found.", name))?;
        self.rules.remove(idx);
        Ok(())
    }

    pub fn toggle_rule(&mut self, name: &str) -> Result<bool, String> {
        let rule = self.rules.iter_mut()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| format!("Rule '{}' not found.", name))?;
        rule.enabled = !rule.enabled;
        Ok(rule.enabled)
    }

    pub fn evaluate_rules(&self, home: &SmartHome) -> Vec<Action> {
        self.rules.iter()
            .filter(|r| r.enabled)
            .filter_map(|rule| {
                let triggered = match &rule.trigger {
                    Trigger::DeviceStateChange { device_name, target_state } =>
                        home.get_device(device_name).map(|d| &d.state == target_state).unwrap_or(false),
                    Trigger::TemperatureAbove { device_name, threshold } =>
                        home.get_device(device_name).and_then(|d| d.temperature).map(|t| t > *threshold).unwrap_or(false),
                    Trigger::TemperatureBelow { device_name, threshold } =>
                        home.get_device(device_name).and_then(|d| d.temperature).map(|t| t < *threshold).unwrap_or(false),
                };
                triggered.then(|| rule.action.clone())
            })
            .collect()
    }

    pub fn execute_actions(actions: &[Action], home: &mut SmartHome) {
        for action in actions {
            match action {
                Action::DeviceState { device_name, state } => { let _ = home.set_state(device_name, state.clone()); }
                Action::Brightness { device_name, brightness } => { let _ = home.set_brightness(device_name, *brightness); }
                Action::Temperature { device_name, temperature } => { let _ = home.set_temperature(device_name, *temperature); }
            }
        }
    }

    pub fn list_rules(&self) -> &[AutomationRule] {
        &self.rules
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
        ).unwrap();
        assert!(engine.add_rule("Night", Trigger::DeviceStateChange { device_name: "x".into(), target_state: DeviceState::On }, Action::DeviceState { device_name: "y".into(), state: DeviceState::Off }).is_err());
        let enabled = engine.toggle_rule("night").unwrap();
        assert!(!enabled);
    }

    #[test]
    fn test_evaluate_and_execute() {
        let mut home = SmartHome::new();
        home.add_device("thermo", DeviceType::Thermostat).unwrap();
        home.set_temperature("thermo", 30.0).unwrap();

        let mut engine = AutomationEngine::new();
        engine.add_rule(
            "cool",
            Trigger::TemperatureAbove { device_name: "thermo".into(), threshold: 25.0 },
            Action::Temperature { device_name: "thermo".into(), temperature: 22.0 },
        ).unwrap();

        let actions = engine.evaluate_rules(&home);
        assert_eq!(actions.len(), 1);
        AutomationEngine::execute_actions(&actions, &mut home);
        assert_eq!(home.get_device("thermo").unwrap().temperature, Some(22.0));
    }
}
