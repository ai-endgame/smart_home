use axum::{Json, extract::{Path, State}};
use axum::http::StatusCode;
use chrono::Utc;

use crate::domain::presence::{PersonTracker, SourceState};
use crate::http::{
    errors::ApiError,
    helpers::{person_to_response, record_event, validate_name},
    types::{CreatePersonRequest, EventKind, PersonResponse, UpdateSourceRequest},
};
use crate::infrastructure::db;
use crate::state::AppState;

pub async fn list_persons(State(state): State<AppState>) -> Json<Vec<PersonResponse>> {
    let now = Utc::now();
    let reg = state.presence.read().await;
    Json(reg.list().iter().map(|p| person_to_response(p, now)).collect())
}

pub async fn create_person(
    State(state): State<AppState>,
    Json(payload): Json<CreatePersonRequest>,
) -> Result<(StatusCode, Json<PersonResponse>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("person name cannot be empty".to_string()));
    }
    validate_name(&name)?;
    let grace = payload.grace_period_secs.unwrap_or(120);
    let person = PersonTracker::new(&name, grace);
    let now = Utc::now();
    let response = person_to_response(&person, now);
    {
        let mut reg = state.presence.write().await;
        reg.add(person.clone()).map_err(|e| ApiError::Conflict(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_person(pool, &person).await {
            log::error!("presence: failed to persist '{}': {}", person.name, e);
        }
    record_event(&state, EventKind::Server, "presence", format!("person '{}' created", name), None, None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_person(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PersonResponse>, ApiError> {
    let now = Utc::now();
    let reg = state.presence.read().await;
    let person = reg.get(&id).ok_or_else(|| ApiError::NotFound(format!("person '{}' not found", id)))?;
    Ok(Json(person_to_response(person, now)))
}

pub async fn delete_person(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let person = {
        let mut reg = state.presence.write().await;
        reg.remove(&id).map_err(|e| ApiError::NotFound(e.to_string()))?
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::delete_person(pool, &id).await {
            log::error!("presence: failed to delete '{}' from db: {}", id, e);
        }
    record_event(&state, EventKind::Server, "presence", format!("person '{}' deleted", person.name), None, None).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_source(
    State(state): State<AppState>,
    Path((id, source)): Path<(String, String)>,
    Json(payload): Json<UpdateSourceRequest>,
) -> Result<Json<PersonResponse>, ApiError> {
    let source_state = parse_source_state(&payload.state)?;
    let now = Utc::now();
    let person = {
        let mut reg = state.presence.write().await;
        reg.update_source(&id, &source, source_state, now)
            .map_err(|e| ApiError::NotFound(e.to_string()))?
            .clone()
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_person(pool, &person).await {
            log::error!("presence: failed to persist source update for '{}': {}", id, e);
        }
    Ok(Json(person_to_response(&person, now)))
}

fn parse_source_state(s: &str) -> Result<SourceState, ApiError> {
    match s.to_ascii_lowercase().as_str() {
        "home"    => Ok(SourceState::Home),
        "away"    => Ok(SourceState::Away),
        "unknown" => Ok(SourceState::Unknown),
        other => Err(ApiError::BadRequest(format!("invalid source state '{}' (use home|away|unknown)", other))),
    }
}
