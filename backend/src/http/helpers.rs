use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    automation::{Action, AutomationRule, Trigger},
    dashboard::Dashboard,
    device::{Device, HistoryEntry},
    error::DomainError,
};
use crate::domain::automation::Condition;
use crate::http::{
    errors::ApiError,
    types::{ActionInput, ConditionInput, DeviceResponse, EventKind, RuleResponse, ServerEvent, TriggerInput, state_to_string},
};
use crate::state::AppState;

// ── Input validation ──────────────────────────────────────────────────────────

pub const MAX_NAME_LEN: usize = 120;

pub fn validate_name(s: &str) -> Result<(), ApiError> {
    if s.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::BadRequest(format!(
            "name exceeds maximum length of {MAX_NAME_LEN} characters"
        )));
    }
    Ok(())
}

// ── Persistence helpers ───────────────────────────────────────────────────────

/// Persist the current snapshot of a device to the database (no-op when DB is absent).
pub async fn persist_device(state: &AppState, device: &Device) {
    if let Some(pool) = &state.db {
        // Look up the area the device belongs to (if any) to persist floor/icon.
        let home = state.home.read().await;
        let area = device.room.as_deref()
            .and_then(|room_name| home.areas.get(&room_name.to_lowercase()));
        if let Err(e) = crate::infrastructure::db::upsert_device(pool, device, area).await {
            log::error!("DB persist failed for '{}': {e}", device.name);
        }
    }
}

/// Delete a device from the database by its UUID (no-op when DB is absent).
pub async fn delete_from_db(state: &AppState, device_id: &str) {
    if let Some(pool) = &state.db
        && let Err(e) = crate::infrastructure::db::delete_device(pool, device_id).await {
            log::error!("DB delete failed for '{device_id}': {e}");
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
    let mut events = state.events.write().await;
    if events.len() >= crate::state::MAX_EVENTS {
        events.pop_front();
    }
    events.push_back(event.clone());
    let _ = state.events_tx.send(event);
}

pub async fn record_history(state: &AppState, device_name: &str, entry: HistoryEntry) {
    let key = device_name.to_lowercase();
    let mut history = state.history.write().await;
    let buf = history.entry(key).or_insert_with(std::collections::VecDeque::new);
    if buf.len() >= crate::state::MAX_HISTORY_PER_DEVICE {
        buf.pop_front();
    }
    buf.push_back(entry);
}

// ── Error mappers ─────────────────────────────────────────────────────────────

pub fn map_create_error(err: DomainError) -> ApiError {
    match err {
        DomainError::AlreadyExists(msg) => ApiError::Conflict(msg),
        DomainError::NotFound(msg) => ApiError::NotFound(msg),
        DomainError::InvalidOperation(msg) => ApiError::BadRequest(msg),
    }
}

pub fn map_common_error(err: DomainError) -> ApiError {
    match err {
        DomainError::NotFound(msg) => ApiError::NotFound(msg),
        DomainError::AlreadyExists(msg) => ApiError::Conflict(msg),
        DomainError::InvalidOperation(msg) => ApiError::BadRequest(msg),
    }
}

// ── Response mapping ──────────────────────────────────────────────────────────

pub fn device_to_response(device: &Device) -> DeviceResponse {
    DeviceResponse {
        id: device.id.clone(),
        name: device.name.clone(),
        device_type: device.device_type.to_string(),
        state: state_to_string(&device.state).to_string(),
        room: device.room.clone(),
        connected: device.connected,
        last_error: device.last_error.clone(),
        brightness: device.brightness,
        temperature: device.temperature,
        control_protocol: device.control_protocol.as_ref().map(|p| p.to_string()),
        zigbee_role: device.zigbee_role.clone(),
        thread_role: device.thread_role.clone(),
        matter_fabric: device.matter_fabric.clone(),
        power_w: device.power_w,
        energy_kwh: device.energy_kwh,
    }
}

pub fn rule_to_response(rule: &AutomationRule) -> RuleResponse {
    use crate::http::types::TimeRange;
    RuleResponse {
        name: rule.name.clone(),
        enabled: rule.enabled,
        safe_mode: rule.safe_mode,
        trigger: trigger_to_response(&rule.trigger),
        action: action_to_response(&rule.action),
        time_range: rule.time_range.as_ref().map(|(f, t)| TimeRange { from: f.clone(), to: t.clone() }),
        conditions: rule.conditions.iter().map(condition_to_response).collect(),
        notify_url: rule.notify_url.clone(),
    }
}

pub fn condition_to_response(condition: &Condition) -> ConditionInput {
    match condition {
        Condition::StateEquals { device_name, state } =>
            ConditionInput::StateEquals { device_name: device_name.clone(), state: state_to_string(state).to_string() },
        Condition::BrightnessAbove { device_name, value } =>
            ConditionInput::BrightnessAbove { device_name: device_name.clone(), value: *value },
        Condition::BrightnessBelow { device_name, value } =>
            ConditionInput::BrightnessBelow { device_name: device_name.clone(), value: *value },
        Condition::TemplateEval { expr } =>
            ConditionInput::TemplateEval { expr: expr.clone() },
    }
}

pub fn trigger_to_response(trigger: &Trigger) -> TriggerInput {
    use crate::domain::automation::{NumericAttr, SunEvent};
    match trigger {
        Trigger::DeviceStateChange { device_name, target_state } =>
            TriggerInput::DeviceStateChange { device_name: device_name.clone(), target_state: state_to_string(target_state).to_string() },
        Trigger::TemperatureAbove { device_name, threshold } =>
            TriggerInput::TemperatureAbove { device_name: device_name.clone(), threshold: *threshold },
        Trigger::TemperatureBelow { device_name, threshold } =>
            TriggerInput::TemperatureBelow { device_name: device_name.clone(), threshold: *threshold },
        Trigger::Time { time } =>
            TriggerInput::Time { time: time.clone() },
        Trigger::Sun { event, offset_minutes } =>
            TriggerInput::Sun {
                event: match event { SunEvent::Sunrise => "sunrise".into(), SunEvent::Sunset => "sunset".into() },
                offset_minutes: Some(*offset_minutes),
            },
        Trigger::NumericStateAbove { device_name, attribute, threshold } =>
            TriggerInput::NumericStateAbove {
                device_name: device_name.clone(),
                attribute: match attribute { NumericAttr::Brightness => "brightness".into(), NumericAttr::Temperature => "temperature".into() },
                threshold: *threshold,
            },
        Trigger::NumericStateBelow { device_name, attribute, threshold } =>
            TriggerInput::NumericStateBelow {
                device_name: device_name.clone(),
                attribute: match attribute { NumericAttr::Brightness => "brightness".into(), NumericAttr::Temperature => "temperature".into() },
                threshold: *threshold,
            },
        Trigger::Webhook { id } =>
            TriggerInput::Webhook { id: id.clone() },
        Trigger::PresenceChange { person_name, target_state } =>
            TriggerInput::PresenceChange {
                person_name: person_name.clone(),
                target_state: target_state.to_string(),
            },
    }
}

pub fn action_to_response(action: &Action) -> ActionInput {
    match action {
        Action::DeviceState { device_name, state } =>
            ActionInput::State { device_name: device_name.clone(), state: state_to_string(state).to_string() },
        Action::Brightness { device_name, brightness } =>
            ActionInput::Brightness { device_name: device_name.clone(), brightness: *brightness },
        Action::Temperature { device_name, temperature } =>
            ActionInput::Temperature { device_name: device_name.clone(), temperature: *temperature },
        Action::Notify { message } =>
            ActionInput::Notify { message: message.clone() },
        Action::ScriptCall { script_name, args } =>
            ActionInput::ScriptCall { script_name: script_name.clone(), args: args.clone() },
    }
}

pub fn person_to_response(
    person: &crate::domain::presence::PersonTracker,
    now: chrono::DateTime<chrono::Utc>,
) -> crate::http::types::PersonResponse {
    crate::http::types::PersonResponse {
        id: person.id.clone(),
        name: person.name.clone(),
        grace_period_secs: person.grace_period_secs,
        effective_state: person.effective_state(now).to_string(),
        sources: person.sources.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
    }
}

pub fn dashboard_to_response(dashboard: &Dashboard) -> Dashboard {
    dashboard.clone()
}
