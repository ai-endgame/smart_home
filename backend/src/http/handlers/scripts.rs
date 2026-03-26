use axum::{Json, extract::{Path, State}};
use axum::http::StatusCode;

use crate::domain::script::Script;
use crate::http::{
    errors::ApiError,
    helpers::{record_event, validate_name},
    types::{CreateScriptRequest, EventKind, RunScriptRequest, RunScriptResponse, ScriptResponse},
};
use crate::infrastructure::{db, script_executor};
use crate::state::AppState;

pub async fn list_scripts(State(state): State<AppState>) -> Json<Vec<ScriptResponse>> {
    let reg = state.scripts.read().await;
    Json(reg.list().iter().map(|s| ScriptResponse::from(*s)).collect())
}

pub async fn get_script(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ScriptResponse>, ApiError> {
    let reg = state.scripts.read().await;
    let script = reg.get(&id).ok_or_else(|| ApiError::NotFound(format!("script '{}' not found", id)))?;
    Ok(Json(ScriptResponse::from(script)))
}

pub async fn create_script(
    State(state): State<AppState>,
    Json(payload): Json<CreateScriptRequest>,
) -> Result<(StatusCode, Json<ScriptResponse>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("script name cannot be empty".to_string()));
    }
    validate_name(&name)?;
    let script = Script::new(&name, &payload.description, payload.params, payload.steps);
    let response = ScriptResponse::from(&script);
    {
        let mut reg = state.scripts.write().await;
        reg.add(script.clone()).map_err(|e| ApiError::Conflict(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_script(pool, &script).await {
            log::error!("scripts: failed to persist '{}': {}", script.name, e);
        }
    record_event(&state, EventKind::Server, "script", format!("script '{}' created", name), None, None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_script(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateScriptRequest>,
) -> Result<Json<ScriptResponse>, ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("script name cannot be empty".to_string()));
    }
    let updated = Script {
        id: id.clone(),
        name,
        description: payload.description,
        params: payload.params,
        steps: payload.steps,
    };
    let response = ScriptResponse::from(&updated);
    {
        let mut reg = state.scripts.write().await;
        reg.update(&id, updated.clone()).map_err(|e| match e {
            crate::domain::error::DomainError::NotFound(m) => ApiError::NotFound(m),
            other => ApiError::Conflict(other.to_string()),
        })?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_script(pool, &updated).await {
            log::error!("scripts: failed to persist update '{}': {}", updated.name, e);
        }
    Ok(Json(response))
}

pub async fn delete_script(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let script = {
        let mut reg = state.scripts.write().await;
        reg.remove(&id).map_err(|e| ApiError::NotFound(e.to_string()))?
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::delete_script(pool, &id).await {
            log::error!("scripts: failed to delete '{}' from db: {}", id, e);
        }
    record_event(&state, EventKind::Server, "script", format!("script '{}' deleted", script.name), None, None).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_script(
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Option<Json<RunScriptRequest>>,
) -> Result<(StatusCode, Json<RunScriptResponse>), ApiError> {
    let script = {
        let reg = state.scripts.read().await;
        reg.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("script '{}' not found", id)))?
    };
    let args = payload.map(|p| p.0.args).unwrap_or_default();
    let script_id = script.id.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let errors = script_executor::run_script(script, args, state_clone, 0).await;
        if !errors.is_empty() {
            log::warn!("script run errors: {:?}", errors);
        }
    });
    Ok((StatusCode::ACCEPTED, Json(RunScriptResponse { script_id, status: "accepted" })))
}
