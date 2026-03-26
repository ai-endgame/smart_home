use std::collections::HashMap;

use axum::{Json, extract::State};

use crate::domain::device::Protocol;
use crate::http::types::{
    EcosystemLayers, EcosystemResponse, ProtocolEntry, ProtocolInfoResponse,
};
use crate::state::AppState;

/// GET /api/protocols — static registry of all supported protocols with metadata.
pub async fn get_protocols() -> Json<Vec<ProtocolInfoResponse>> {
    let list = Protocol::all()
        .iter()
        .map(|p| {
            let info = p.info();
            ProtocolInfoResponse {
                id: p.to_string(),
                transport: info.transport,
                local_only: info.local_only,
                mesh: info.mesh,
                description: info.description,
            }
        })
        .collect();
    Json(list)
}

/// GET /api/ecosystem — live topology snapshot of the home.
pub async fn get_ecosystem(State(state): State<AppState>) -> Json<EcosystemResponse> {
    let home = state.home.read().await;
    let devices: Vec<_> = home.list_devices();

    let total = devices.len();
    let connected = devices.iter().filter(|d| d.connected).count();
    let disconnected = total - connected;
    let unprotocolled = devices.iter().filter(|d| d.control_protocol.is_none()).count();

    // Count devices per protocol
    let mut protocol_counts: HashMap<String, usize> = HashMap::new();
    for device in &devices {
        if let Some(p) = &device.control_protocol {
            *protocol_counts.entry(p.to_string()).or_insert(0) += 1;
        }
    }

    // Layer counts
    let local_devices = devices
        .iter()
        .filter(|d| d.control_protocol.as_ref().is_some_and(|p| p.info().local_only))
        .count();
    let cloud_devices = devices
        .iter()
        .filter(|d| d.control_protocol.as_ref().is_some_and(|p| !p.info().local_only))
        .count();

    // Build protocol entries — only protocols present in the home
    let mut protocols: Vec<ProtocolEntry> = protocol_counts
        .iter()
        .filter_map(|(id, &count)| {
            Protocol::from_str_loose(id).map(|p| {
                let info = p.info();
                ProtocolEntry {
                    id: id.clone(),
                    transport: info.transport,
                    local_only: info.local_only,
                    mesh: info.mesh,
                    description: info.description,
                    device_count: count,
                }
            })
        })
        .collect();
    // Stable order: sort by id
    protocols.sort_by(|a, b| a.id.cmp(&b.id));

    Json(EcosystemResponse {
        total_devices: total,
        connected_count: connected,
        disconnected_count: disconnected,
        unprotocolled_devices: unprotocolled,
        layers: EcosystemLayers { local_devices, cloud_devices },
        protocols,
    })
}
