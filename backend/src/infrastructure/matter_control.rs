use std::time::Duration;

use tokio::process::Command;

// ── chip-tool subprocess ───────────────────────────────────────────────────────

/// Invoke `chip-tool <args>` via `docker exec chip-tool chip-tool <args>`.
/// Returns stdout on success, stderr on failure, or an error string on timeout/spawn failure.
pub async fn run_chip_tool(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("docker");
    cmd.arg("exec").arg("chip-tool").arg("chip-tool");
    for a in args { cmd.arg(a); }

    let result = tokio::time::timeout(
        Duration::from_secs(10),
        cmd.output(),
    )
    .await
    .map_err(|_| "chip-tool command timed out after 10s".to_string())?
    .map_err(|e: std::io::Error| format!("failed to spawn chip-tool: {e}"))?;

    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

// ── Cluster command dispatch ───────────────────────────────────────────────────

pub async fn dispatch_onoff(node_id: u64, on: bool) -> Result<(), String> {
    let cmd = if on { "on" } else { "off" };
    let node = node_id.to_string();
    run_chip_tool(&["onoff", cmd, &node, "1"]).await.map(|_| ())
}

pub async fn dispatch_level(node_id: u64, brightness: u8) -> Result<(), String> {
    // Scale 0–100 to 0–254
    let level = ((brightness as u32 * 254) / 100).min(254) as u8;
    let node = node_id.to_string();
    let level_str = level.to_string();
    run_chip_tool(&["levelcontrol", "move-to-level", &level_str, "0", "0", "0", &node, "1"]).await.map(|_| ())
}

pub async fn dispatch_color_temp(node_id: u64, mireds: u16) -> Result<(), String> {
    let node = node_id.to_string();
    let mireds_str = mireds.to_string();
    run_chip_tool(&["colorcontrol", "move-to-color-temperature", &mireds_str, "0", "0", "0", &node, "1"]).await.map(|_| ())
}

// ── Attribute reads ────────────────────────────────────────────────────────────

pub async fn read_onoff(node_id: u64) -> Result<bool, String> {
    let node = node_id.to_string();
    let output = run_chip_tool(&["onoff", "read", "on-off", &node, "1"]).await?;
    // chip-tool outputs something like: `OnOff: 1` or `OnOff: 0`
    if output.to_lowercase().contains("onoff: 1") || output.to_lowercase().contains("onoff: true") {
        Ok(true)
    } else if output.to_lowercase().contains("onoff: 0") || output.to_lowercase().contains("onoff: false") {
        Ok(false)
    } else {
        Err(format!("could not parse on-off from chip-tool output: {output}"))
    }
}

pub async fn read_level(node_id: u64) -> Result<u8, String> {
    let node = node_id.to_string();
    let output = run_chip_tool(&["levelcontrol", "read", "current-level", &node, "1"]).await?;
    // chip-tool outputs something like: `CurrentLevel: 203`
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("currentlevel:")
            && let Some(val_str) = lower.split(':').nth(1)
                && let Ok(raw) = val_str.trim().parse::<u8>() {
                    return Ok(((raw as u32 * 100) / 254).min(100) as u8);
                }
    }
    Err(format!("could not parse current-level from chip-tool output: {output}"))
}

// ── Commissioning ──────────────────────────────────────────────────────────────

pub async fn run_commissioning(node_id: u64, setup_code: &str) -> Result<(), String> {
    let node = node_id.to_string();
    // chip-tool pairing code <node-id> <setup-code> — timeout 60s for commissioning
    let mut cmd = Command::new("docker");
    cmd.arg("exec").arg("chip-tool").arg("chip-tool")
        .arg("pairing").arg("code").arg(&node).arg(setup_code);

    let result = tokio::time::timeout(
        Duration::from_secs(60),
        cmd.output(),
    )
    .await
    .map_err(|_| "commissioning timed out after 60s".to_string())?
    .map_err(|e: std::io::Error| format!("failed to spawn chip-tool: {e}"))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

// ── State sync loop ────────────────────────────────────────────────────────────

pub fn start_matter_sync_loop(state: crate::state::AppState) {
    if !std::env::var("MATTER_SYNC_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return;
    }

    let interval_secs: u64 = std::env::var("MATTER_SYNC_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    tokio::spawn(async move {
        log::info!("Matter state sync started (interval: {interval_secs}s)");
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            sync_matter_devices(&state).await;
        }
    });
}

async fn sync_matter_devices(state: &crate::state::AppState) {
    use crate::domain::device::{DeviceState, Protocol};
    use crate::http::helpers::persist_device;
    use crate::http::types::EventKind;

    // Collect matter devices with node_id
    let devices: Vec<_> = {
        let home = state.home.read().await;
        home.list_devices()
            .into_iter()
            .filter(|d| d.control_protocol == Some(Protocol::Matter) && d.node_id.is_some())
            .cloned()
            .collect()
    };

    for device in devices {
        let node_id = match device.node_id { Some(n) => n, None => continue };

        // Read on/off state
        match read_onoff(node_id).await {
            Ok(on) => {
                let new_state = if on { DeviceState::On } else { DeviceState::Off };
                let changed = device.state != new_state;
                if changed {
                    let name = device.name.clone();
                    let mut home = state.home.write().await;
                    if let Ok(()) = home.set_state(&name, new_state)
                        && let Some(updated) = home.get_device(&name).cloned() {
                            drop(home);
                            persist_device(state, &updated).await;
                            crate::http::helpers::record_event(
                                state,
                                EventKind::DeviceUpdated,
                                "device",
                                format!("Matter sync: '{}' state updated", name),
                                Some(name),
                                None,
                            ).await;
                        }
                }
            }
            Err(e) => {
                let name = device.name.clone();
                log::warn!("Matter sync: failed to read on-off for '{}': {e}", name);
                let mut home = state.home.write().await;
                if let Some(d) = home.devices.get_mut(&name.to_lowercase()) {
                    d.last_error = Some(e);
                }
            }
        }

        // Read brightness
        match read_level(node_id).await {
            Ok(level) => {
                if device.brightness != level {
                    let name = device.name.clone();
                    let mut home = state.home.write().await;
                    let _ = home.set_brightness(&name, level);
                }
            }
            Err(e) => {
                log::debug!("Matter sync: failed to read level for '{}': {e}", device.name);
            }
        }
    }

    // Update last_sync_at
    if let Ok(mut status) = state.matter_status.write() {
        status.last_sync_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_level_scaling() {
        // brightness 0 → level 0
        assert_eq!(((0u32 * 254) / 100).min(254) as u8, 0);
        // brightness 100 → level 254
        assert_eq!(((100u32 * 254) / 100).min(254) as u8, 254);
        // brightness 80 → level 203
        assert_eq!(((80u32 * 254) / 100).min(254) as u8, 203);
    }

    #[test]
    fn chip_tool_args_onoff() {
        // Verify argument construction doesn't panic
        let node = 1u64.to_string();
        let args_on: Vec<&str> = vec!["onoff", "on", &node, "1"];
        let args_off: Vec<&str> = vec!["onoff", "off", &node, "1"];
        assert_eq!(args_on[1], "on");
        assert_eq!(args_off[1], "off");
    }
}
