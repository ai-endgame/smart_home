use axum::{Json, extract::{Path, State}};
use axum::http::StatusCode;

use crate::domain::dashboard::{Card, Dashboard, View};
use crate::http::{
    errors::ApiError,
    helpers::validate_name,
    types::{CreateCardRequest, CreateDashboardRequest, CreateViewRequest},
};
use crate::infrastructure::db;
use crate::state::AppState;

pub async fn list_dashboards(State(state): State<AppState>) -> Json<Vec<Dashboard>> {
    let reg = state.dashboard.read().await;
    Json(reg.list().into_iter().cloned().collect())
}

pub async fn create_dashboard(
    State(state): State<AppState>,
    Json(payload): Json<CreateDashboardRequest>,
) -> Result<(StatusCode, Json<Dashboard>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("dashboard name cannot be empty".to_string()));
    }
    validate_name(&name)?;
    let dashboard = Dashboard::new(&name, payload.icon);
    let response = dashboard.clone();
    {
        let mut reg = state.dashboard.write().await;
        reg.add(dashboard.clone()).map_err(|e| ApiError::Conflict(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_dashboard(pool, &dashboard).await {
            log::error!("dashboard: failed to persist '{}': {}", dashboard.name, e);
        }
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Dashboard>, ApiError> {
    let reg = state.dashboard.read().await;
    let dashboard = reg.get(&id).ok_or_else(|| ApiError::NotFound(format!("dashboard '{}' not found", id)))?;
    Ok(Json(dashboard.clone()))
}

pub async fn delete_dashboard(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    {
        let mut reg = state.dashboard.write().await;
        reg.remove(&id).map_err(|e| ApiError::NotFound(e.to_string()))?;
    }
    if let Some(pool) = &state.db
        && let Err(e) = db::delete_dashboard(pool, &id).await {
            log::error!("dashboard: failed to delete '{}' from db: {}", id, e);
        }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateViewRequest>,
) -> Result<Json<Dashboard>, ApiError> {
    let title = payload.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("view title cannot be empty".to_string()));
    }
    validate_name(&title)?;
    let view = View::new(&title, payload.icon);
    let dashboard = {
        let mut reg = state.dashboard.write().await;
        let dash = reg.get_mut(&id).ok_or_else(|| ApiError::NotFound(format!("dashboard '{}' not found", id)))?;
        dash.views.push(view);
        dash.clone()
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_dashboard(pool, &dashboard).await {
            log::error!("dashboard: failed to persist view add for '{}': {}", id, e);
        }
    Ok(Json(dashboard))
}

pub async fn delete_view(
    State(state): State<AppState>,
    Path((id, view_id)): Path<(String, String)>,
) -> Result<Json<Dashboard>, ApiError> {
    let dashboard = {
        let mut reg = state.dashboard.write().await;
        let dash = reg.get_mut(&id).ok_or_else(|| ApiError::NotFound(format!("dashboard '{}' not found", id)))?;
        let before = dash.views.len();
        dash.views.retain(|v| v.id != view_id);
        if dash.views.len() == before {
            return Err(ApiError::NotFound(format!("view '{}' not found", view_id)));
        }
        dash.clone()
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_dashboard(pool, &dashboard).await {
            log::error!("dashboard: failed to persist view delete for '{}': {}", id, e);
        }
    Ok(Json(dashboard))
}

pub async fn add_card(
    State(state): State<AppState>,
    Path((id, view_id)): Path<(String, String)>,
    Json(content): Json<CreateCardRequest>,
) -> Result<Json<Dashboard>, ApiError> {
    let card = Card::new(content);
    let dashboard = {
        let mut reg = state.dashboard.write().await;
        let dash = reg.get_mut(&id).ok_or_else(|| ApiError::NotFound(format!("dashboard '{}' not found", id)))?;
        let view = dash.views.iter_mut().find(|v| v.id == view_id)
            .ok_or_else(|| ApiError::NotFound(format!("view '{}' not found", view_id)))?;
        view.cards.push(card);
        dash.clone()
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_dashboard(pool, &dashboard).await {
            log::error!("dashboard: failed to persist card add for '{}': {}", id, e);
        }
    Ok(Json(dashboard))
}

pub async fn delete_card(
    State(state): State<AppState>,
    Path((id, view_id, card_id)): Path<(String, String, String)>,
) -> Result<Json<Dashboard>, ApiError> {
    let dashboard = {
        let mut reg = state.dashboard.write().await;
        let dash = reg.get_mut(&id).ok_or_else(|| ApiError::NotFound(format!("dashboard '{}' not found", id)))?;
        let view = dash.views.iter_mut().find(|v| v.id == view_id)
            .ok_or_else(|| ApiError::NotFound(format!("view '{}' not found", view_id)))?;
        let before = view.cards.len();
        view.cards.retain(|c| c.id != card_id);
        if view.cards.len() == before {
            return Err(ApiError::NotFound(format!("card '{}' not found", card_id)));
        }
        dash.clone()
    };
    if let Some(pool) = &state.db
        && let Err(e) = db::upsert_dashboard(pool, &dashboard).await {
            log::error!("dashboard: failed to persist card delete for '{}': {}", id, e);
        }
    Ok(Json(dashboard))
}
