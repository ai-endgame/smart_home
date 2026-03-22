use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::device::{DeviceState, MatterFabric, ThreadRole, ZigbeeRole};
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

#[derive(Debug, Deserialize)]
pub struct DevicesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
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

impl TemperatureUpdateRequest {
    pub fn validate(&self) -> Result<(), crate::http::errors::ApiError> {
        let t = self.temperature;
        if t.is_nan() || t.is_infinite() {
            return Err(crate::http::errors::ApiError::BadRequest(
                "temperature must be a finite number".to_string(),
            ));
        }
        if !(-40.0..=100.0).contains(&t) {
            return Err(crate::http::errors::ApiError::BadRequest(
                "temperature must be between -40.0 and 100.0".to_string(),
            ));
        }
        Ok(())
    }
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

#[derive(Debug, Serialize, Deserialize)]
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
    pub control_protocol: Option<String>,
    pub zigbee_role: Option<ZigbeeRole>,
    pub thread_role: Option<ThreadRole>,
    pub matter_fabric: Option<MatterFabric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_w: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_kwh: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct EnergyUpdateRequest {
    pub power_w: Option<f64>,
    pub energy_kwh: Option<f64>,
}

// ── Commission request / response types ───────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CommissionRequest {
    pub setup_code: String,
    pub node_id: u64,
}

#[derive(Debug, Serialize)]
pub struct CommissionJobResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
    pub device_id: Option<String>,
    pub error: Option<String>,
}

impl From<&crate::infrastructure::matter::CommissionJob> for CommissionJobResponse {
    fn from(j: &crate::infrastructure::matter::CommissionJob) -> Self {
        use crate::infrastructure::matter::CommissionStatus;
        Self {
            job_id: j.job_id.clone(),
            status: match j.status {
                CommissionStatus::Pending    => "pending",
                CommissionStatus::InProgress => "in_progress",
                CommissionStatus::Done       => "done",
                CommissionStatus::Failed     => "failed",
            }.to_string(),
            message: j.message.clone(),
            device_id: j.device_id.clone(),
            error: j.error.clone(),
        }
    }
}

// ── Matter response types ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MatterStatusResponse {
    pub devices_seen: u64,
    pub commissioning_count: u64,
    pub last_seen_at: Option<String>,
    pub sync_enabled: bool,
    pub last_sync_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MatterDeviceResponse {
    pub id: String,
    pub name: String,
    pub host: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub discriminator: Option<u16>,
    pub commissioning_mode: u8,
    pub thread_role: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Serialize)]
pub struct FabricResponse {
    pub fabric_id: String,
    pub vendor_id: u16,
    pub commissioner: String,
    pub device_count: usize,
}

// ── MQTT response types ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MqttStatusResponse {
    pub connected: bool,
    pub broker: Option<String>,
    pub topics_received: u64,
    pub last_message_at: Option<String>,
}

// ── Entity response types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EntitiesQuery {
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EntityResponse {
    pub entity_id: String,
    pub kind: String,
    pub device_id: String,
    pub name: String,
    pub state: String,
    pub unit_of_measurement: Option<String>,
    pub attributes: serde_json::Value,
}

// ── Area response types ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AreaResponse {
    pub area_id: String,
    pub name: String,
    pub floor: Option<u8>,
    pub icon: Option<String>,
    pub device_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AreaDetailResponse {
    pub area_id: String,
    pub name: String,
    pub floor: Option<u8>,
    pub icon: Option<String>,
    pub device_count: usize,
    pub devices: Vec<DeviceResponse>,
}

// ── Ecosystem / Protocol response types ───────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProtocolInfoResponse {
    pub id: String,
    pub transport: &'static str,
    pub local_only: bool,
    pub mesh: bool,
    pub description: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ProtocolEntry {
    pub id: String,
    pub transport: &'static str,
    pub local_only: bool,
    pub mesh: bool,
    pub description: &'static str,
    pub device_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EcosystemLayers {
    pub local_devices: usize,
    pub cloud_devices: usize,
}

#[derive(Debug, Serialize)]
pub struct EcosystemResponse {
    pub total_devices: usize,
    pub connected_count: usize,
    pub disconnected_count: usize,
    pub unprotocolled_devices: usize,
    pub layers: EcosystemLayers,
    pub protocols: Vec<ProtocolEntry>,
}

// ── Automation request / response types ───────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeRange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct AddRuleRequest {
    pub name: String,
    pub trigger: TriggerInput,
    pub action: ActionInput,
    pub time_range: Option<TimeRange>,
    #[serde(default)]
    pub conditions: Vec<ConditionInput>,
    #[serde(default)]
    pub notify_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerInput {
    DeviceStateChange { device_name: String, target_state: String },
    TemperatureAbove { device_name: String, threshold: f64 },
    TemperatureBelow { device_name: String, threshold: f64 },
    Time { time: String },
    Sun { event: String, offset_minutes: Option<i32> },
    NumericStateAbove { device_name: String, attribute: String, threshold: f64 },
    NumericStateBelow { device_name: String, attribute: String, threshold: f64 },
    Webhook { id: String },
    PresenceChange { person_name: String, target_state: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionInput {
    State { device_name: String, state: String },
    Brightness { device_name: String, brightness: u8 },
    Temperature { device_name: String, temperature: f64 },
    Notify { message: String },
    ScriptCall { script_name: String, #[serde(default)] args: HashMap<String, Value> },
}

/// Automation condition — all conditions use AND semantics.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionInput {
    StateEquals    { device_name: String, state: String },
    BrightnessAbove { device_name: String, value: f64 },
    BrightnessBelow { device_name: String, value: f64 },
    TemplateEval   { expr: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleResponse {
    pub name: String,
    pub enabled: bool,
    pub safe_mode: bool,
    pub trigger: TriggerInput,
    pub action: ActionInput,
    pub time_range: Option<TimeRange>,
    pub conditions: Vec<ConditionInput>,
    pub notify_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutomationRunResponse {
    pub actions_executed: usize,
    pub actions: Vec<ActionInput>,
}

#[derive(Debug, Serialize)]
pub struct WebhookFireResponse {
    pub rule_name: String,
    pub action_executed: bool,
    pub message: String,
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
    match state { DeviceState::On => "on", DeviceState::Off => "off", DeviceState::Unknown => "unknown" }
}

fn validate_threshold(threshold: f64) -> Result<(), ApiError> {
    if threshold.is_nan() || threshold.is_infinite() {
        return Err(ApiError::BadRequest("threshold must be a finite number".to_string()));
    }
    if !(-40.0..=100.0).contains(&threshold) {
        return Err(ApiError::BadRequest("threshold must be between -40.0 and 100.0".to_string()));
    }
    Ok(())
}

impl TriggerInput {
    pub fn to_domain(self) -> Result<crate::domain::automation::Trigger, ApiError> {
        use crate::domain::automation::{SunEvent, Trigger};
        match self {
            TriggerInput::DeviceStateChange { device_name, target_state } =>
                Ok(Trigger::DeviceStateChange { device_name, target_state: parse_device_state(&target_state)? }),
            TriggerInput::TemperatureAbove { device_name, threshold } => {
                validate_threshold(threshold)?;
                Ok(Trigger::TemperatureAbove { device_name, threshold })
            }
            TriggerInput::TemperatureBelow { device_name, threshold } => {
                validate_threshold(threshold)?;
                Ok(Trigger::TemperatureBelow { device_name, threshold })
            }
            TriggerInput::Time { time } => {
                chrono::NaiveTime::parse_from_str(&time, "%H:%M")
                    .map_err(|_| ApiError::BadRequest(format!("invalid time '{}' (use HH:MM)", time)))?;
                Ok(Trigger::Time { time })
            }
            TriggerInput::Sun { event, offset_minutes } => {
                let sun_event = match event.to_ascii_lowercase().as_str() {
                    "sunrise" => SunEvent::Sunrise,
                    "sunset"  => SunEvent::Sunset,
                    other => return Err(ApiError::BadRequest(format!("unknown sun event '{}' (use sunrise|sunset)", other))),
                };
                Ok(Trigger::Sun { event: sun_event, offset_minutes: offset_minutes.unwrap_or(0) })
            }
            TriggerInput::NumericStateAbove { device_name, attribute, threshold } => {
                let attr = parse_numeric_attr(&attribute)?;
                Ok(Trigger::NumericStateAbove { device_name, attribute: attr, threshold })
            }
            TriggerInput::NumericStateBelow { device_name, attribute, threshold } => {
                let attr = parse_numeric_attr(&attribute)?;
                Ok(Trigger::NumericStateBelow { device_name, attribute: attr, threshold })
            }
            TriggerInput::Webhook { id } => Ok(Trigger::Webhook { id }),
            TriggerInput::PresenceChange { person_name, target_state } => {
                use crate::domain::presence::PresenceState;
                let state = match target_state.to_ascii_lowercase().as_str() {
                    "home"    => PresenceState::Home,
                    "away"    => PresenceState::Away,
                    "unknown" => PresenceState::Unknown,
                    other => return Err(ApiError::BadRequest(format!("invalid presence state '{}' (use home|away|unknown)", other))),
                };
                Ok(Trigger::PresenceChange { person_name, target_state: state })
            }
        }
    }
}

fn parse_numeric_attr(s: &str) -> Result<crate::domain::automation::NumericAttr, ApiError> {
    match s.to_ascii_lowercase().as_str() {
        "brightness"   => Ok(crate::domain::automation::NumericAttr::Brightness),
        "temperature"  => Ok(crate::domain::automation::NumericAttr::Temperature),
        other => Err(ApiError::BadRequest(format!("unknown attribute '{}' (use brightness|temperature)", other))),
    }
}

impl ActionInput {
    pub fn to_domain(self) -> Result<crate::domain::automation::Action, ApiError> {
        use crate::domain::automation::Action;
        match self {
            ActionInput::State { device_name, state } =>
                Ok(Action::DeviceState { device_name, state: parse_device_state(&state)? }),
            ActionInput::Brightness { device_name, brightness } =>
                Ok(Action::Brightness { device_name, brightness }),
            ActionInput::Temperature { device_name, temperature } => {
                let req = TemperatureUpdateRequest { temperature };
                req.validate()?;
                Ok(Action::Temperature { device_name, temperature })
            }
            ActionInput::Notify { message } => {
                if message.is_empty() {
                    return Err(ApiError::BadRequest("notify message cannot be empty".to_string()));
                }
                if message.len() > 512 {
                    return Err(ApiError::BadRequest("notify message must not exceed 512 characters".to_string()));
                }
                Ok(Action::Notify { message })
            }
            ActionInput::ScriptCall { script_name, args } =>
                Ok(Action::ScriptCall { script_name, args }),
        }
    }
}

impl ConditionInput {
    pub fn to_domain(self) -> Result<crate::domain::automation::Condition, ApiError> {
        use crate::domain::automation::Condition;
        match self {
            ConditionInput::StateEquals { device_name, state } =>
                Ok(Condition::StateEquals { device_name, state: parse_device_state(&state)? }),
            ConditionInput::BrightnessAbove { device_name, value } =>
                Ok(Condition::BrightnessAbove { device_name, value }),
            ConditionInput::BrightnessBelow { device_name, value } =>
                Ok(Condition::BrightnessBelow { device_name, value }),
            ConditionInput::TemplateEval { expr } =>
                Ok(Condition::TemplateEval { expr }),
        }
    }
}

// ── Script request / response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub params: Vec<crate::domain::script::ScriptParam>,
    pub steps: Vec<crate::domain::script::ScriptStep>,
}

#[derive(Debug, Serialize)]
pub struct ScriptResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub params: Vec<crate::domain::script::ScriptParam>,
    pub steps: Vec<crate::domain::script::ScriptStep>,
}

impl From<&crate::domain::script::Script> for ScriptResponse {
    fn from(s: &crate::domain::script::Script) -> Self {
        ScriptResponse {
            id: s.id.clone(),
            name: s.name.clone(),
            description: s.description.clone(),
            params: s.params.clone(),
            steps: s.steps.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RunScriptRequest {
    #[serde(default)]
    pub args: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct RunScriptResponse {
    pub script_id: String,
    pub status: &'static str,
}

// ── Scene request / response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateSceneRequest {
    pub name: String,
    pub states: HashMap<String, SceneStateInput>,
}

#[derive(Debug, Deserialize)]
pub struct SceneStateInput {
    pub state: Option<String>,
    pub brightness: Option<u8>,
    pub temperature: Option<f64>,
}

impl SceneStateInput {
    pub fn to_domain(self) -> Result<crate::domain::scene::SceneState, ApiError> {
        let state = match &self.state {
            Some(s) => Some(parse_device_state(s)?),
            None => None,
        };
        Ok(crate::domain::scene::SceneState { state, brightness: self.brightness, temperature: self.temperature })
    }
}

#[derive(Debug, Deserialize)]
pub struct SnapshotSceneRequest {
    pub name: String,
    pub device_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSceneRequest {
    pub states: HashMap<String, SceneStateInput>,
}

#[derive(Debug, Serialize)]
pub struct SceneResponse {
    pub id: String,
    pub name: String,
    pub states: HashMap<String, SceneStateResponse>,
}

#[derive(Debug, Serialize)]
pub struct SceneStateResponse {
    pub state: Option<String>,
    pub brightness: Option<u8>,
    pub temperature: Option<f64>,
}

impl From<&crate::domain::scene::Scene> for SceneResponse {
    fn from(s: &crate::domain::scene::Scene) -> Self {
        SceneResponse {
            id: s.id.clone(),
            name: s.name.clone(),
            states: s.states.iter().map(|(k, v)| {
                (k.clone(), SceneStateResponse {
                    state: v.state.as_ref().map(|st| state_to_string(st).to_string()),
                    brightness: v.brightness,
                    temperature: v.temperature,
                })
            }).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApplySceneResponse {
    pub applied: usize,
    pub errors: Vec<String>,
}

// ── Dashboard request / response types ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateViewRequest {
    pub title: String,
    pub icon: Option<String>,
}

/// Flat JSON request for adding a card. Uses `card_type` discriminator matching `CardContent`.
pub type CreateCardRequest = crate::domain::dashboard::CardContent;

// ── Presence request / response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub name: String,
    pub grace_period_secs: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct PersonResponse {
    pub id: String,
    pub name: String,
    pub grace_period_secs: u32,
    pub effective_state: String,
    pub sources: HashMap<String, String>,
}

// ── Backup ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupDocument {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub devices: Vec<DeviceResponse>,
    pub automation_rules: Vec<RuleResponse>,
    pub scripts: Vec<crate::domain::script::Script>,
    pub scenes: Vec<crate::domain::scene::Scene>,
    pub persons: Vec<crate::domain::presence::PersonTracker>,
    pub dashboards: Vec<crate::domain::dashboard::Dashboard>,
}

