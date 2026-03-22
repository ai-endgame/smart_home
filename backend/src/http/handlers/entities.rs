use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;

use crate::domain::device::EntityKind;
use crate::http::{
    errors::ApiError,
    types::{EntitiesQuery, EntityResponse},
};
use crate::state::AppState;

fn entity_response(e: crate::domain::device::Entity) -> EntityResponse {
    EntityResponse {
        entity_id: e.entity_id,
        kind: e.kind.to_string(),
        device_id: e.device_id,
        name: e.name,
        state: e.state,
        unit_of_measurement: e.unit_of_measurement,
        attributes: e.attributes,
    }
}

/// GET /api/entities — flat list of all entities across the home, optional ?kind= filter.
pub async fn list_entities(
    State(state): State<AppState>,
    Query(query): Query<EntitiesQuery>,
) -> Result<Json<Vec<EntityResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();
    let home = state.home.read().await;
    let mut entities: Vec<EntityResponse> = home
        .list_devices()
        .into_iter()
        .flat_map(|d| d.entities())
        .filter(|e| {
            query.kind.as_deref()
                .is_none_or(|k| e.kind.to_string() == k)
        })
        .map(entity_response)
        .collect();

    // Append person entities from the presence registry.
    let kind_filter = query.kind.as_deref();
    if kind_filter.is_none_or(|k| k == "person") {
        let presence = state.presence.read().await;
        for person in presence.list() {
            let slug = crate::domain::device::slugify(&person.name);
            entities.push(EntityResponse {
                entity_id: format!("person.{}", slug),
                kind: EntityKind::Person.to_string(),
                device_id: person.id.clone(),
                name: person.name.clone(),
                state: person.effective_state(now).to_string(),
                unit_of_measurement: None,
                attributes: serde_json::json!({}),
            });
        }
    }

    entities.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
    Ok(Json(entities))
}

/// GET /api/entities/{entity_id} — single entity lookup by entity_id.
pub async fn get_entity(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
) -> Result<Json<EntityResponse>, ApiError> {
    let now = Utc::now();
    let home = state.home.read().await;

    // Search device entities first.
    for device in home.list_devices() {
        for e in device.entities() {
            if e.entity_id == entity_id {
                return Ok(Json(entity_response(e)));
            }
        }
    }

    // Check person entities.
    let presence = state.presence.read().await;
    for person in presence.list() {
        let slug = crate::domain::device::slugify(&person.name);
        let eid = format!("person.{}", slug);
        if eid == entity_id {
            return Ok(Json(EntityResponse {
                entity_id: eid,
                kind: EntityKind::Person.to_string(),
                device_id: person.id.clone(),
                name: person.name.clone(),
                state: person.effective_state(now).to_string(),
                unit_of_measurement: None,
                attributes: serde_json::json!({}),
            }));
        }
    }

    Err(ApiError::NotFound(format!("entity '{}' not found", entity_id)))
}

/// GET /api/devices/{name}/entities — entities for a specific device.
pub async fn list_device_entities(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<EntityResponse>>, ApiError> {
    let home = state.home.read().await;
    match home.get_device(&name) {
        Some(device) => {
            let entities = device.entities().into_iter().map(entity_response).collect();
            Ok(Json(entities))
        }
        None => Err(ApiError::NotFound(format!("Device '{}' not found.", name))),
    }
}
