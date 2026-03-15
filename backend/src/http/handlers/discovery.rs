use axum::{Json, extract::State};

use crate::domain::device::DeviceType;
use crate::http::{
    errors::ApiError,
    helpers::{device_to_response, map_create_error, persist_device, record_event},
    types::{AddDiscoveredRequest, DeviceResponse, EventKind},
};
use crate::infrastructure::mdns::DiscoveredDevice;
use crate::state::AppState;

pub async fn list_discovered(State(state): State<AppState>) -> Json<Vec<DiscoveredDevice>> {
    let mut list: Vec<DiscoveredDevice> = state.discovery.read().unwrap().values().cloned().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Json(list)
}

pub async fn add_discovered_device(
    State(state): State<AppState>,
    Json(body): Json<AddDiscoveredRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let disc = {
        let store = state.discovery.read().unwrap();
        store.get(&body.discovered_id).cloned()
            .ok_or_else(|| ApiError::NotFound(format!("discovered device not found: '{}'", body.discovered_id)))?
    };

    let name = body.name.unwrap_or_else(|| disc.name.clone());
    let device_type = DeviceType::from_str_loose(&body.device_type).ok_or_else(|| {
        ApiError::BadRequest(format!("invalid device_type '{}'; use light|thermostat|lock|switch|sensor", body.device_type))
    })?;

    { state.home.write().await.add_device(&name, device_type).map_err(map_create_error)?; }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name).cloned()
            .ok_or_else(|| ApiError::Internal("device creation failed".to_string()))?
    };

    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("discovered device '{}' added to home", name), Some(name), None).await;
    Ok(Json(device_to_response(&device)))
}
