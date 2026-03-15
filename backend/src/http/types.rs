use serde::{Deserialize, Serialize};

use crate::domain::device::DeviceState;
use crate::http::errors::ApiError;

// ── Shared response envelopes ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub devices: usize,
    pub connected_devices: usize,
    pub rooms: usize,
    pub rules: usize,
    pub clients: usize,
    pub connected_clients: usize,
    pub events: usize,
}

// ── Events & clients ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ClientSession {
    pub client_id: String,
    pub name: String,
    pub connected: bool,
    pub connected_at: String,
    pub disconnected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Request,
    DeviceConnected,
    DeviceDisconnected,
    DeviceUpdated,
    DeviceError,
    ClientConnected,
    ClientDisconnected,
    Automation,
    Server,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerEvent {
    pub event_id: String,
    pub timestamp: String,
    pub kind: EventKind,
    pub entity: String,
    pub message: String,
    pub device_name: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub limit: Option<usize>,
}

// ── Device request / response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: String,
}

#[derive(Debug, Deserialize)]
pub struct StateUpdateRequest {
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct BrightnessUpdateRequest {
    pub brightness: u8,
}

#[derive(Debug, Deserialize)]
pub struct TemperatureUpdateRequest {
    pub temperature: f64,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCommandRequest {
    pub command: String,
    pub state: Option<String>,
    pub brightness: Option<u8>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceErrorRequest {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub state: Option<String>,
    pub brightness: Option<u8>,
    pub temperature: Option<f64>,
    pub connected: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub state: String,
    pub room: Option<String>,
    pub connected: bool,
    pub last_error: Option<String>,
    pub brightness: u8,
    pub temperature: Option<f64>,
}

// ── Automation request / response types ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddRuleRequest {
    pub name: String,
    pub trigger: TriggerInput,
    pub action: ActionInput,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerInput {
    DeviceStateChange { device_name: String, target_state: String },
    TemperatureAbove { device_name: String, threshold: f64 },
    TemperatureBelow { device_name: String, threshold: f64 },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionInput {
    DeviceState { device_name: String, state: String },
    Brightness { device_name: String, brightness: u8 },
    Temperature { device_name: String, temperature: f64 },
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub name: String,
    pub enabled: bool,
    pub trigger: TriggerInput,
    pub action: ActionInput,
}

#[derive(Debug, Serialize)]
pub struct AutomationRunResponse {
    pub actions_executed: usize,
    pub actions: Vec<ActionInput>,
}

// ── Discovery ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddDiscoveredRequest {
    pub discovered_id: String,
    pub device_type: String,
    pub name: Option<String>,
}

// ── Domain conversion helpers ─────────────────────────────────────────────────

pub fn parse_device_state(raw: &str) -> Result<DeviceState, ApiError> {
    match raw.to_ascii_lowercase().as_str() {
        "on" => Ok(DeviceState::On),
        "off" => Ok(DeviceState::Off),
        _ => Err(ApiError::BadRequest(format!("invalid state '{}' (use on|off)", raw))),
    }
}

pub fn state_to_string(state: &DeviceState) -> &'static str {
    match state { DeviceState::On => "on", DeviceState::Off => "off" }
}

impl TriggerInput {
    pub fn to_domain(self) -> Result<crate::domain::automation::Trigger, ApiError> {
        use crate::domain::automation::Trigger;
        match self {
            TriggerInput::DeviceStateChange { device_name, target_state } =>
                Ok(Trigger::DeviceStateChange { device_name, target_state: parse_device_state(&target_state)? }),
            TriggerInput::TemperatureAbove { device_name, threshold } =>
                Ok(Trigger::TemperatureAbove { device_name, threshold }),
            TriggerInput::TemperatureBelow { device_name, threshold } =>
                Ok(Trigger::TemperatureBelow { device_name, threshold }),
        }
    }
}

impl ActionInput {
    pub fn to_domain(self) -> Result<crate::domain::automation::Action, ApiError> {
        use crate::domain::automation::Action;
        match self {
            ActionInput::DeviceState { device_name, state } =>
                Ok(Action::DeviceState { device_name, state: parse_device_state(&state)? }),
            ActionInput::Brightness { device_name, brightness } =>
                Ok(Action::Brightness { device_name, brightness }),
            ActionInput::Temperature { device_name, temperature } =>
                Ok(Action::Temperature { device_name, temperature }),
        }
    }
}
