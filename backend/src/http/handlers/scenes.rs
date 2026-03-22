use std::collections::HashMap;

use axum::{Json, extract::{Path, State}};
use axum::http::StatusCode;

use crate::domain::scene::{Scene, SceneRegistry, SceneState};
use crate::http::{
    errors::ApiError,
    helpers::{record_event, validate_name},
    types::{ApplySceneResponse, CreateSceneRequest, EventKind, SceneResponse, SnapshotSceneRequest, UpdateSceneRequest},
};
use crate::infrastructure::db;
use crate::state::AppState;

pub async fn list_scenes(State(state): State<AppState>) -> Json<Vec<SceneResponse>> {
    let reg = state.scenes.read().await;
    Json(reg.list().iter().map(|s| SceneResponse::from(*s)).collect())
}

pub async fn get_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SceneResponse>, ApiError> {
    let reg = state.scenes.read().await;
    let scene = reg.get(&id).ok_or_else(|| ApiError::NotFound(format!("scene '{}' not found", id)))?;
    Ok(Json(SceneResponse::from(scene)))
}

pub async fn create_scene(
    State(state): State<AppState>,
    Json(payload): Json<CreateSceneRequest>,
) -> Result<(StatusCode, Json<SceneResponse>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("scene name cannot be empty".to_string()));
    }
    validate_name(&name)?;
    let mut states: HashMap<String, SceneState> = HashMap::new();
    for (device_id, s) in payload.states {
        states.insert(device_id, s.to_domain()?);
    }
    let scene = Scene::new(&name, states);
    let response = SceneResponse::from(&scene);
    {
        let mut reg = state.scenes.write().await;
        reg.add(scene.clone()).map_err(|e| ApiError::Conflict(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_scene(pool, &scene).await {
            log::error!("scenes: failed to persist '{}': {}", scene.name, e);
        }
    record_event(&state, EventKind::Server, "scene", format!("scene '{}' created", name), None, None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn snapshot_scene(
    State(state): State<AppState>,
    Json(payload): Json<SnapshotSceneRequest>,
) -> Result<(StatusCode, Json<SceneResponse>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("scene name cannot be empty".to_string()));
    }
    let mut states: HashMap<String, SceneState> = HashMap::new();
    {
        let home = state.home.read().await;
        for device_id in &payload.device_ids {
            // Find device by id via reverse index
            if let Some(name_key) = home.devices_by_id().get(device_id)
                && let Some(device) = home.devices.get(name_key) {
                    states.insert(device_id.clone(), SceneState {
                        state: Some(device.state.clone()),
                        brightness: Some(device.brightness),
                        temperature: device.temperature,
                    });
                }
        }
    }
    let scene = Scene::new(&name, states);
    let response = SceneResponse::from(&scene);
    {
        let mut reg = state.scenes.write().await;
        reg.add(scene.clone()).map_err(|e| ApiError::Conflict(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_scene(pool, &scene).await {
            log::error!("scenes: failed to persist snapshot '{}': {}", scene.name, e);
        }
    record_event(&state, EventKind::Server, "scene", format!("scene '{}' created from snapshot", name), None, None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSceneRequest>,
) -> Result<Json<SceneResponse>, ApiError> {
    let mut states: HashMap<String, SceneState> = HashMap::new();
    for (device_id, s) in payload.states {
        states.insert(device_id, s.to_domain()?);
    }
    let updated_scene = {
        let mut reg = state.scenes.write().await;
        reg.update(&id, states).map_err(|e| ApiError::NotFound(e.to_string()))?
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_scene(pool, &updated_scene).await {
            log::error!("scenes: failed to persist update '{}': {}", updated_scene.name, e);
        }
    Ok(Json(SceneResponse::from(&updated_scene)))
}

pub async fn delete_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let scene = {
        let mut reg = state.scenes.write().await;
        reg.remove(&id).map_err(|e| ApiError::NotFound(e.to_string()))?
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::delete_scene(pool, &id).await {
            log::error!("scenes: failed to delete '{}' from db: {}", id, e);
        }
    record_event(&state, EventKind::Server, "scene", format!("scene '{}' deleted", scene.name), None, None).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn apply_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApplySceneResponse>, ApiError> {
    let scene = {
        let reg = state.scenes.read().await;
        reg.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("scene '{}' not found", id)))?
    };
    let (applied, errors) = {
        let mut home = state.home.write().await;
        SceneRegistry::apply(&scene, &mut home)
    };
    // Persist changed devices
    if let Some(pool) = &state.db {
        let home = state.home.read().await;
        for device_id in scene.states.keys() {
            if let Some(name_key) = home.devices_by_id().get(device_id)
                && let Some(device) = home.devices.get(name_key) {
                    let _ = db::upsert_device(pool, device, None).await;
                }
        }
    }
    record_event(&state, EventKind::Server, "scene", format!("scene '{}' applied ({} devices)", scene.name, applied), None, None).await;
    Ok(Json(ApplySceneResponse { applied, errors }))
}
