use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    automation::{Action, AutomationRule, Trigger},
    device::Device,
};
use crate::http::{
    errors::ApiError,
    types::{ActionInput, DeviceResponse, EventKind, RuleResponse, ServerEvent, TriggerInput, state_to_string},
};
use crate::state::AppState;

// ── Persistence helpers ───────────────────────────────────────────────────────

/// Persist the current snapshot of a device to the database (no-op when DB is absent).
pub async fn persist_device(state: &AppState, device: &Device) {
    if let Some(pool) = &state.db {
        if let Err(e) = crate::infrastructure::db::upsert_device(pool, device).await {
            log::error!("DB persist failed for '{}': {e}", device.name);
        }
    }
}

/// Delete a device from the database by its UUID (no-op when DB is absent).
pub async fn delete_from_db(state: &AppState, device_id: &str) {
    if let Some(pool) = &state.db {
        if let Err(e) = crate::infrastructure::db::delete_device(pool, device_id).await {
            log::error!("DB delete failed for '{device_id}': {e}");
        }
    }
}

// ── Event recording ───────────────────────────────────────────────────────────

pub async fn record_event(
    state: &AppState,
    kind: EventKind,
    entity: &str,
    message: String,
    device_name: Option<String>,
    client_id: Option<String>,
) {
    let event = ServerEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        kind,
        entity: entity.to_string(),
        message,
        device_name,
        client_id,
    };
    state.events.write().await.push(event);
}

// ── Error mappers ─────────────────────────────────────────────────────────────

pub fn map_create_error(err: String) -> ApiError {
    if err.to_lowercase().contains("already exists") {
        return ApiError::Conflict(err);
    }
    map_common_error(err)
}

pub fn map_common_error(err: String) -> ApiError {
    if err.to_lowercase().contains("not found") {
        ApiError::NotFound(err)
    } else {
        ApiError::BadRequest(err)
    }
}

// ── Response mapping ──────────────────────────────────────────────────────────

pub fn device_to_response(device: &Device) -> DeviceResponse {
    DeviceResponse {
        id: device.id.clone(),
        name: device.name.clone(),
        device_type: format!("{}", device.device_type).to_ascii_lowercase(),
        state: state_to_string(&device.state).to_string(),
        room: device.room.clone(),
        connected: device.connected,
        last_error: device.last_error.clone(),
        brightness: device.brightness,
        temperature: device.temperature,
    }
}

pub fn rule_to_response(rule: &AutomationRule) -> RuleResponse {
    RuleResponse {
        name: rule.name.clone(),
        enabled: rule.enabled,
        trigger: trigger_to_response(&rule.trigger),
        action: action_to_response(&rule.action),
    }
}

pub fn trigger_to_response(trigger: &Trigger) -> TriggerInput {
    match trigger {
        Trigger::DeviceStateChange { device_name, target_state } =>
            TriggerInput::DeviceStateChange { device_name: device_name.clone(), target_state: state_to_string(target_state).to_string() },
        Trigger::TemperatureAbove { device_name, threshold } =>
            TriggerInput::TemperatureAbove { device_name: device_name.clone(), threshold: *threshold },
        Trigger::TemperatureBelow { device_name, threshold } =>
            TriggerInput::TemperatureBelow { device_name: device_name.clone(), threshold: *threshold },
    }
}

pub fn action_to_response(action: &Action) -> ActionInput {
    match action {
        Action::DeviceState { device_name, state } =>
            ActionInput::DeviceState { device_name: device_name.clone(), state: state_to_string(state).to_string() },
        Action::Brightness { device_name, brightness } =>
            ActionInput::Brightness { device_name: device_name.clone(), brightness: *brightness },
        Action::Temperature { device_name, temperature } =>
            ActionInput::Temperature { device_name: device_name.clone(), temperature: *temperature },
    }
}
