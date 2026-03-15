use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_cli_script(script: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_smart_home"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn smart_home binary");

    child.stdin.as_mut().expect("stdin unavailable")
        .write_all(script.as_bytes())
        .expect("failed to write CLI script");

    child.wait_with_output().expect("failed waiting on CLI")
}

#[test]
fn smart_home_cli_covers_core_command_paths() {
    let script = [
        "help",
        "add-device",
        "add-device lamp light",
        "add-device thermo thermostat",
        "add-device sensor sensor",
        "add-device lamp light",
        "remove-device missing",
        "set-state lamp maybe",
        "set-state lamp on",
        "set-brightness lamp abc",
        "set-brightness lamp 55",
        "set-brightness thermo 15",
        "set-temp thermo bad",
        "set-temp thermo 24.5",
        "list-devices",
        "device-info lamp",
        "device-info missing",
        "add-room",
        "add-room kitchen",
        "add-room kitchen",
        "assign lamp",
        "assign lamp kitchen",
        "assign missing kitchen",
        "list-rooms",
        "room-info kitchen",
        "room-info unknown",
        "add-rule",
        "add-rule bad-rule state:lamp:maybe state:lamp:on",
        "add-rule night state:sensor:on state:lamp:on",
        "add-rule cool temp-above:thermo:23 temp:thermo:20",
        "list-rules",
        "toggle-rule missing",
        "toggle-rule night",
        "toggle-rule night",
        "run-rules",
        "set-state sensor on",
        "run-rules",
        "remove-rule missing",
        "remove-rule night",
        "status",
        "unknown-command",
        "quit",
    ].join("\n");

    let output = run_cli_script(&(script + "\n"));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Smart Home Manager"));
    assert!(stdout.contains("Unknown command"));
    assert!(stdout.contains("Actions executed"));
    assert!(stdout.contains("Goodbye"));
}

#[test]
fn smart_home_server_binary_fails_on_invalid_addr_arg() {
    let output = Command::new(env!("CARGO_BIN_EXE_smart_home_server"))
        .arg("--addr")
        .arg("invalid_addr")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn smart_home_server binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid bind address"));
}

#[test]
fn smart_home_server_binary_reads_env_addr() {
    let output = Command::new(env!("CARGO_BIN_EXE_smart_home_server"))
        .env("SMART_HOME_BIND_ADDR", "invalid_addr")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn smart_home_server binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid bind address"));
}
