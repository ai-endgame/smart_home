use chrono::Utc;
use rumqttc::{AsyncClient, EventLoop, Packet, QoS};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::device::{DeviceState, Protocol, ZigbeeRole};
use crate::infrastructure::mdns::DiscoveredDevice;
use crate::state::AppState;

// ── MqttStatus ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct MqttStatus {
    pub connected: bool,
    pub broker: Option<String>,
    pub topics_received: u64,
    /// ISO 8601 timestamp of the last received message.
    pub last_message_at: Option<String>,
}

pub type MqttStatusStore = Arc<RwLock<MqttStatus>>;

pub fn new_status_store() -> MqttStatusStore {
    Arc::new(RwLock::new(MqttStatus::default()))
}

// ── URL redaction ─────────────────────────────────────────────────────────────

/// Strip userinfo from the URL and replace the port with `***` to avoid
/// leaking credentials in the status endpoint.
pub fn redact_url(url: &str) -> String {
    // Parse out the scheme, host, and port — drop userinfo and path.
    // Example: mqtt://user:pass@broker.local:1883 → mqtt://broker.local:***
    if let Some(after_scheme) = url.split_once("://") {
        let scheme = after_scheme.0;
        let rest = after_scheme.1;
        // Drop userinfo (everything before '@')
        let without_userinfo = rest.split_once('@').map(|(_, h)| h).unwrap_or(rest);
        // Separate host from port
        if let Some((host, _port)) = without_userinfo.rsplit_once(':') {
            return format!("{}://{}:***", scheme, host);
        }
        return format!("{}://{}:***", scheme, without_userinfo);
    }
    // Fallback: replace anything after last colon
    url.rsplit_once(':')
        .map(|(prefix, _)| format!("{}:***", prefix))
        .unwrap_or_else(|| url.to_string())
}

// ── MQTT subscriber loop ───────────────────────────────────────────────────────

/// Start the MQTT subscriber loop in a background task.
/// The loop subscribes to `zigbee2mqtt/#` and syncs state into `AppState`.
pub async fn start_mqtt_loop(mqtt_url: &str, client: AsyncClient, mut event_loop: EventLoop, state: AppState) {
    let redacted = redact_url(mqtt_url);

    // Subscribe to all zigbee2mqtt topics
    if let Err(e) = client.subscribe("zigbee2mqtt/#", QoS::AtMostOnce).await {
        log::error!("mqtt: failed to subscribe: {e}");
        return;
    }

    // Mark connected
    {
        let mut status = state.mqtt_status.write().await;
        status.connected = true;
        status.broker = Some(redacted.clone());
    }
    log::info!("mqtt: connected to {redacted}, subscribed to zigbee2mqtt/#");

    tokio::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(event) => {
                    if let rumqttc::Event::Incoming(Packet::Publish(publish)) = event {
                        let topic = publish.topic.clone();
                        let payload = publish.payload.clone();

                        // Update stats
                        {
                            let mut status = state.mqtt_status.write().await;
                            status.topics_received += 1;
                            status.last_message_at = Some(Utc::now().to_rfc3339());
                        }

                        // Dispatch by topic
                        if topic == "zigbee2mqtt/bridge/devices" {
                            handle_bridge_devices(&payload, &state).await;
                        } else if let Some(name) = topic.strip_prefix("zigbee2mqtt/") {
                            // Skip other bridge/* subtopics
                            if !name.starts_with("bridge/") {
                                handle_state_message(name, &payload, &state).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("mqtt: connection error: {e}");
                    // rumqttc will reconnect automatically; brief pause before retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    });
}

// ── State message handler ─────────────────────────────────────────────────────

async fn handle_state_message(device_name: &str, payload: &[u8], state: &AppState) {
    let json: serde_json::Value = match serde_json::from_slice(payload) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut home = state.home.write().await;
    let device = match home.get_device_mut(device_name) {
        Some(d) => d,
        None => return, // unknown device — silently ignore
    };

    if let Some(s) = json.get("state").and_then(|v| v.as_str()) {
        device.state = if s.eq_ignore_ascii_case("on") { DeviceState::On } else { DeviceState::Off };
    }
    if let Some(b) = json.get("brightness").and_then(|v| v.as_u64()) {
        device.brightness = b.min(100) as u8;
    }
    if let Some(t) = json.get("temperature").and_then(|v| v.as_f64()) {
        device.temperature = Some(t);
    }
    // Store linkquality in attributes
    if let Some(lq) = json.get("linkquality")
        && let serde_json::Value::Object(ref mut map) = device.attributes {
            map.insert("linkquality".to_string(), lq.clone());
        }
}

// ── Bridge devices handler ────────────────────────────────────────────────────

async fn handle_bridge_devices(payload: &[u8], state: &AppState) {
    let arr: Vec<serde_json::Value> = match serde_json::from_slice(payload) {
        Ok(serde_json::Value::Array(a)) => a,
        _ => return,
    };

    let home = state.home.read().await;
    let mut discovery = state.discovery.write().unwrap();

    for entry in &arr {
        let friendly_name = match entry.get("friendly_name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        // Skip if already in the home
        if home.get_device(friendly_name).is_some() {
            continue;
        }
        // Skip if already discovered
        if discovery.contains_key(friendly_name) {
            continue;
        }
        let role = entry.get("type")
            .and_then(|v| v.as_str())
            .and_then(ZigbeeRole::from_z2m_type);

        let discovered = DiscoveredDevice {
            id: friendly_name.to_string(),
            name: friendly_name.to_string(),
            service_type: "_zigbee._tcp".to_string(),
            host: String::new(),
            addresses: Vec::new(),
            port: 0,
            properties: HashMap::new(),
            suggested_type: "sensor".to_string(),
            protocol: Some(Protocol::Zigbee),
            zigbee_role: role,
        };
        discovery.insert(friendly_name.to_string(), discovered);
        log::info!("mqtt: discovered new Zigbee device '{}'", friendly_name);
    }
}

// ── Command publish ───────────────────────────────────────────────────────────

/// Publish a command to `zigbee2mqtt/{name}/set` (fire-and-forget).
pub async fn publish_command(client: &AsyncClient, name: &str, patch: serde_json::Value) {
    let topic = format!("zigbee2mqtt/{}/set", name);
    let payload = match serde_json::to_vec(&patch) {
        Ok(b) => b,
        Err(e) => { log::error!("mqtt: failed to serialize command: {e}"); return; }
    };
    if let Err(e) = client.publish(topic, QoS::AtMostOnce, false, payload).await {
        log::error!("mqtt: publish_command failed for '{name}': {e}");
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_url_with_credentials() {
        assert_eq!(redact_url("mqtt://user:pass@broker.local:1883"), "mqtt://broker.local:***");
    }

    #[test]
    fn test_redact_url_no_credentials() {
        assert_eq!(redact_url("mqtt://localhost:1883"), "mqtt://localhost:***");
    }

    #[test]
    fn test_redact_url_no_port() {
        let result = redact_url("mqtt://localhost");
        assert!(result.contains("***"));
    }

    #[test]
    fn test_zigbee_role_display() {
        assert_eq!(ZigbeeRole::Coordinator.to_string(), "coordinator");
        assert_eq!(ZigbeeRole::Router.to_string(),      "router");
        assert_eq!(ZigbeeRole::EndDevice.to_string(),   "end_device");
    }

    #[test]
    fn test_zigbee_role_from_z2m() {
        assert_eq!(ZigbeeRole::from_z2m_type("Coordinator"), Some(ZigbeeRole::Coordinator));
        assert_eq!(ZigbeeRole::from_z2m_type("Router"),      Some(ZigbeeRole::Router));
        assert_eq!(ZigbeeRole::from_z2m_type("EndDevice"),   Some(ZigbeeRole::EndDevice));
        assert_eq!(ZigbeeRole::from_z2m_type("unknown_role"), None);
    }
}
