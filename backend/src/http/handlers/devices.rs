use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use serde_json::json;

use crate::domain::device::DeviceType;
use crate::domain::device::Protocol;
use crate::infrastructure::{device_control, matter_control, mqtt};
use crate::domain::device::HistoryEntry;
use crate::http::{
    errors::ApiError,
    helpers::{delete_from_db, device_to_response, map_common_error, map_create_error, persist_device, record_event, record_history, validate_name},
    types::{
        BrightnessUpdateRequest, CreateDeviceRequest, DeviceCommandRequest, DeviceErrorRequest,
        DeviceResponse, DevicesQuery, EventKind, HistoryQuery, MessageResponse, StateUpdateRequest,
        TemperatureUpdateRequest, UpdateDeviceRequest, parse_device_state,
    },
};
use crate::state::AppState;

pub async fn list_devices(
    State(state): State<AppState>,
    Query(query): Query<DevicesQuery>,
) -> Json<Vec<DeviceResponse>> {
    let home = state.home.read().await;
    let all: Vec<DeviceResponse> = home.list_devices().into_iter().map(device_to_response).collect();
    let limit = query.limit.unwrap_or(100).min(1000);
    let offset = query.offset.unwrap_or(0);
    let page: Vec<DeviceResponse> = all.into_iter().skip(offset).take(limit).collect();
    Json(page)
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
) -> Result<(StatusCode, Json<DeviceResponse>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty or whitespace".to_string()));
    }
    validate_name(&name)?;
    let device_type = DeviceType::from_str_loose(&payload.device_type).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid device_type '{}'; valid types: light, thermostat, fan, lock, switch, outlet, tv, speaker, media_player, sensor, camera, alarm, cover, hub",
            payload.device_type
        ))
    })?;
    { state.home.write().await.add_device(&name, device_type).map_err(map_create_error)?; }
    let device = {
        let home = state.home.read().await;
        home.get_device(&name).cloned()
            .ok_or_else(|| ApiError::Internal("device creation failed".to_string()))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' created", name), Some(name), None).await;
    Ok((StatusCode::CREATED, Json(device_to_response(&device))))
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
    let device = {
        let mut home = state.home.write().await;
        if let Some(ref s) = payload.state { home.set_state(&name, parse_device_state(s)?).map_err(map_common_error)?; }
        if let Some(b) = payload.brightness { home.set_brightness(&name, b).map_err(map_common_error)?; }
        if let Some(t) = payload.temperature { home.set_temperature(&name, t).map_err(map_common_error)?; }
        if let Some(c) = payload.connected {
            if c { home.connect_device(&name).map_err(map_common_error)?; }
            else { home.disconnect_device(&name).map_err(map_common_error)?; }
        }
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' updated", name), Some(name.clone()), None).await;
    // MQTT command publish for changed fields
    if let Some(client) = &state.mqtt_client {
        let mut patch = serde_json::Map::new();
        if payload.state.is_some() {
            let state_str = if device.state == crate::domain::device::DeviceState::On { "ON" } else { "OFF" };
            patch.insert("state".to_string(), json!(state_str));
        }
        if payload.brightness.is_some() {
            patch.insert("brightness".to_string(), json!(device.brightness));
        }
        if payload.temperature.is_some()
            && let Some(t) = device.temperature {
                patch.insert("temperature".to_string(), json!(t));
            }
        if !patch.is_empty() {
            let c = client.clone();
            let n = name.clone();
            tokio::spawn(async move { mqtt::publish_command(&c, &n, serde_json::Value::Object(patch)).await });
        }
    }
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_state(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<StateUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let state_value = parse_device_state(&payload.state)?;
    let device = {
        let mut home = state.home.write().await;
        home.set_state(&name, state_value).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_history(&state, &name, HistoryEntry { timestamp: chrono::Utc::now(), state: device.state.clone(), brightness: device.brightness, temperature: device.temperature }).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' state updated", name), Some(name.clone()), None).await;
    // Fire-and-forget: send command to physical device if endpoint is configured
    let d = device.clone();
    let s = device.state.clone();
    tokio::spawn(async move { device_control::send_state(d, &s).await });
    // Protocol-aware command dispatch
    if device.control_protocol == Some(Protocol::Matter) {
        if let Some(node_id) = device.node_id {
            let on = device.state == crate::domain::device::DeviceState::On;
            let n = name.clone();
            let s = state.clone();
            tokio::spawn(async move {
                if let Err(e) = matter_control::dispatch_onoff(node_id, on).await {
                    log::error!("Matter dispatch_onoff failed for '{}': {e}", n);
                    let mut home = s.home.write().await;
                    if let Some(d) = home.devices.get_mut(&n.to_lowercase()) { d.last_error = Some(e); }
                }
            });
        }
    } else if let Some(client) = &state.mqtt_client {
        let state_str = if device.state == crate::domain::device::DeviceState::On { "ON" } else { "OFF" };
        let patch = json!({"state": state_str});
        let c = client.clone();
        let n = name.clone();
        tokio::spawn(async move { mqtt::publish_command(&c, &n, patch).await });
    }
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_brightness(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<BrightnessUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
        let mut home = state.home.write().await;
        home.set_brightness(&name, payload.brightness).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_history(&state, &name, HistoryEntry { timestamp: chrono::Utc::now(), state: device.state.clone(), brightness: device.brightness, temperature: device.temperature }).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' brightness updated", name), Some(name.clone()), None).await;
    if device.control_protocol == Some(Protocol::Matter) {
        if let Some(node_id) = device.node_id {
            let brightness = device.brightness;
            let n = name.clone();
            let s = state.clone();
            tokio::spawn(async move {
                if let Err(e) = matter_control::dispatch_level(node_id, brightness).await {
                    log::error!("Matter dispatch_level failed for '{}': {e}", n);
                    let mut home = s.home.write().await;
                    if let Some(d) = home.devices.get_mut(&n.to_lowercase()) { d.last_error = Some(e); }
                }
            });
        }
    } else {
        let d = device.clone();
        let b = device.brightness;
        tokio::spawn(async move { device_control::send_brightness(d, b).await });
    }
    Ok(Json(device_to_response(&device)))
}

pub async fn set_device_temperature(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<TemperatureUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    payload.validate()?;
    let device = {
        let mut home = state.home.write().await;
        home.set_temperature(&name, payload.temperature).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_history(&state, &name, HistoryEntry { timestamp: chrono::Utc::now(), state: device.state.clone(), brightness: device.brightness, temperature: device.temperature }).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("device '{}' temperature updated", name), Some(name.clone()), None).await;
    if device.control_protocol == Some(Protocol::Matter)
        && let Some(node_id) = device.node_id {
            // Convert Celsius to approximate mireds: 1,000,000 / kelvin; use raw f64 * 100 as mired approximation
            let mireds = (payload.temperature as f32 * 10.0) as u16;
            let n = name.clone();
            let s = state.clone();
            tokio::spawn(async move {
                if let Err(e) = matter_control::dispatch_color_temp(node_id, mireds).await {
                    log::error!("Matter dispatch_color_temp failed for '{}': {e}", n);
                    let mut home = s.home.write().await;
                    if let Some(d) = home.devices.get_mut(&n.to_lowercase()) { d.last_error = Some(e); }
                }
            });
        }
    Ok(Json(device_to_response(&device)))
}

pub async fn send_device_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<DeviceCommandRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
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
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::Request, "device_command", format!("command '{}' applied to '{}'", payload.command, name), Some(name.clone()), None).await;
    // Protocol-aware dispatch for set_state / set_brightness
    if device.control_protocol == Some(Protocol::Matter)
        && let Some(node_id) = device.node_id {
            match payload.command.to_lowercase().as_str() {
                "set_state" => {
                    let on = device.state == crate::domain::device::DeviceState::On;
                    let n = name.clone(); let s = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = matter_control::dispatch_onoff(node_id, on).await {
                            log::error!("Matter dispatch_onoff failed for '{}': {e}", n);
                            let mut home = s.home.write().await;
                            if let Some(d) = home.devices.get_mut(&n.to_lowercase()) { d.last_error = Some(e); }
                        }
                    });
                }
                "set_brightness" => {
                    let b = device.brightness;
                    let n = name.clone(); let s = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = matter_control::dispatch_level(node_id, b).await {
                            log::error!("Matter dispatch_level failed for '{}': {e}", n);
                            let mut home = s.home.write().await;
                            if let Some(d) = home.devices.get_mut(&n.to_lowercase()) { d.last_error = Some(e); }
                        }
                    });
                }
                _ => {}
            }
        }
    Ok(Json(device_to_response(&device)))
}

pub async fn connect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
        let mut home = state.home.write().await;
        home.connect_device(&name).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceConnected, "device", format!("device '{}' connected", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn disconnect_device(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
        let mut home = state.home.write().await;
        home.disconnect_device(&name).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
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
    let device = {
        let mut home = state.home.write().await;
        home.set_device_error(&name, payload.message.clone()).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceError, "device", format!("device '{}' error: {}", name, payload.message), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}

pub async fn clear_device_error(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
        let mut home = state.home.write().await;
        home.clear_device_error(&name).map_err(map_common_error)?;
        home.get_device(&name).cloned().ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?
    };
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

pub async fn get_device_history(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<HistoryEntry>>, ApiError> {
    {
        let home = state.home.read().await;
        if home.get_device(&name).is_none() {
            return Err(ApiError::NotFound(format!("device '{}' not found", name)));
        }
    }
    let history = state.history.read().await;
    let key = name.to_lowercase();
    let entries = match history.get(&key) {
        None => vec![],
        Some(buf) => {
            if let Some(limit) = query.limit {
                buf.iter().rev().take(limit).cloned().collect::<Vec<_>>().into_iter().rev().collect()
            } else {
                buf.iter().cloned().collect()
            }
        }
    };
    Ok(Json(entries))
}
