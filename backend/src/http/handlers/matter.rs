use axum::{Json, extract::State};

use crate::domain::device::Protocol;
use crate::http::types::{FabricResponse, MatterDeviceResponse, MatterStatusResponse};
use crate::state::AppState;

pub async fn get_matter_status(State(state): State<AppState>) -> Json<MatterStatusResponse> {
    let status = state.matter_status.read().unwrap_or_else(|e| e.into_inner());
    Json(MatterStatusResponse {
        devices_seen: status.devices_seen,
        commissioning_count: status.commissioning_count,
        last_seen_at: status.last_seen_at.clone(),
        sync_enabled: status.sync_enabled,
        last_sync_at: status.last_sync_at.clone(),
    })
}

pub async fn list_matter_devices(State(state): State<AppState>) -> Json<Vec<MatterDeviceResponse>> {
    let store = state.discovery.read().unwrap_or_else(|e| e.into_inner());
    let devices: Vec<MatterDeviceResponse> = store
        .values()
        .filter(|d| d.protocol == Some(Protocol::Matter))
        .map(|d| MatterDeviceResponse {
            id: d.id.clone(),
            name: d.name.clone(),
            host: d.host.clone(),
            vendor_id: d.properties.get("vendor_id").and_then(|v| v.parse().ok()),
            product_id: d.properties.get("product_id").and_then(|v| v.parse().ok()),
            discriminator: d.properties.get("discriminator").and_then(|v| v.parse().ok()),
            commissioning_mode: d.properties.get("commissioning_mode")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            thread_role: d.properties.get("thread_role").cloned(),
            protocol: "matter".to_string(),
        })
        .collect();
    Json(devices)
}

pub async fn list_matter_fabrics(State(state): State<AppState>) -> Json<Vec<FabricResponse>> {
    let home = state.home.read().await;
    let mut fabric_map: std::collections::HashMap<String, (FabricResponse, usize)> = std::collections::HashMap::new();

    for device in home.devices.values() {
        if let Some(ref fabric) = device.matter_fabric {
            let entry = fabric_map.entry(fabric.fabric_id.clone()).or_insert_with(|| {
                (FabricResponse {
                    fabric_id: fabric.fabric_id.clone(),
                    vendor_id: fabric.vendor_id,
                    commissioner: fabric.commissioner.clone(),
                    device_count: 0,
                }, 0)
            });
            entry.0.device_count += 1;
        }
    }

    let fabrics: Vec<FabricResponse> = fabric_map.into_values().map(|(f, _)| f).collect();
    Json(fabrics)
}
