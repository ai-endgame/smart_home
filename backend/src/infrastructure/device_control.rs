/// Fire-and-forget HTTP control for physical smart-home devices.
///
/// Every function in this module is intentionally best-effort: errors are
/// logged but never propagated to the caller.  The pattern is:
///
///   ```ignore
///   tokio::spawn(send_state(device.clone(), new_state));
///   ```
use crate::domain::device::{Device, DeviceState, Protocol};

/// Send an on/off command to the physical device.
pub async fn send_state(device: Device, state: &DeviceState) {
    let (endpoint, protocol) = match extract(&device) {
        Some(v) => v,
        None => return,
    };
    let on = matches!(state, DeviceState::On);
    if let Err(e) = dispatch_state(&endpoint, &protocol, on).await {
        log::warn!("device_control: failed to set state on '{}' ({}): {}", device.name, endpoint, e);
    }
}

/// Send a brightness command (0-100) to the physical device.
pub async fn send_brightness(device: Device, brightness: u8) {
    let (endpoint, protocol) = match extract(&device) {
        Some(v) => v,
        None => return,
    };
    if let Err(e) = dispatch_brightness(&endpoint, &protocol, brightness).await {
        log::warn!("device_control: failed to set brightness on '{}' ({}): {}", device.name, endpoint, e);
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn extract(device: &Device) -> Option<(String, String)> {
    match (&device.endpoint, &device.control_protocol) {
        (Some(ep), Some(proto)) => Some((ep.clone(), proto.to_string())),
        _ => None, // no endpoint configured — silently skip
    }
}

async fn dispatch_state(endpoint: &str, protocol: &str, on: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    match protocol {
        // Shelly Gen1: GET /relay/0?turn=on|off
        "shelly" => {
            let turn = if on { "on" } else { "off" };
            let url = format!("{}/relay/0?turn={}", endpoint, turn);
            client.get(&url).send().await?;
        }
        // WLED: POST /json/state {"on": true/false}
        "wled" => {
            let body = serde_json::json!({ "on": on });
            client.post(format!("{}/json/state", endpoint)).json(&body).send().await?;
        }
        // Tasmota: GET /cm?cmnd=Power%20On|Off
        "tasmota" => {
            let cmd = if on { "Power%20On" } else { "Power%20Off" };
            let url = format!("{}/cm?cmnd={}", endpoint, cmd);
            client.get(&url).send().await?;
        }
        // ESPHome native API is not HTTP — log and skip
        "esphome" => {
            log::debug!("device_control: ESPHome '{}' uses native API, skipping HTTP control", endpoint);
        }
        other => {
            log::debug!("device_control: unknown protocol '{}' for {}, skipping", other, endpoint);
        }
    }
    Ok(())
}

async fn dispatch_brightness(endpoint: &str, protocol: &str, brightness: u8) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    match protocol {
        // WLED: POST /json/state {"bri": 1-255, "on": true}
        "wled" => {
            let bri = ((brightness as u16 * 255) / 100).max(1) as u8;
            let body = serde_json::json!({ "bri": bri, "on": brightness > 0 });
            client.post(format!("{}/json/state", endpoint)).json(&body).send().await?;
        }
        // Shelly Dimmer Gen1: GET /light/0?brightness=0-100&turn=on
        "shelly" => {
            let turn = if brightness > 0 { "on" } else { "off" };
            let url = format!("{}/light/0?brightness={}&turn={}", endpoint, brightness, turn);
            client.get(&url).send().await?;
        }
        other => {
            log::debug!("device_control: brightness not supported for protocol '{}' at {}", other, endpoint);
        }
    }
    Ok(())
}

/// Derive the control protocol from an mDNS service type string.
pub fn protocol_from_service_type(service_type: &str) -> Option<Protocol> {
    match service_type {
        s if s.starts_with("_shelly.")     => Some(Protocol::Shelly),
        s if s.starts_with("_wled.")       => Some(Protocol::WLED),
        s if s.starts_with("_esphomelib.") => Some(Protocol::ESPHome),
        s if s.starts_with("_arduino.")    => Some(Protocol::ESPHome), // treat Arduino as ESPHome-like
        _ => None,
    }
}
