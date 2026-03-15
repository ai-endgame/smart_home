use axum::{Json, extract::{Path, Query, State}};
use chrono::Utc;
use uuid::Uuid;

use crate::http::{
    errors::ApiError,
    helpers::record_event,
    types::{
        ClientSession, CreateClientRequest, EventKind, EventsQuery, HealthResponse,
        MessageResponse, StatusResponse,
    },
};
use crate::state::AppState;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let home = state.home.read().await;
    let rules = state.automation.read().await;
    let clients = state.clients.read().await;
    let events = state.events.read().await;

    let connected_devices = home.devices.values().filter(|d| d.connected).count();
    let connected_clients = clients.values().filter(|c| c.connected).count();

    Json(StatusResponse {
        devices: home.devices.len(),
        connected_devices,
        rooms: home.rooms.len(),
        rules: rules.rules.len(),
        clients: clients.len(),
        connected_clients,
        events: events.len(),
    })
}

pub async fn stop_server(State(state): State<AppState>) -> Result<Json<MessageResponse>, ApiError> {
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        tx.send(()).map_err(|_| ApiError::Internal("failed to stop server".to_string()))?;
        record_event(&state, EventKind::Server, "server", "shutdown requested by API".to_string(), None, None).await;
        return Ok(Json(MessageResponse { message: "shutdown initiated".to_string() }));
    }
    Err(ApiError::BadRequest("shutdown signal unavailable".to_string()))
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> Json<Vec<crate::http::types::ServerEvent>> {
    let events = state.events.read().await;
    let mut list = events.clone();
    if let Some(limit) = query.limit {
        if list.len() > limit {
            list = list[list.len() - limit..].to_vec();
        }
    }
    Json(list)
}

pub async fn connect_client(
    State(state): State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<Json<ClientSession>, ApiError> {
    let client_id = Uuid::new_v4().to_string();
    let name = payload.name.filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "anonymous".to_string());
    let session = ClientSession {
        client_id: client_id.clone(),
        name,
        connected: true,
        connected_at: Utc::now().to_rfc3339(),
        disconnected_at: None,
    };
    state.clients.write().await.insert(client_id.clone(), session.clone());
    record_event(&state, EventKind::ClientConnected, "client", format!("client '{}' connected", client_id), None, Some(client_id)).await;
    Ok(Json(session))
}

pub async fn disconnect_client(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
) -> Result<Json<ClientSession>, ApiError> {
    let updated = {
        let mut clients = state.clients.write().await;
        let session = clients.get_mut(&client_id)
            .ok_or_else(|| ApiError::NotFound(format!("client '{}' not found", client_id)))?;
        session.connected = false;
        session.disconnected_at = Some(Utc::now().to_rfc3339());
        session.clone()
    };
    record_event(&state, EventKind::ClientDisconnected, "client", format!("client '{}' disconnected", client_id), None, Some(client_id)).await;
    Ok(Json(updated))
}

pub async fn list_clients(State(state): State<AppState>) -> Json<Vec<ClientSession>> {
    let clients = state.clients.read().await;
    let mut list = clients.values().cloned().collect::<Vec<_>>();
    list.sort_by(|a, b| a.client_id.cmp(&b.client_id));
    Json(list)
}
