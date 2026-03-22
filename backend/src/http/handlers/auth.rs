use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::http::errors::ApiError;
use crate::state::AppState;

// ── SHA-256 helper ────────────────────────────────────────────────────────────

/// Returns the lowercase hex SHA-256 digest of the input string.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub auth_enabled: bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/auth/status — returns whether auth is enabled, no token required.
pub async fn auth_status(State(state): State<AppState>) -> Json<AuthStatusResponse> {
    let auth_enabled = state.api_key.is_some() || state.pin_hash.is_some();
    Json(AuthStatusResponse { auth_enabled })
}

/// POST /api/auth/login — validate PIN, issue session token.
/// Returns 403 if PIN auth is not configured (use API_KEY env instead).
/// Returns 401 if the supplied PIN is incorrect.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let Some(ref expected_hash) = state.pin_hash else {
        return Err(ApiError::Forbidden(
            "PIN auth is not configured on this server".to_string(),
        ));
    };

    if payload.pin.is_empty() {
        return Err(ApiError::Unauthorized("PIN cannot be empty".to_string()));
    }

    let supplied_hash = sha256_hex(&payload.pin);
    if supplied_hash != *expected_hash {
        return Err(ApiError::Unauthorized("incorrect PIN".to_string()));
    }

    // Generate a fresh session token and store it.
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    *state.session_token.lock().await = Some(token.clone());
    Ok(Json(LoginResponse { token }))
}
