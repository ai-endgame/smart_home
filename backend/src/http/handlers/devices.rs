use axum::{Json, extract::{Path, State}};

use crate::domain::device::DeviceType;
use crate::http::{
    errors::ApiError,
    helpers::{delete_from_db, device_to_response, map_common_error, map_create_error, persist_device, record_event},
    types::{
        BrightnessUpdateRequest, CreateDeviceRequest, DeviceCommandRequest, DeviceErrorRequest,
        DeviceResponse, EventKind, MessageResponse, StateUpdateRequest, TemperatureUpdateRequest,
        UpdateDeviceRequest, parse_device_state,
    },
};
use crate::state::AppState;

pub async fn list_devices(State(state): State<AppState>) -> Json<Vec<DeviceResponse>> {
    let home = state.home.read().await;
    Json(home.list_devices().into_iter().map(device_to_response).collect())
}

pub async fn get_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let home = state.home.read().await;
    let device = home.get_device(&name)
        .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?;
    Ok(Json(device_to_response(device)))
}

pub async fn create_device(
    State(state): State<AppState>,
    Json(payload): Json<CreateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device_type = DeviceType::from_str_loose(&payload.device_type).ok_or_else(|| {
        ApiError::BadRequest(format!("invalid device_type '{}'; use light|thermostat|lock|switch|sensor", payload.device_type))
    })?;
    { state.home.write().await.add_device(&payload.name, device_type).map_err(map_create_error)?; }
    let device = {
        let home = state.home.read().await;
        home.get_device(&payload.name).cloned()
            .ok_or_else(|| ApiError::Internal("device creation failed".to_string()))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' created", payload.name), Some(payload.name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn remove_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    let device_id = {
        let home = state.home.read().await;
        home.get_device(&name).map(|d| d.id.clone())
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    { state.home.write().await.remove_device(&name).map_err(map_common_error)?; }
    delete_from_db(&state, &device_id).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' removed", name), Some(name.clone()), None).await;
    Ok(Json(MessageResponse { message: format!("device '{}' removed", name) }))
}

pub async fn update_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    if payload.state.is_none() && payload.brightness.is_none() && payload.temperature.is_none() && payload.connected.is_none() {
        return Err(ApiError::BadRequest("no update fields provided".to_string()));
    }
    {
        let mut home = state.home.write().await;
        if let Some(s) = payload.state { home.set_state(&name, parse_device_state(&s)?).map_err(map_common_error)?; }
        if let Some(b) = payload.brightness { home.set_brightness(&name, b).map_err(map_common_error)?; }
        if let Some(t) = payload.temperature { home.set_temperature(&name, t).map_err(map_common_error)?; }
        if let Some(c) = payload.connected {
            if c { home.connect_device(&name).map_err(map_common_error)?; }
            else { home.disconnect_device(&name).map_err(map_common_error)?; }
        }
    }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' updated", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_state(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<StateUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let state_value = parse_device_state(&payload.state)?;
    { state.home.write().await.set_state(&name, state_value).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' state updated", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_brightness(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<BrightnessUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    { state.home.write().await.set_brightness(&name, payload.brightness).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' brightness updated", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_temperature(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<TemperatureUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    { state.home.write().await.set_temperature(&name, payload.temperature).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' temperature updated", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn send_device_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeviceCommandRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    {
        let mut home = state.home.write().await;
        match payload.command.to_lowercase().as_str() {
            "set_state" => {
                let raw = payload.state.as_deref().ok_or_else(|| ApiError::BadRequest("state is required for set_state".to_string()))?;
                home.set_state(&name, parse_device_state(raw)?).map_err(map_common_error)?;
            }
            "set_brightness" => {
                let b = payload.brightness.ok_or_else(|| ApiError::BadRequest("brightness is required for set_brightness".to_string()))?;
                home.set_brightness(&name, b).map_err(map_common_error)?;
            }
            "set_temperature" => {
                let t = payload.temperature.ok_or_else(|| ApiError::BadRequest("temperature is required for set_temperature".to_string()))?;
                home.set_temperature(&name, t).map_err(map_common_error)?;
            }
            other => return Err(ApiError::BadRequest(format!("unsupported command '{}' (use set_state|set_brightness|set_temperature)", other))),
        }
    }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::Request, "device_command", format!("command '{}' applied to '{}'", payload.command, name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn connect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    { state.home.write().await.connect_device(&name).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceConnected, "device", format!("device '{}' connected", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn disconnect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    { state.home.write().await.disconnect_device(&name).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceDisconnected, "device", format!("device '{}' disconnected", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn report_device_error(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeviceErrorRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    if payload.message.trim().is_empty() {
        return Err(ApiError::BadRequest("error message cannot be empty".to_string()));
    }
    { state.home.write().await.set_device_error(&name, payload.message.clone()).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceError, "device", format!("device '{}' error: {}", name, payload.message), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn clear_device_error(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    { state.home.write().await.clear_device_error(&name).map_err(map_common_error)?; }
    let device = { state.home.read().await.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))? };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' error cleared", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn list_device_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<Vec<crate::http::types::ServerEvent>> {
    let events = state.events.read().await;
    let list = events.iter()
        .filter(|e| e.device_name.as_ref().map(|dn| dn.eq_ignore_ascii_case(&name)).unwrap_or(false))
        .cloned()
        .collect();
    Json(list)
}
