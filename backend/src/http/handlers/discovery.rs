use axum::{Json, extract::State, http::StatusCode};

use crate::domain::device::DeviceType;
use crate::http::{
    errors::ApiError,
    helpers::{device_to_response, map_create_error, persist_device, record_event},
    types::{AddDiscoveredRequest, DeviceResponse, EventKind},
};
use crate::infrastructure::mdns::DiscoveredDevice;
use crate::infrastructure::device_control::protocol_from_service_type;
use crate::state::AppState;

pub async fn list_discovered(State(state): State<AppState>) -> Json<Vec<DiscoveredDevice>> {
    let mut list: Vec<DiscoveredDevice> = state.discovery.read().unwrap_or_else(|e| e.into_inner()).values().cloned().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Json(list)
}

pub async fn add_discovered_device(
    State(state): State<AppState>,
    Json(body): Json<AddDiscoveredRequest>,
) -> Result<(StatusCode, Json<DeviceResponse>), ApiError> {
    let disc = {
        let store = state.discovery.read().unwrap_or_else(|e| e.into_inner());
        store.get(&body.discovered_id).cloned()
            .ok_or_else(|| ApiError::NotFound(format!("discovered device not found: '{}'", body.discovered_id)))?
    };

    let name = body.name.unwrap_or_else(|| disc.name.clone());
    let device_type = DeviceType::from_str_loose(&body.device_type).ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid device_type '{}'; valid types: light, thermostat, fan, lock, switch, outlet, tv, speaker, media_player, sensor, camera, alarm, cover, hub",
            body.device_type
        ))
    })?;

    {
        let mut home = state.home.write().await;
        home.add_device(&name, device_type).map_err(map_create_error)?;

        // Attach endpoint + control protocol so physical-device commands work.
        if let Some(d) = home.get_device_mut(&name) {
            // Build endpoint from first address or fallback to host
            let host = disc.addresses.first()
                .map(|a| a.as_str())
                .unwrap_or(disc.host.as_str());
            d.endpoint = Some(format!("http://{}:{}", host, disc.port));
            d.control_protocol = protocol_from_service_type(&disc.service_type);
        }
    }

    let device = {
        let home = state.home.read().await;
        home.get_device(&name).cloned()
            .ok_or_else(|| ApiError::Internal("device creation failed".to_string()))?
    };

    persist_device(&state, &device).await;
    record_event(&state, EventKind::DeviceUpdated, "device", format!("discovered device '{}' added to home", name), Some(name), None).await;
    Ok((StatusCode::CREATED, Json(device_to_response(&device))))
}
