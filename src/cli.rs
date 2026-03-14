use std::io::{self, Write};

use crate::automation::{Action, AutomationEngine, Trigger};
use crate::manager::SmartHome;
use crate::models::{DeviceState, DeviceType};

const HELP_TEXT: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║                   Smart Home CLI — Commands                  ║
╠══════════════════════════════════════════════════════════════╣
║  Devices                                                     ║
║    add-device <name> <type>       Add a device               ║
║    remove-device <name>           Remove a device             ║
║    set-state <device> on|off      Turn device on/off          ║
║    set-brightness <device> <0-100> Set light brightness       ║
║    set-temp <device> <temp>       Set thermostat temp         ║
║    list-devices                   List all devices            ║
║    device-info <name>             Show device details         ║
║                                                              ║
║  Rooms                                                       ║
║    add-room <name>                Create a room               ║
║    assign <device> <room>         Assign device to room       ║
║    list-rooms                     List all rooms              ║
║    room-info <name>               Show room details           ║
║                                                              ║
║  Automation                                                  ║
║    add-rule <name> <trigger> <action>  Add an automation rule ║
║    remove-rule <name>             Remove a rule               ║
║    toggle-rule <name>             Enable/disable a rule       ║
║    list-rules                     List all rules              ║
║    run-rules                      Evaluate & execute rules    ║
║                                                              ║
║  General                                                     ║
║    status                         Home overview               ║
║    help                           Show this help              ║
║    quit                           Exit                        ║
║                                                              ║
║  Device types: light, thermostat, lock, switch, sensor       ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Run the interactive CLI loop.
pub fn run_cli() {
    let mut home = SmartHome::new();
    let mut engine = AutomationEngine::new();

    println!();
    println!("🏠  Smart Home Manager v0.1.0");
    println!("   Type 'help' for available commands.\n");

    loop {
        print!("smart-home> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input.");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(4, ' ').collect();
        let command = parts[0].to_lowercase();

        match command.as_str() {
            "help" => println!("{}", HELP_TEXT),
            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }

            // ── Devices ─────────────────────────────────────────
            "add-device" => {
                if parts.len() < 3 {
                    println!("Usage: add-device <name> <type>");
                    println!("  Types: light, thermostat, lock, switch, sensor");
                    continue;
                }
                let name = parts[1];
                let type_str = parts[2];
                match DeviceType::from_str_loose(type_str) {
                    Some(dt) => match home.add_device(name, dt) {
                        Ok(id) => println!("✅ Added device '{}' (id: {})", name, &id[..8]),
                        Err(e) => println!("❌ {}", e),
                    },
                    None => println!(
                        "❌ Unknown device type '{}'. Use: light, thermostat, lock, switch, sensor",
                        type_str
                    ),
                }
            }

            "remove-device" => {
                if parts.len() < 2 {
                    println!("Usage: remove-device <name>");
                    continue;
                }
                match home.remove_device(parts[1]) {
                    Ok(_) => println!("✅ Removed device '{}'.", parts[1]),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "set-state" => {
                if parts.len() < 3 {
                    println!("Usage: set-state <device> on|off");
                    continue;
                }
                let state = match parts[2].to_lowercase().as_str() {
                    "on" => DeviceState::On,
                    "off" => DeviceState::Off,
                    _ => {
                        println!("❌ State must be 'on' or 'off'.");
                        continue;
                    }
                };
                match home.set_state(parts[1], state) {
                    Ok(_) => println!("✅ '{}' is now {}.", parts[1], parts[2].to_uppercase()),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "set-brightness" => {
                if parts.len() < 3 {
                    println!("Usage: set-brightness <device> <0-100>");
                    continue;
                }
                match parts[2].parse::<u8>() {
                    Ok(val) => match home.set_brightness(parts[1], val) {
                        Ok(_) => println!("✅ '{}' brightness set to {}%.", parts[1], val),
                        Err(e) => println!("❌ {}", e),
                    },
                    Err(_) => println!("❌ Brightness must be a number 0–100."),
                }
            }

            "set-temp" => {
                if parts.len() < 3 {
                    println!("Usage: set-temp <device> <temperature>");
                    continue;
                }
                match parts[2].parse::<f64>() {
                    Ok(val) => match home.set_temperature(parts[1], val) {
                        Ok(_) => println!("✅ '{}' temperature set to {:.1}°C.", parts[1], val),
                        Err(e) => println!("❌ {}", e),
                    },
                    Err(_) => println!("❌ Temperature must be a number."),
                }
            }

            "list-devices" => {
                let devices = home.list_devices();
                if devices.is_empty() {
                    println!("No devices registered.");
                } else {
                    println!("── Devices ({}) ──", devices.len());
                    for d in devices {
                        println!("  {}", d);
                    }
                }
            }

            "device-info" => {
                if parts.len() < 2 {
                    println!("Usage: device-info <name>");
                    continue;
                }
                match home.get_device(parts[1]) {
                    Some(d) => println!("{}", d),
                    None => println!("❌ Device '{}' not found.", parts[1]),
                }
            }

            // ── Rooms ───────────────────────────────────────────
            "add-room" => {
                if parts.len() < 2 {
                    println!("Usage: add-room <name>");
                    continue;
                }
                match home.add_room(parts[1]) {
                    Ok(_) => println!("✅ Room '{}' created.", parts[1]),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "assign" => {
                if parts.len() < 3 {
                    println!("Usage: assign <device> <room>");
                    continue;
                }
                match home.assign_device_to_room(parts[1], parts[2]) {
                    Ok(_) => println!("✅ '{}' assigned to room '{}'.", parts[1], parts[2]),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "list-rooms" => {
                let rooms = home.list_rooms();
                if rooms.is_empty() {
                    println!("No rooms created.");
                } else {
                    println!("── Rooms ({}) ──", rooms.len());
                    for r in rooms {
                        println!("  {}", r);
                    }
                }
            }

            "room-info" => {
                if parts.len() < 2 {
                    println!("Usage: room-info <name>");
                    continue;
                }
                let devices = home.get_room_devices(parts[1]);
                if devices.is_empty() {
                    println!("Room '{}' has no devices (or does not exist).", parts[1]);
                } else {
                    println!("── Room '{}' ({} devices) ──", parts[1], devices.len());
                    for d in devices {
                        println!("  {}", d);
                    }
                }
            }

            // ── Automation ──────────────────────────────────────
            "add-rule" => {
                if parts.len() < 4 {
                    println!("Usage: add-rule <name> <trigger> <action>");
                    println!(
                        "  Triggers: state:<device>:on|off  temp-above:<device>:<val>  temp-below:<device>:<val>"
                    );
                    println!(
                        "  Actions:  state:<device>:on|off  brightness:<device>:<val>  temp:<device>:<val>"
                    );
                    continue;
                }

                let rule_name = parts[1];
                let trigger = match parse_trigger(parts[2]) {
                    Some(t) => t,
                    None => {
                        println!(
                            "❌ Invalid trigger format. Use: state:<device>:on|off  temp-above:<device>:<val>  temp-below:<device>:<val>"
                        );
                        continue;
                    }
                };
                let action = match parse_action(parts[3]) {
                    Some(a) => a,
                    None => {
                        println!(
                            "❌ Invalid action format. Use: state:<device>:on|off  brightness:<device>:<val>  temp:<device>:<val>"
                        );
                        continue;
                    }
                };

                match engine.add_rule(rule_name, trigger, action) {
                    Ok(_) => println!("✅ Rule '{}' added.", rule_name),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "remove-rule" => {
                if parts.len() < 2 {
                    println!("Usage: remove-rule <name>");
                    continue;
                }
                match engine.remove_rule(parts[1]) {
                    Ok(_) => println!("✅ Rule '{}' removed.", parts[1]),
                    Err(e) => println!("❌ {}", e),
                }
            }

            "toggle-rule" => {
                if parts.len() < 2 {
                    println!("Usage: toggle-rule <name>");
                    continue;
                }
                match engine.toggle_rule(parts[1]) {
                    Ok(enabled) => {
                        let status = if enabled { "enabled" } else { "disabled" };
                        println!("✅ Rule '{}' is now {}.", parts[1], status);
                    }
                    Err(e) => println!("❌ {}", e),
                }
            }

            "list-rules" => {
                let rules = engine.list_rules();
                if rules.is_empty() {
                    println!("No automation rules defined.");
                } else {
                    println!("── Rules ({}) ──", rules.len());
                    for r in rules {
                        println!("  {}", r);
                    }
                }
            }

            "run-rules" => {
                let actions = engine.evaluate_rules(&home);
                if actions.is_empty() {
                    println!("No rules triggered.");
                } else {
                    println!("⚡ {} action(s) triggered:", actions.len());
                    for a in &actions {
                        println!("  → {}", a);
                    }
                    AutomationEngine::execute_actions(&actions, &mut home);
                    println!("✅ Actions executed.");
                }
            }

            // ── Status ──────────────────────────────────────────
            "status" => {
                let devices = home.list_devices();
                let rooms = home.list_rooms();
                let rules = engine.list_rules();

                println!("╔════════════════════════════════╗");
                println!("║      🏠 Smart Home Status      ║");
                println!("╠════════════════════════════════╣");
                println!("║  Devices: {:<20} ║", devices.len());
                println!("║  Rooms:   {:<20} ║", rooms.len());
                println!("║  Rules:   {:<20} ║", rules.len());
                println!("╚════════════════════════════════╝");

                let on_count = devices
                    .iter()
                    .filter(|d| d.state == DeviceState::On)
                    .count();
                if !devices.is_empty() {
                    println!(
                        "  {} device(s) ON, {} OFF",
                        on_count,
                        devices.len() - on_count
                    );
                }
            }

            _ => println!(
                "❓ Unknown command '{}'. Type 'help' for commands.",
                command
            ),
        }
    }
}

/// Parse a trigger string like "state:lamp:on" or "temp-above:thermo:25".
fn parse_trigger(s: &str) -> Option<Trigger> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    match parts[0] {
        "state" => {
            let state = match parts[2].to_lowercase().as_str() {
                "on" => DeviceState::On,
                "off" => DeviceState::Off,
                _ => return None,
            };
            Some(Trigger::DeviceStateChange {
                device_name: parts[1].to_string(),
                target_state: state,
            })
        }
        "temp-above" => {
            let threshold = parts[2].parse::<f64>().ok()?;
            Some(Trigger::TemperatureAbove {
                device_name: parts[1].to_string(),
                threshold,
            })
        }
        "temp-below" => {
            let threshold = parts[2].parse::<f64>().ok()?;
            Some(Trigger::TemperatureBelow {
                device_name: parts[1].to_string(),
                threshold,
            })
        }
        _ => None,
    }
}

/// Parse an action string like "state:lamp:on" or "brightness:lamp:50".
fn parse_action(s: &str) -> Option<Action> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    match parts[0] {
        "state" => {
            let state = match parts[2].to_lowercase().as_str() {
                "on" => DeviceState::On,
                "off" => DeviceState::Off,
                _ => return None,
            };
            Some(Action::DeviceState {
                device_name: parts[1].to_string(),
                state,
            })
        }
        "brightness" => {
            let val = parts[2].parse::<u8>().ok()?;
            Some(Action::Brightness {
                device_name: parts[1].to_string(),
                brightness: val,
            })
        }
        "temp" => {
            let val = parts[2].parse::<f64>().ok()?;
            Some(Action::Temperature {
                device_name: parts[1].to_string(),
                temperature: val,
            })
        }
        _ => None,
    }
}
