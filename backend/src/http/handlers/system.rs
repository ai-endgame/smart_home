use std::convert::Infallible;
use std::time::Duration;

use axum::{Json, extract::{Path, Query, State}, http::HeaderMap, response::IntoResponse};
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::Utc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use uuid::Uuid;

use crate::domain::device::{Device, DeviceState, DeviceType};
use crate::http::{
    errors::ApiError,
    helpers::{device_to_response, persist_device, record_event, rule_to_response},
    types::{
        BackupDocument, ClientSession, CreateClientRequest, EventKind, EventsQuery, HealthResponse,
        MessageResponse, StatusResponse,
    },
};
use crate::state::AppState;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let home = state.home.read().await;
    let rules = state.automation.read().await;
    let clients = state.clients.read().await;
    let events = state.events.read().await;

    let connected_devices = home.devices.values().filter(|d| d.connected).count();
    let connected_clients = clients.values().filter(|c| c.connected).count();

    Json(StatusResponse {
        devices: home.devices.len(),
        connected_devices,
        rooms: home.areas.len(),
        rules: rules.rules.len(),
        clients: clients.len(),
        connected_clients,
        events: events.len(),
    })
}

pub async fn stop_server(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MessageResponse>, ApiError> {
    match &state.admin_token {
        None => return Err(ApiError::Forbidden("endpoint disabled: no ADMIN_TOKEN configured".to_string())),
        Some(expected) => {
            let provided = headers
                .get("x-admin-token")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if provided != expected.as_str() {
                return Err(ApiError::Forbidden("invalid or missing X-Admin-Token".to_string()));
            }
        }
    }
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        record_event(&state, EventKind::Server, "server", "shutdown requested by API".to_string(), None, None).await;
        tx.send(()).map_err(|_| ApiError::Internal("failed to stop server".to_string()))?;
        return Ok(Json(MessageResponse { message: "shutdown initiated".to_string() }));
    }
    Err(ApiError::Conflict("shutdown signal unavailable".to_string()))
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> Json<Vec<crate::http::types::ServerEvent>> {
    let events = state.events.read().await;
    let list: Vec<crate::http::types::ServerEvent> = if let Some(limit) = query.limit {
        events.iter().rev().take(limit).cloned().collect::<Vec<_>>().into_iter().rev().collect()
    } else {
        events.iter().cloned().collect()
    };
    Json(list)
}

pub async fn connect_client(
    State(state): State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<Json<ClientSession>, ApiError> {
    let client_id = Uuid::new_v4().to_string();
    let name = payload.name.filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "anonymous".to_string());
    let session = ClientSession {
        client_id: client_id.clone(),
        name,
        connected: true,
        connected_at: Utc::now().to_rfc3339(),
        disconnected_at: None,
    };
    state.clients.write().await.insert(client_id.clone(), session.clone());
    record_event(&state, EventKind::ClientConnected, "client", format!("client '{}' connected", client_id), None, Some(client_id)).await;
    Ok(Json(session))
}

pub async fn disconnect_client(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
) -> Result<Json<ClientSession>, ApiError> {
    let updated = {
        let mut clients = state.clients.write().await;
        let session = clients.get_mut(&client_id)
            .ok_or_else(|| ApiError::NotFound(format!("client '{}' not found", client_id)))?;
        session.connected = false;
        session.disconnected_at = Some(Utc::now().to_rfc3339());
        session.clone()
    };
    record_event(&state, EventKind::ClientDisconnected, "client", format!("client '{}' disconnected", client_id), None, Some(client_id)).await;
    Ok(Json(updated))
}

pub async fn list_clients(State(state): State<AppState>) -> Json<Vec<ClientSession>> {
    let clients = state.clients.read().await;
    let mut list = clients.values().cloned().collect::<Vec<_>>();
    list.sort_by(|a, b| a.client_id.cmp(&b.client_id));
    Json(list)
}

pub async fn get_backup(State(state): State<AppState>) -> Json<BackupDocument> {
    let devices = {
        let home = state.home.read().await;
        home.devices.values().map(device_to_response).collect()
    };
    let automation_rules = {
        let automation = state.automation.read().await;
        automation.list_rules().into_iter().map(rule_to_response).collect()
    };
    let scripts = {
        let reg = state.scripts.read().await;
        reg.list().iter().map(|s| (*s).clone()).collect()
    };
    let scenes = {
        let reg = state.scenes.read().await;
        reg.list().iter().map(|s| (*s).clone()).collect()
    };
    let persons = {
        let reg = state.presence.read().await;
        reg.list().into_iter().cloned().collect()
    };
    let dashboards = {
        let reg = state.dashboard.read().await;
        reg.list().into_iter().cloned().collect()
    };
    Json(BackupDocument {
        version: "1".to_string(),
        exported_at: Utc::now(),
        devices,
        automation_rules,
        scripts,
        scenes,
        persons,
        dashboards,
    })
}

pub async fn restore_backup(
    State(state): State<AppState>,
    Json(backup): Json<BackupDocument>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let device_count = backup.devices.len();
    let rule_count = backup.automation_rules.len();
    let script_count = backup.scripts.len();
    let scene_count = backup.scenes.len();
    let person_count = backup.persons.len();
    let dashboard_count = backup.dashboards.len();

    // Acquire all write locks in canonical order
    {
        let mut home = state.home.write().await;
        let mut automation = state.automation.write().await;
        let mut scripts = state.scripts.write().await;
        let mut scenes = state.scenes.write().await;
        let mut presence = state.presence.write().await;
        let mut dashboard = state.dashboard.write().await;

        // Clear
        *home = crate::domain::manager::SmartHome::new();
        *automation = crate::domain::AutomationEngine::new();
        *scripts = crate::domain::script::ScriptRegistry::new();
        *scenes = crate::domain::scene::SceneRegistry::new();
        *presence = crate::domain::presence::PresenceRegistry::new();
        *dashboard = crate::domain::dashboard::DashboardRegistry::new();

        // Restore devices
        for d in &backup.devices {
            let device_type = DeviceType::from_str_loose(&d.device_type)
                .unwrap_or(DeviceType::Sensor);
            let state_val = match d.state.to_ascii_lowercase().as_str() {
                "on" => DeviceState::On,
                "off" => DeviceState::Off,
                _ => DeviceState::Unknown,
            };
            let mut device = Device::new(&d.name, device_type);
            device.id = d.id.clone();
            device.state = state_val;
            device.brightness = d.brightness;
            device.temperature = d.temperature;
            device.connected = d.connected;
            device.room = d.room.clone();
            device.last_error = d.last_error.clone();
            home.insert_device(device);
        }

        // Restore automation rules
        for rule in &backup.automation_rules {
            let trigger = match rule.trigger.clone().to_domain() { Ok(t) => t, Err(_) => continue };
            let action = match rule.action.clone().to_domain() { Ok(a) => a, Err(_) => continue };
            let time_range = rule.time_range.as_ref().map(|tr| (tr.from.clone(), tr.to.clone()));
            let conditions = rule.conditions.iter()
                .filter_map(|c| c.clone().to_domain().ok())
                .collect();
            let _ = automation.add_rule(&rule.name, trigger, action, time_range, conditions);
        }

        // Restore scripts
        for s in backup.scripts {
            let _ = scripts.add(s);
        }

        // Restore scenes
        for s in backup.scenes {
            let _ = scenes.add(s);
        }

        // Restore persons
        for p in backup.persons {
            let _ = presence.add(p);
        }

        // Restore dashboards
        for d in backup.dashboards {
            let _ = dashboard.add(d);
        }
    }

    // Persist restored devices to DB if available
    if state.db.is_some() {
        let home = state.home.read().await;
        for device in home.devices.values() {
            persist_device(&state, device).await;
        }
    }

    record_event(
        &state,
        EventKind::Server,
        "server",
        format!(
            "restore: {} devices, {} rules, {} scripts, {} scenes, {} persons, {} dashboards",
            device_count, rule_count, script_count, scene_count, person_count, dashboard_count
        ),
        None,
        None,
    ).await;

    Ok(Json(serde_json::json!({
        "restored": {
            "devices": device_count,
            "rules": rule_count,
            "scripts": script_count,
            "scenes": scene_count,
            "persons": person_count,
            "dashboards": dashboard_count,
        }
    })))
}

pub async fn event_stream(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.events_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        result.ok().map(|ev| {
            let data = serde_json::to_string(&ev).unwrap_or_default();
            Ok::<Event, Infallible>(Event::default().data(data).retry(Duration::from_secs(3)))
        })
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
