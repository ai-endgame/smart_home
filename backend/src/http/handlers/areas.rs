use axum::{
    Json,
    extract::{Path, State},
};

use crate::http::{
    errors::ApiError,
    helpers::device_to_response,
    types::{AreaDetailResponse, AreaResponse},
};
use crate::state::AppState;

/// GET /api/areas — all areas with device count.
pub async fn list_areas(State(state): State<AppState>) -> Json<Vec<AreaResponse>> {
    let home = state.home.read().await;
    let mut areas: Vec<AreaResponse> = home
        .list_rooms()
        .into_iter()
        .map(|a| AreaResponse {
            area_id: a.area_id.clone(),
            name: a.name.clone(),
            floor: a.floor,
            icon: a.icon.clone(),
            device_count: a.device_ids.len(),
        })
        .collect();
    areas.sort_by(|a, b| a.name.cmp(&b.name));
    Json(areas)
}

/// GET /api/areas/{area_id} — area detail with full device list.
pub async fn get_area(
    State(state): State<AppState>,
    Path(area_id): Path<String>,
) -> Result<Json<AreaDetailResponse>, ApiError> {
    let home = state.home.read().await;
    match home.get_area(&area_id) {
        Some(area) => {
            let devices = home
                .get_room_devices(&area.name)
                .into_iter()
                .map(device_to_response)
                .collect();
            Ok(Json(AreaDetailResponse {
                area_id: area.area_id.clone(),
                name: area.name.clone(),
                floor: area.floor,
                icon: area.icon.clone(),
                device_count: area.device_ids.len(),
                devices,
            }))
        }
        None => Err(ApiError::NotFound(format!("Area '{}' not found.", area_id))),
    }
}
