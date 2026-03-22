use axum::{Json, extract::{Path, State}};
use serde::Serialize;

use crate::http::{
    errors::ApiError,
    helpers::{device_to_response, persist_device, record_event},
    types::{DeviceResponse, EnergyUpdateRequest, EventKind},
};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct EnergySummaryResponse {
    pub total_power_w: f64,
    pub devices_reporting: usize,
}

/// GET /api/energy/summary — aggregate power across all devices.
pub async fn get_energy_summary(State(state): State<AppState>) -> Json<EnergySummaryResponse> {
    let home = state.home.read().await;
    let devices = home.list_devices();
    let reporting: Vec<f64> = devices.iter().filter_map(|d| d.power_w).collect();
    Json(EnergySummaryResponse {
        total_power_w: reporting.iter().sum(),
        devices_reporting: reporting.len(),
    })
}

/// PATCH /api/devices/{name}/energy — update power_w and/or energy_kwh.
pub async fn update_device_energy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<EnergyUpdateRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let device = {
        let mut home = state.home.write().await;
        let device = home.get_device_mut(&name)
            .ok_or_else(|| ApiError::NotFound(format!("device '{}' not found", name)))?;
        if let Some(w) = payload.power_w {
            device.power_w = Some(w);
        }
        if let Some(kwh) = payload.energy_kwh {
            device.energy_kwh = Some(kwh);
        }
        device.clone()
    };
    persist_device(&state, &device).await;
    record_event(
        &state,
        EventKind::DeviceUpdated,
        &name,
        format!("energy updated: {:?}W / {:?}kWh", device.power_w, device.energy_kwh),
        Some(name.clone()),
        None,
    ).await;
    Ok(Json(device_to_response(&device)))
}
