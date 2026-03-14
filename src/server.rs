use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use chrono::Utc;
use log::info;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock, oneshot};
use uuid::Uuid;

use crate::{
    automation::{Action, AutomationEngine, Trigger},
    manager::SmartHome,
    models::{Device, DeviceState, DeviceType},
};

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

#[derive(Debug, Error)]
pub enum ServerStartError {
    #[error("invalid bind address '{0}'")]
    InvalidBindAddress(String),
    #[error("failed to start server: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct AppState {
    home: Arc<RwLock<SmartHome>>,
    automation: Arc<RwLock<AutomationEngine>>,
    events: Arc<RwLock<Vec<ServerEvent>>>,
    clients: Arc<RwLock<HashMap<String, ClientSession>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl AppState {
    fn new(shutdown_tx: Option<oneshot::Sender<()>>) -> Self {
        Self {
            home: Arc::new(RwLock::new(SmartHome::new())),
            automation: Arc::new(RwLock::new(AutomationEngine::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(shutdown_tx)),
        }
    }
}

pub async fn run_server(addr: Option<String>) -> Result<(), ServerStartError> {
    let bind_addr = addr
        .or_else(|| std::env::var("SMART_HOME_SERVER_ADDR").ok())
        .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());

    let socket_addr: SocketAddr = bind_addr
        .parse()
        .map_err(|_| ServerStartError::InvalidBindAddress(bind_addr.clone()))?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = AppState::new(Some(shutdown_tx));
    let app = router(state.clone());

    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    info!("smart_home_server listening on http://{}", socket_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await?;

    info!("smart_home_server stopped");
    Ok(())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/server/stop", post(stop_server))
        .route("/events", get(list_events))
        .route("/clients", get(list_clients))
        .route("/clients/connect", post(connect_client))
        .route("/clients/{client_id}/disconnect", post(disconnect_client))
        .route("/devices", get(list_devices).post(create_device))
        .route(
            "/devices/{name}",
            get(get_device).delete(remove_device).patch(update_device),
        )
        .route("/devices/{name}/state", patch(set_device_state))
        .route("/devices/{name}/brightness", patch(set_device_brightness))
        .route("/devices/{name}/temperature", patch(set_device_temperature))
        .route("/devices/{name}/commands", post(send_device_command))
        .route("/devices/{name}/connect", post(connect_device))
        .route("/devices/{name}/disconnect", post(disconnect_device))
        .route("/devices/{name}/error", post(report_device_error))
        .route("/devices/{name}/error/clear", post(clear_device_error))
        .route("/devices/{name}/events", get(list_device_events))
        .route("/automation/rules", get(list_rules).post(add_rule))
        .route("/automation/rules/{name}", delete(remove_rule))
        .route("/automation/rules/{name}/toggle", post(toggle_rule))
        .route("/automation/run", post(run_automation))
        .with_state(state)
}

async fn shutdown_signal(mut rx: oneshot::Receiver<()>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl-c shutdown signal");
        }
        _ = &mut rx => {
            info!("received API shutdown signal");
        }
    }
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    devices: usize,
    connected_devices: usize,
    rooms: usize,
    rules: usize,
    clients: usize,
    connected_clients: usize,
    events: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ClientSession {
    client_id: String,
    name: String,
    connected: bool,
    connected_at: String,
    disconnected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventKind {
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
struct ServerEvent {
    event_id: String,
    timestamp: String,
    kind: EventKind,
    entity: String,
    message: String,
    device_name: Option<String>,
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CreateClientRequest {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateDeviceRequest {
    name: String,
    device_type: String,
}

#[derive(Debug, Deserialize)]
struct StateUpdateRequest {
    state: String,
}

#[derive(Debug, Deserialize)]
struct BrightnessUpdateRequest {
    brightness: u8,
}

#[derive(Debug, Deserialize)]
struct TemperatureUpdateRequest {
    temperature: f64,
}

#[derive(Debug, Deserialize)]
struct DeviceCommandRequest {
    command: String,
    state: Option<String>,
    brightness: Option<u8>,
    temperature: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DeviceErrorRequest {
    message: String,
}

#[derive(Debug, Deserialize)]
struct UpdateDeviceRequest {
    state: Option<String>,
    brightness: Option<u8>,
    temperature: Option<f64>,
    connected: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DeviceResponse {
    id: String,
    name: String,
    device_type: String,
    state: String,
    room: Option<String>,
    connected: bool,
    last_error: Option<String>,
    brightness: u8,
    temperature: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AddRuleRequest {
    name: String,
    trigger: TriggerInput,
    action: ActionInput,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TriggerInput {
    DeviceStateChange {
        device_name: String,
        target_state: String,
    },
    TemperatureAbove {
        device_name: String,
        threshold: f64,
    },
    TemperatureBelow {
        device_name: String,
        threshold: f64,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ActionInput {
    DeviceState {
        device_name: String,
        state: String,
    },
    Brightness {
        device_name: String,
        brightness: u8,
    },
    Temperature {
        device_name: String,
        temperature: f64,
    },
}

#[derive(Debug, Serialize)]
struct RuleResponse {
    name: String,
    enabled: bool,
    trigger: TriggerInput,
    action: ActionInput,
}

#[derive(Debug, Serialize)]
struct AutomationRunResponse {
    actions_executed: usize,
    actions: Vec<ActionInput>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let home = state.home.read().await;
    let rules = state.automation.read().await;
    let clients = state.clients.read().await;
    let events = state.events.read().await;

    let connected_devices = home
        .devices
        .values()
        .filter(|device| device.connected)
        .count();
    let connected_clients = clients.values().filter(|client| client.connected).count();

    Json(StatusResponse {
        devices: home.devices.len(),
        connected_devices,
        rooms: home.rooms.len(),
        rules: rules.rules.len(),
        clients: clients.len(),
        connected_clients,
        events: events.len(),
    })
}

async fn stop_server(State(state): State<AppState>) -> Result<Json<MessageResponse>, ApiError> {
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        tx.send(())
            .map_err(|_| ApiError::Internal("failed to stop server".to_string()))?;

        record_event(
            &state,
            EventKind::Server,
            "server",
            "shutdown requested by API".to_string(),
            None,
            None,
        )
        .await;

        return Ok(Json(MessageResponse {
            message: "shutdown initiated".to_string(),
        }));
    }

    Err(ApiError::BadRequest(
        "shutdown signal unavailable".to_string(),
    ))
}

async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> Json<Vec<ServerEvent>> {
    let events = state.events.read().await;
    let mut list = events.clone();
    if let Some(limit) = query.limit {
        if list.len() > limit {
            list = list[list.len() - limit..].to_vec();
        }
    }
    Json(list)
}

async fn connect_client(
    State(state): State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<Json<ClientSession>, ApiError> {
    let client_id = Uuid::new_v4().to_string();
    let name = payload
        .name
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "anonymous".to_string());

    let session = ClientSession {
        client_id: client_id.clone(),
        name,
        connected: true,
        connected_at: Utc::now().to_rfc3339(),
        disconnected_at: None,
    };

    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id.clone(), session.clone());
    }

    record_event(
        &state,
        EventKind::ClientConnected,
        "client",
        format!("client '{}' connected", session.client_id),
        None,
        Some(session.client_id.clone()),
    )
    .await;

    Ok(Json(session))
}

async fn disconnect_client(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
) -> Result<Json<ClientSession>, ApiError> {
    let updated = {
        let mut clients = state.clients.write().await;
        let session = clients
            .get_mut(&client_id)
            .ok_or_else(|| ApiError::NotFound(format!("client '{}' not found", client_id)))?;

        session.connected = false;
        session.disconnected_at = Some(Utc::now().to_rfc3339());
        session.clone()
    };

    record_event(
        &state,
        EventKind::ClientDisconnected,
        "client",
        format!("client '{}' disconnected", client_id),
        None,
        Some(client_id),
    )
    .await;

    Ok(Json(updated))
}

async fn list_clients(State(state): State<AppState>) -> Json<Vec<ClientSession>> {
    let clients = state.clients.read().await;
    let mut list = clients.values().cloned().collect::<Vec<_>>();
    list.sort_by(|a, b| a.client_id.cmp(&b.client_id));
    Json(list)
}

async fn create_device(
    State(state): State<AppState>,
    Json(payload): Json<CreateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device_type = DeviceType::from_str_loose(&payload.device_type).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid device_type '{}'; use light|thermostat|lock|switch|sensor",
            payload.device_type
        ))
    })?;

    {
        let mut home = state.home.write().await;
        home.add_device(&payload.name, device_type)
            .map_err(map_create_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&payload.name)
            .cloned()
            .ok_or_else(|| ApiError::Internal("device creation failed".to_string()))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' created", payload.name),
        Some(payload.name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn remove_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.remove_device(&name).map_err(map_common_error)?;
    }

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' removed", name),
        Some(name.clone()),
        None,
    )
    .await;

    Ok(Json(MessageResponse {
        message: format!("device '{}' removed", name),
    }))
}

async fn list_devices(State(state): State<AppState>) -> Json<Vec<DeviceResponse>> {
    let home = state.home.read().await;
    let list = home
        .list_devices()
        .into_iter()
        .map(device_to_response)
        .collect();

    Json(list)
}

async fn get_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let home = state.home.read().await;
    let device = home
        .get_device(&name)
        .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?;

    Ok(Json(device_to_response(device)))
}

async fn update_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    if payload.state.is_none()
        && payload.brightness.is_none()
        && payload.temperature.is_none()
        && payload.connected.is_none()
    {
        return Err(ApiError::BadRequest(
            "no update fields provided".to_string(),
        ));
    }

    {
        let mut home = state.home.write().await;

        if let Some(state_raw) = payload.state {
            let state_value = parse_device_state(&state_raw)?;
            home.set_state(&name, state_value)
                .map_err(map_common_error)?;
        }

        if let Some(brightness) = payload.brightness {
            home.set_brightness(&name, brightness)
                .map_err(map_common_error)?;
        }

        if let Some(temperature) = payload.temperature {
            home.set_temperature(&name, temperature)
                .map_err(map_common_error)?;
        }

        if let Some(connected) = payload.connected {
            if connected {
                home.connect_device(&name).map_err(map_common_error)?;
            } else {
                home.disconnect_device(&name).map_err(map_common_error)?;
            }
        }
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' updated", name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn set_device_state(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<StateUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let state_value = parse_device_state(&payload.state)?;

    {
        let mut home = state.home.write().await;
        home.set_state(&name, state_value)
            .map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' state updated", name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn set_device_brightness(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<BrightnessUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.set_brightness(&name, payload.brightness)
            .map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' brightness updated", name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn set_device_temperature(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<TemperatureUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.set_temperature(&name, payload.temperature)
            .map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' temperature updated", name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn send_device_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeviceCommandRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        match payload.command.to_lowercase().as_str() {
            "set_state" => {
                let raw_state = payload.state.as_deref().ok_or_else(|| {
                    ApiError::BadRequest("state is required for set_state".to_string())
                })?;
                let state_value = parse_device_state(raw_state)?;
                home.set_state(&name, state_value)
                    .map_err(map_common_error)?;
            }
            "set_brightness" => {
                let brightness = payload.brightness.ok_or_else(|| {
                    ApiError::BadRequest("brightness is required for set_brightness".to_string())
                })?;
                home.set_brightness(&name, brightness)
                    .map_err(map_common_error)?;
            }
            "set_temperature" => {
                let temperature = payload.temperature.ok_or_else(|| {
                    ApiError::BadRequest("temperature is required for set_temperature".to_string())
                })?;
                home.set_temperature(&name, temperature)
                    .map_err(map_common_error)?;
            }
            other => {
                return Err(ApiError::BadRequest(format!(
                    "unsupported command '{}' (use set_state|set_brightness|set_temperature)",
                    other
                )));
            }
        }
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::Request,
        "device_command",
        format!(
            "command '{}' applied to device '{}'",
            payload.command, device.name
        ),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn connect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.connect_device(&name).map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceConnected,
        "device",
        format!("device '{}' connected", device.name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn disconnect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.disconnect_device(&name).map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceDisconnected,
        "device",
        format!("device '{}' disconnected", device.name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn report_device_error(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeviceErrorRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    if payload.message.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "error message cannot be empty".to_string(),
        ));
    }

    {
        let mut home = state.home.write().await;
        home.set_device_error(&name, payload.message.clone())
            .map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceError,
        "device",
        format!("device '{}' error: {}", name, payload.message),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn clear_device_error(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        home.clear_device_error(&name).map_err(map_common_error)?;
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };

    record_event(
        &state,
        EventKind::DeviceUpdated,
        "device",
        format!("device '{}' error cleared", name),
        Some(name),
        None,
    )
    .await;

    Ok(Json(device_to_response(&device)))
}

async fn list_device_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<Vec<ServerEvent>> {
    let events = state.events.read().await;
    let list = events
        .iter()
        .filter(|event| {
            event
                .device_name
                .as_ref()
                .map(|device_name| device_name.eq_ignore_ascii_case(&name))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    Json(list)
}

async fn add_rule(
    State(state): State<AppState>,
    Json(payload): Json<AddRuleRequest>,
) -> Result<Json<RuleResponse>, ApiError> {
    let trigger = payload.trigger.clone().to_domain()?;
    let action = payload.action.clone().to_domain()?;

    {
        let mut automation = state.automation.write().await;
        automation
            .add_rule(&payload.name, trigger, action)
            .map_err(map_create_error)?;
    }

    record_event(
        &state,
        EventKind::Automation,
        "automation_rule",
        format!("rule '{}' added", payload.name),
        None,
        None,
    )
    .await;

    Ok(Json(RuleResponse {
        name: payload.name,
        enabled: true,
        trigger: payload.trigger,
        action: payload.action,
    }))
}

async fn list_rules(State(state): State<AppState>) -> Json<Vec<RuleResponse>> {
    let automation = state.automation.read().await;
    let rules = automation
        .list_rules()
        .iter()
        .map(rule_to_response)
        .collect::<Vec<_>>();

    Json(rules)
}

async fn remove_rule(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    {
        let mut automation = state.automation.write().await;
        automation.remove_rule(&name).map_err(map_common_error)?;
    }

    record_event(
        &state,
        EventKind::Automation,
        "automation_rule",
        format!("rule '{}' removed", name),
        None,
        None,
    )
    .await;

    Ok(Json(MessageResponse {
        message: format!("rule '{}' removed", name),
    }))
}

async fn toggle_rule(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<RuleResponse>, ApiError> {
    let enabled = {
        let mut automation = state.automation.write().await;
        automation.toggle_rule(&name).map_err(map_common_error)?
    };

    let rule = {
        let automation = state.automation.read().await;
        automation
            .list_rules()
            .iter()
            .find(|rule| rule.name.eq_ignore_ascii_case(&name))
            .map(rule_to_response)
            .ok_or_else(|| ApiError::NotFound(format!("rule '{}' not found", name)))?
    };

    let status = if enabled { "enabled" } else { "disabled" };
    record_event(
        &state,
        EventKind::Automation,
        "automation_rule",
        format!("rule '{}' {}", name, status),
        None,
        None,
    )
    .await;

    Ok(Json(rule))
}

async fn run_automation(
    State(state): State<AppState>,
) -> Result<Json<AutomationRunResponse>, ApiError> {
    let actions = {
        let home = state.home.read().await;
        let automation = state.automation.read().await;
        automation.evaluate_rules(&home)
    };

    {
        let mut home = state.home.write().await;
        AutomationEngine::execute_actions(&actions, &mut home);
    }

    record_event(
        &state,
        EventKind::Automation,
        "automation_engine",
        format!("{} action(s) executed", actions.len()),
        None,
        None,
    )
    .await;

    let actions = actions.iter().map(action_to_response).collect::<Vec<_>>();

    Ok(Json(AutomationRunResponse {
        actions_executed: actions.len(),
        actions,
    }))
}

fn map_create_error(err: String) -> ApiError {
    if err.to_lowercase().contains("already exists") {
        return ApiError::Conflict(err);
    }
    map_common_error(err)
}

fn map_common_error(err: String) -> ApiError {
    if err.to_lowercase().contains("not found") {
        ApiError::NotFound(err)
    } else {
        ApiError::BadRequest(err)
    }
}

fn parse_device_state(raw: &str) -> Result<DeviceState, ApiError> {
    match raw.to_ascii_lowercase().as_str() {
        "on" => Ok(DeviceState::On),
        "off" => Ok(DeviceState::Off),
        _ => Err(ApiError::BadRequest(format!(
            "invalid state '{}' (use on|off)",
            raw
        ))),
    }
}

fn state_to_string(state: &DeviceState) -> &'static str {
    match state {
        DeviceState::On => "on",
        DeviceState::Off => "off",
    }
}

fn device_to_response(device: &Device) -> DeviceResponse {
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

fn rule_to_response(rule: &crate::automation::AutomationRule) -> RuleResponse {
    RuleResponse {
        name: rule.name.clone(),
        enabled: rule.enabled,
        trigger: trigger_to_response(&rule.trigger),
        action: action_to_response(&rule.action),
    }
}

fn trigger_to_response(trigger: &Trigger) -> TriggerInput {
    match trigger {
        Trigger::DeviceStateChange {
            device_name,
            target_state,
        } => TriggerInput::DeviceStateChange {
            device_name: device_name.clone(),
            target_state: state_to_string(target_state).to_string(),
        },
        Trigger::TemperatureAbove {
            device_name,
            threshold,
        } => TriggerInput::TemperatureAbove {
            device_name: device_name.clone(),
            threshold: *threshold,
        },
        Trigger::TemperatureBelow {
            device_name,
            threshold,
        } => TriggerInput::TemperatureBelow {
            device_name: device_name.clone(),
            threshold: *threshold,
        },
    }
}

fn action_to_response(action: &Action) -> ActionInput {
    match action {
        Action::DeviceState { device_name, state } => ActionInput::DeviceState {
            device_name: device_name.clone(),
            state: state_to_string(state).to_string(),
        },
        Action::Brightness {
            device_name,
            brightness,
        } => ActionInput::Brightness {
            device_name: device_name.clone(),
            brightness: *brightness,
        },
        Action::Temperature {
            device_name,
            temperature,
        } => ActionInput::Temperature {
            device_name: device_name.clone(),
            temperature: *temperature,
        },
    }
}

impl TriggerInput {
    fn to_domain(self) -> Result<Trigger, ApiError> {
        match self {
            TriggerInput::DeviceStateChange {
                device_name,
                target_state,
            } => Ok(Trigger::DeviceStateChange {
                device_name,
                target_state: parse_device_state(&target_state)?,
            }),
            TriggerInput::TemperatureAbove {
                device_name,
                threshold,
            } => Ok(Trigger::TemperatureAbove {
                device_name,
                threshold,
            }),
            TriggerInput::TemperatureBelow {
                device_name,
                threshold,
            } => Ok(Trigger::TemperatureBelow {
                device_name,
                threshold,
            }),
        }
    }
}

impl ActionInput {
    fn to_domain(self) -> Result<Action, ApiError> {
        match self {
            ActionInput::DeviceState { device_name, state } => Ok(Action::DeviceState {
                device_name,
                state: parse_device_state(&state)?,
            }),
            ActionInput::Brightness {
                device_name,
                brightness,
            } => Ok(Action::Brightness {
                device_name,
                brightness,
            }),
            ActionInput::Temperature {
                device_name,
                temperature,
            } => Ok(Action::Temperature {
                device_name,
                temperature,
            }),
        }
    }
}

async fn record_event(
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
    events.push(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use serde_json::json;
    use tower::ServiceExt;

    fn app() -> Router {
        router(AppState::new(None))
    }

    async fn request(
        app: &Router,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut builder = Request::builder().uri(uri).method(method);
        let request_body = match body {
            Some(payload) => {
                builder = builder.header("content-type", "application/json");
                Body::from(payload.to_string())
            }
            None => Body::empty(),
        };

        let response = app
            .clone()
            .oneshot(builder.body(request_body).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json = if body.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body).unwrap()
        };

        (status, json)
    }

    #[tokio::test]
    async fn health_endpoint_works() {
        let app = app();
        let (status, body) = request(&app, "GET", "/health", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn create_and_fetch_device() {
        let app = app();

        let (status, _) = request(
            &app,
            "POST",
            "/devices",
            Some(json!({"name":"kitchen_light","device_type":"light"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, json) = request(&app, "GET", "/devices/kitchen_light", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["name"], "kitchen_light");
        assert_eq!(json["device_type"], "light");
    }

    #[tokio::test]
    async fn automation_run_executes_actions() {
        let app = app();

        let create_device = |name: &str, device_type: &str| {
            Request::builder()
                .uri("/devices")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"name":"{}","device_type":"{}"}}"#,
                    name, device_type
                )))
                .unwrap()
        };

        let response = app
            .clone()
            .oneshot(create_device("thermo", "thermostat"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/devices/thermo/temperature")
                    .method("PATCH")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"temperature":30.0}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/automation/rules")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"cool_down","trigger":{"type":"temperature_above","device_name":"thermo","threshold":25.0},"action":{"type":"temperature","device_name":"thermo","temperature":22.0}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/automation/run")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices/thermo")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["temperature"], 22.0);
    }

    #[tokio::test]
    async fn status_clients_and_events_flow() {
        let app = app();

        let (status, body) = request(&app, "GET", "/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["devices"], 0);
        assert_eq!(body["clients"], 0);

        let (status, body) = request(
            &app,
            "POST",
            "/clients/connect",
            Some(json!({"name":"mobile-app"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let client_id = body["client_id"].as_str().unwrap().to_string();

        let (status, clients) = request(&app, "GET", "/clients", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(clients.as_array().unwrap().len(), 1);
        assert_eq!(clients[0]["connected"], true);

        let uri = format!("/clients/{}/disconnect", client_id);
        let (status, body) = request(&app, "POST", &uri, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["connected"], false);
        assert!(body["disconnected_at"].is_string());

        let (status, _) = request(&app, "POST", "/clients/unknown/disconnect", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, events) = request(&app, "GET", "/events?limit=1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(events.as_array().unwrap().len(), 1);

        let (status, body) = request(&app, "GET", "/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["clients"], 1);
        assert_eq!(body["connected_clients"], 0);
        assert!(body["events"].as_u64().unwrap() >= 2);
    }

    #[tokio::test]
    async fn stop_server_endpoint_behaviors() {
        let app_without_shutdown = router(AppState::new(None));
        let (status, _) = request(&app_without_shutdown, "POST", "/server/stop", None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (tx, rx) = oneshot::channel();
        let app = router(AppState::new(Some(tx)));
        let (status, body) = request(&app, "POST", "/server/stop", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["message"], "shutdown initiated");

        assert!(rx.await.is_ok());

        let (status, _) = request(&app, "POST", "/server/stop", None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn device_lifecycle_and_update_error_paths() {
        let app = app();

        let (status, _) = request(
            &app,
            "POST",
            "/devices",
            Some(json!({"name":"lamp","device_type":"light"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, error) = request(
            &app,
            "POST",
            "/devices",
            Some(json!({"name":"lamp","device_type":"light"})),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(error["code"], "conflict");

        let (status, _) = request(
            &app,
            "PATCH",
            "/devices/lamp",
            Some(json!({"state":"on","brightness":80,"connected":true})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, error) = request(&app, "PATCH", "/devices/lamp", Some(json!({}))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["code"], "bad_request");

        let (status, _) = request(
            &app,
            "PATCH",
            "/devices/lamp/state",
            Some(json!({"state":"invalid"})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(
            &app,
            "PATCH",
            "/devices/unknown/state",
            Some(json!({"state":"on"})),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(
            &app,
            "PATCH",
            "/devices/lamp/temperature",
            Some(json!({"temperature":24.0})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(&app, "POST", "/devices/lamp/connect", None).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = request(&app, "POST", "/devices/lamp/disconnect", None).await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/lamp/error",
            Some(json!({"message":"   "})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, body) = request(
            &app,
            "POST",
            "/devices/lamp/error",
            Some(json!({"message":"offline"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["last_error"], "offline");

        let (status, body) = request(&app, "POST", "/devices/lamp/error/clear", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["last_error"].is_null());

        let (status, _) = request(&app, "GET", "/devices/lamp/events", None).await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = request(&app, "DELETE", "/devices/unknown", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/invalid/commands",
            Some(json!({"command":"set_state","state":"on"})),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn device_commands_and_type_constraints() {
        let app = app();

        let (status, _) = request(
            &app,
            "POST",
            "/devices",
            Some(json!({"name":"thermo","device_type":"thermostat"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/thermo/commands",
            Some(json!({"command":"set_state"})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/thermo/commands",
            Some(json!({"command":"set_brightness","brightness":60})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/thermo/commands",
            Some(json!({"command":"set_temperature"})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, body) = request(
            &app,
            "POST",
            "/devices/thermo/commands",
            Some(json!({"command":"set_temperature","temperature":19.5})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["temperature"], 19.5);

        let (status, _) = request(
            &app,
            "POST",
            "/devices/thermo/commands",
            Some(json!({"command":"unknown"})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(
            &app,
            "PATCH",
            "/devices/thermo/brightness",
            Some(json!({"brightness":20})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rule_management_and_validation_paths() {
        let app = app();

        let (status, _) = request(
            &app,
            "POST",
            "/automation/rules",
            Some(json!({
                "name":"bad-trigger",
                "trigger":{"type":"device_state_change","device_name":"sensor","target_state":"bad"},
                "action":{"type":"device_state","device_name":"lamp","state":"on"}
            })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let (status, _) = request(
            &app,
            "POST",
            "/automation/rules",
            Some(json!({
                "name":"bad-action",
                "trigger":{"type":"temperature_above","device_name":"thermo","threshold":25.0},
                "action":{"type":"device_state","device_name":"lamp","state":"bad"}
            })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let valid_rule = json!({
            "name":"night-mode",
            "trigger":{"type":"device_state_change","device_name":"sensor","target_state":"on"},
            "action":{"type":"device_state","device_name":"lamp","state":"off"}
        });

        let (status, _) =
            request(&app, "POST", "/automation/rules", Some(valid_rule.clone())).await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = request(&app, "POST", "/automation/rules", Some(valid_rule)).await;
        assert_eq!(status, StatusCode::CONFLICT);

        let (status, rules) = request(&app, "GET", "/automation/rules", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(rules.as_array().unwrap().len(), 1);

        let (status, _) = request(&app, "POST", "/automation/rules/missing/toggle", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, body) =
            request(&app, "POST", "/automation/rules/night-mode/toggle", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["enabled"], false);

        let (status, _) = request(&app, "DELETE", "/automation/rules/missing", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(&app, "DELETE", "/automation/rules/night-mode", None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_device_type_and_not_found_paths() {
        let app = app();

        let (status, error) = request(
            &app,
            "POST",
            "/devices",
            Some(json!({"name":"dev1","device_type":"invalid-type"})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["code"], "bad_request");

        let (status, _) = request(&app, "GET", "/devices/missing", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(&app, "POST", "/devices/missing/connect", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(&app, "POST", "/devices/missing/disconnect", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(&app, "POST", "/devices/missing/error/clear", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn run_server_returns_invalid_address_error() {
        let err = run_server(Some("invalid_addr".to_string()))
            .await
            .unwrap_err();
        match err {
            ServerStartError::InvalidBindAddress(addr) => assert_eq!(addr, "invalid_addr"),
            _ => panic!("expected invalid bind address error"),
        }
    }

    #[tokio::test]
    async fn shutdown_signal_oneshot_path_completes() {
        let (tx, rx) = oneshot::channel();
        let task = tokio::spawn(shutdown_signal(rx));
        tx.send(()).unwrap();
        task.await.unwrap();
    }

    #[test]
    fn api_error_into_response_maps_status_and_payload() {
        let response = ApiError::BadRequest("bad".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = ApiError::NotFound("missing".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = ApiError::Conflict("dup".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let response = ApiError::Internal("oops".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn conversion_helpers_cover_mapping_logic() {
        assert!(matches!(parse_device_state("on").unwrap(), DeviceState::On));
        assert!(parse_device_state("bad").is_err());
        assert_eq!(state_to_string(&DeviceState::Off), "off");

        let trigger = Trigger::TemperatureBelow {
            device_name: "thermo".to_string(),
            threshold: 12.0,
        };
        let response = trigger_to_response(&trigger);
        match response {
            TriggerInput::TemperatureBelow {
                device_name,
                threshold,
            } => {
                assert_eq!(device_name, "thermo");
                assert_eq!(threshold, 12.0);
            }
            _ => panic!("expected temperature_below"),
        }

        let action = Action::Brightness {
            device_name: "lamp".to_string(),
            brightness: 42,
        };
        let response = action_to_response(&action);
        match response {
            ActionInput::Brightness {
                device_name,
                brightness,
            } => {
                assert_eq!(device_name, "lamp");
                assert_eq!(brightness, 42);
            }
            _ => panic!("expected brightness action"),
        }

        let map = map_create_error("Rule 'x' already exists.".to_string());
        assert!(matches!(map, ApiError::Conflict(_)));
        let map = map_common_error("Device 'x' not found.".to_string());
        assert!(matches!(map, ApiError::NotFound(_)));
    }

    #[test]
    fn trigger_and_action_inputs_convert_to_domain() {
        let trigger = TriggerInput::DeviceStateChange {
            device_name: "sensor".to_string(),
            target_state: "on".to_string(),
        }
        .to_domain()
        .unwrap();

        assert!(matches!(
            trigger,
            Trigger::DeviceStateChange {
                device_name,
                target_state: DeviceState::On
            } if device_name == "sensor"
        ));

        let action = ActionInput::Temperature {
            device_name: "thermo".to_string(),
            temperature: 21.5,
        }
        .to_domain()
        .unwrap();
        assert!(matches!(
            action,
            Action::Temperature {
                device_name,
                temperature
            } if device_name == "thermo" && temperature == 21.5
        ));

        let invalid = ActionInput::DeviceState {
            device_name: "lamp".to_string(),
            state: "bad".to_string(),
        }
        .to_domain();
        assert!(invalid.is_err());
    }
}
