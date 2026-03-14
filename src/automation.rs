use std::fmt;

use crate::manager::SmartHome;
use crate::models::DeviceState;

/// What triggers an automation rule.
#[derive(Debug, Clone)]
pub enum Trigger {
    /// Fires when a named device changes to a specific state.
    DeviceStateChange {
        device_name: String,
        target_state: DeviceState,
    },
    /// Fires when temperature on a device crosses a threshold.
    TemperatureAbove { device_name: String, threshold: f64 },
    /// Fires when temperature on a device drops below a threshold.
    TemperatureBelow { device_name: String, threshold: f64 },
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trigger::DeviceStateChange {
                device_name,
                target_state,
            } => write!(f, "when '{}' turns {}", device_name, target_state),
            Trigger::TemperatureAbove {
                device_name,
                threshold,
            } => write!(f, "when '{}' temp > {:.1}°C", device_name, threshold),
            Trigger::TemperatureBelow {
                device_name,
                threshold,
            } => write!(f, "when '{}' temp < {:.1}°C", device_name, threshold),
        }
    }
}

/// What action to perform when a rule triggers.
#[derive(Debug, Clone)]
pub enum Action {
    DeviceState {
        device_name: String,
        state: DeviceState,
    },
    Brightness {
        device_name: String,
        brightness: u8,
    },
    Temperature {
        device_name: String,
        temperature: f64,
    },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::DeviceState { device_name, state } => {
                write!(f, "set '{}' to {}", device_name, state)
            }
            Action::Brightness {
                device_name,
                brightness,
            } => write!(f, "set '{}' brightness to {}%", device_name, brightness),
            Action::Temperature {
                device_name,
                temperature,
            } => write!(f, "set '{}' temp to {:.1}°C", device_name, temperature),
        }
    }
}

/// A named automation rule pairing a trigger with an action.
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
        write!(
            f,
            "[{}] {} → {} ({})",
            self.name, self.trigger, self.action, status
        )
    }
}

/// Engine that manages and evaluates automation rules.
pub struct AutomationEngine {
    pub rules: Vec<AutomationRule>,
}

impl AutomationEngine {
    pub fn new() -> Self {
        AutomationEngine { rules: Vec::new() }
    }

    /// Add a new automation rule.
    pub fn add_rule(&mut self, name: &str, trigger: Trigger, action: Action) -> Result<(), String> {
        if self
            .rules
            .iter()
            .any(|r| r.name.to_lowercase() == name.to_lowercase())
        {
            return Err(format!("Rule '{}' already exists.", name));
        }

        self.rules.push(AutomationRule {
            name: name.to_string(),
            trigger,
            action,
            enabled: true,
        });

        Ok(())
    }

    /// Remove a rule by name.
    pub fn remove_rule(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .rules
            .iter()
            .position(|r| r.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| format!("Rule '{}' not found.", name))?;
        self.rules.remove(idx);
        Ok(())
    }

    /// Toggle a rule on or off.
    pub fn toggle_rule(&mut self, name: &str) -> Result<bool, String> {
        let rule = self
            .rules
            .iter_mut()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| format!("Rule '{}' not found.", name))?;
        rule.enabled = !rule.enabled;
        Ok(rule.enabled)
    }

    /// Evaluate all enabled rules against the current smart home state.
    /// Returns a list of actions that should be executed.
    pub fn evaluate_rules(&self, home: &SmartHome) -> Vec<Action> {
        let mut actions_to_run = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let triggered = match &rule.trigger {
                Trigger::DeviceStateChange {
                    device_name,
                    target_state,
                } => home
                    .get_device(device_name)
                    .map(|d| &d.state == target_state)
                    .unwrap_or(false),

                Trigger::TemperatureAbove {
                    device_name,
                    threshold,
                } => home
                    .get_device(device_name)
                    .and_then(|d| d.temperature)
                    .map(|t| t > *threshold)
                    .unwrap_or(false),

                Trigger::TemperatureBelow {
                    device_name,
                    threshold,
                } => home
                    .get_device(device_name)
                    .and_then(|d| d.temperature)
                    .map(|t| t < *threshold)
                    .unwrap_or(false),
            };

            if triggered {
                actions_to_run.push(rule.action.clone());
            }
        }

        actions_to_run
    }

    /// Actually execute a list of actions against the smart home.
    pub fn execute_actions(actions: &[Action], home: &mut SmartHome) {
        for action in actions {
            match action {
                Action::DeviceState { device_name, state } => {
                    let _ = home.set_state(device_name, state.clone());
                }
                Action::Brightness {
                    device_name,
                    brightness,
                } => {
                    let _ = home.set_brightness(device_name, *brightness);
                }
                Action::Temperature {
                    device_name,
                    temperature,
                } => {
                    let _ = home.set_temperature(device_name, *temperature);
                }
            }
        }
    }

    /// List all rules.
    pub fn list_rules(&self) -> &[AutomationRule] {
        &self.rules
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DeviceType;

    #[test]
    fn test_add_and_list_rules() {
        let mut engine = AutomationEngine::new();
        engine
            .add_rule(
                "night mode",
                Trigger::DeviceStateChange {
                    device_name: "main light".to_string(),
                    target_state: DeviceState::Off,
                },
                Action::Brightness {
                    device_name: "desk lamp".to_string(),
                    brightness: 30,
                },
            )
            .unwrap();

        assert_eq!(engine.list_rules().len(), 1);
        assert_eq!(engine.list_rules()[0].name, "night mode");
    }

    #[test]
    fn test_duplicate_rule_name() {
        let mut engine = AutomationEngine::new();
        let trigger = Trigger::DeviceStateChange {
            device_name: "x".into(),
            target_state: DeviceState::On,
        };
        let action = Action::DeviceState {
            device_name: "y".into(),
            state: DeviceState::Off,
        };

        engine
            .add_rule("r1", trigger.clone(), action.clone())
            .unwrap();
        assert!(engine.add_rule("R1", trigger, action).is_err());
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = AutomationEngine::new();
        engine
            .add_rule(
                "test",
                Trigger::DeviceStateChange {
                    device_name: "x".into(),
                    target_state: DeviceState::On,
                },
                Action::DeviceState {
                    device_name: "y".into(),
                    state: DeviceState::Off,
                },
            )
            .unwrap();

        engine.remove_rule("test").unwrap();
        assert!(engine.list_rules().is_empty());
    }

    #[test]
    fn test_toggle_rule() {
        let mut engine = AutomationEngine::new();
        engine
            .add_rule(
                "test",
                Trigger::DeviceStateChange {
                    device_name: "x".into(),
                    target_state: DeviceState::On,
                },
                Action::DeviceState {
                    device_name: "y".into(),
                    state: DeviceState::Off,
                },
            )
            .unwrap();

        let enabled = engine.toggle_rule("test").unwrap();
        assert!(!enabled);
        let enabled = engine.toggle_rule("test").unwrap();
        assert!(enabled);
    }

    #[test]
    fn test_evaluate_state_trigger() {
        let mut home = SmartHome::new();
        home.add_device("sensor", DeviceType::Sensor).unwrap();
        home.set_state("sensor", DeviceState::On).unwrap();

        let mut engine = AutomationEngine::new();
        engine
            .add_rule(
                "sensor activates light",
                Trigger::DeviceStateChange {
                    device_name: "sensor".to_string(),
                    target_state: DeviceState::On,
                },
                Action::DeviceState {
                    device_name: "lamp".to_string(),
                    state: DeviceState::On,
                },
            )
            .unwrap();

        let actions = engine.evaluate_rules(&home);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_evaluate_temp_trigger() {
        let mut home = SmartHome::new();
        home.add_device("thermo", DeviceType::Thermostat).unwrap();
        home.set_temperature("thermo", 30.0).unwrap();

        let mut engine = AutomationEngine::new();
        engine
            .add_rule(
                "cool down",
                Trigger::TemperatureAbove {
                    device_name: "thermo".to_string(),
                    threshold: 25.0,
                },
                Action::Temperature {
                    device_name: "thermo".to_string(),
                    temperature: 22.0,
                },
            )
            .unwrap();

        let actions = engine.evaluate_rules(&home);
        assert_eq!(actions.len(), 1);

        AutomationEngine::execute_actions(&actions, &mut home);
        assert_eq!(home.get_device("thermo").unwrap().temperature, Some(22.0));
    }

    #[test]
    fn test_display_and_action_execution_paths() {
        let trigger = Trigger::TemperatureBelow {
            device_name: "thermo".to_string(),
            threshold: 18.0,
        };
        assert!(format!("{}", trigger).contains("temp < 18.0"));

        let action = Action::Brightness {
            device_name: "lamp".to_string(),
            brightness: 10,
        };
        assert!(format!("{}", action).contains("brightness"));

        let rule = AutomationRule {
            name: "night".to_string(),
            trigger,
            action,
            enabled: false,
        };
        assert!(format!("{}", rule).contains("disabled"));

        let mut home = SmartHome::new();
        home.add_device("lamp", DeviceType::Light).unwrap();
        home.add_device("thermo", DeviceType::Thermostat).unwrap();

        let actions = vec![
            Action::DeviceState {
                device_name: "lamp".to_string(),
                state: DeviceState::On,
            },
            Action::Brightness {
                device_name: "lamp".to_string(),
                brightness: 25,
            },
            Action::Temperature {
                device_name: "thermo".to_string(),
                temperature: 19.0,
            },
        ];

        AutomationEngine::execute_actions(&actions, &mut home);
        assert_eq!(home.get_device("lamp").unwrap().state, DeviceState::On);
        assert_eq!(home.get_device("lamp").unwrap().brightness, 25);
        assert_eq!(home.get_device("thermo").unwrap().temperature, Some(19.0));
    }
}
