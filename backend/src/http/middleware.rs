use axum::extract::State;
use axum::http::{HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::IntoResponse;
use tower_http::cors::CorsLayer;

use crate::http::errors::ApiError;
use crate::state::AppState;

/// Combined auth middleware supporting both static API key and PIN-based session tokens.
///
/// Auth is skipped entirely when neither `api_key` nor `pin_hash` is set.
/// GET and OPTIONS requests always pass through (read-only access is public).
/// All other methods require `Authorization: Bearer <token>` or `X-API-Key: <key>`.
/// A valid token is one that matches `state.api_key` OR the current `state.session_token`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    // Auth disabled when neither mechanism is configured.
    let has_api_key  = state.api_key.is_some();
    let has_pin_auth = state.pin_hash.is_some();
    if !has_api_key && !has_pin_auth {
        return next.run(req).await.into_response();
    }

    let method = req.method().clone();
    if method == Method::GET || method == Method::OPTIONS {
        return next.run(req).await.into_response();
    }

    // Allow login and status endpoints through without auth.
    let path = req.uri().path();
    if path == "/api/auth/login" || path == "/api/auth/status" {
        return next.run(req).await.into_response();
    }

    let supplied = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
        .or_else(|| {
            req.headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        });

    let Some(ref token) = supplied else {
        return ApiError::Unauthorized("missing Authorization header".to_string()).into_response();
    };

    // Accept if it matches the static API key.
    if let Some(ref key) = state.api_key {
        if token == key {
            return next.run(req).await.into_response();
        }
    }

    // Accept if it matches the current PIN session token.
    if has_pin_auth {
        let session = state.session_token.lock().await;
        if let Some(ref st) = *session {
            if token == st {
                return next.run(req).await.into_response();
            }
        }
    }

    ApiError::Unauthorized("invalid or missing auth token".to_string()).into_response()
}

/// Build CORS layer. Allows all origins by default; in production restrict via CORS_ORIGINS env var.
pub fn cors_layer(allowed_origins: &[String]) -> CorsLayer {
    if allowed_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .iter()
            .filter_map(|o| match o.parse() {
                Ok(v) => Some(v),
                Err(_) => {
                    log::warn!("CORS: ignoring invalid origin '{}'", o);
                    None
                }
            })
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
    }
}
