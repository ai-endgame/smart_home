pub mod errors;
pub mod handlers;
pub mod helpers;
pub mod middleware;
pub mod router;
pub mod types;

use log::info;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::{
    config::Config,
    infrastructure::{db, mdns},
    state::AppState,
};

#[derive(Debug, Error)]
pub enum ServerStartError {
    #[error("invalid bind address '{0}'")]
    InvalidBindAddress(String),
    #[error("failed to start server: {0}")]
    Io(#[from] std::io::Error),
}

/// Start the full server: DB connection, device loading, mDNS discovery, HTTP.
pub async fn run_server_full(
    addr: Option<String>,
    database_url: Option<String>,
) -> Result<(), ServerStartError> {
    let mut config = Config::from_env();
    if let Some(a) = addr {
        config.bind_addr = a;
    }
    if let Some(url) = database_url {
        config.database_url = Some(url);
    }

    let socket_addr: std::net::SocketAddr = config.bind_addr.parse()
        .map_err(|_| ServerStartError::InvalidBindAddress(config.bind_addr.clone()))?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let mut state = AppState::new(Some(shutdown_tx));

    // ── Database ───────────────────────────────────────────────────────────
    if let Some(url) = &config.database_url {
        match db::create_pool(url).await {
            Ok(pool) => {
                match db::load_all_devices(&pool).await {
                    Ok(devices) => {
                        let count = devices.len();
                        let mut home = state.home.write().await;
                        for device in devices { home.insert_device(device); }
                        info!("database: loaded {count} device(s) from storage");
                    }
                    Err(e) => log::error!("database: failed to load devices: {e}"),
                }
                state.db = Some(pool);
            }
            Err(e) => log::error!("database: connection failed — running without persistence: {e}"),
        }
    } else {
        info!("no DATABASE_URL — running without persistence");
    }

    // ── mDNS discovery ─────────────────────────────────────────────────────
    mdns::start(state.discovery.clone());

    // ── HTTP server ────────────────────────────────────────────────────────
    let app = router::build_with_legacy_routes(state, &config.cors_origins);
    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    info!("smart_home_server listening on http://{}", socket_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await?;

    info!("smart_home_server stopped");
    Ok(())
}

/// Convenience wrapper — starts the server without an explicit database URL.
pub async fn run_server(addr: Option<String>) -> Result<(), ServerStartError> {
    run_server_full(addr, None).await
}

async fn shutdown_signal(mut rx: oneshot::Receiver<()>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => { info!("received ctrl-c shutdown signal"); }
        _ = &mut rx => { info!("received API shutdown signal"); }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tokio::sync::oneshot;
    use tower::ServiceExt;

    use crate::{http::router::build_with_legacy_routes, state::AppState};

    fn app() -> Router {
        build_with_legacy_routes(AppState::new(None), &[])
    }

    async fn request(
        app: &Router,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut builder = Request::builder().uri(uri).method(method);
        let request_body = match body {
            Some(payload) => { builder = builder.header("content-type", "application/json"); Body::from(payload.to_string()) }
            None => Body::empty(),
        };
        let response = app.clone().oneshot(builder.body(request_body).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json = if bytes.is_empty() { json!({}) } else { serde_json::from_slice(&bytes).unwrap() };
        (status, json)
    }

    #[tokio::test]
    async fn health_endpoint_works() {
        let app = app();
        let (status, body) = request(&app, "GET", "/health", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn create_and_fetch_device() {
        let app = app();
        let (status, _) = request(&app, "POST", "/devices", Some(json!({"name":"kitchen_light","device_type":"light"}))).await;
        assert_eq!(status, StatusCode::OK);
        let (status, json) = request(&app, "GET", "/devices/kitchen_light", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["name"], "kitchen_light");
        assert_eq!(json["device_type"], "light");
    }

    #[tokio::test]
    async fn automation_run_executes_actions() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "PATCH", "/devices/thermo/temperature", Some(json!({"temperature":30.0}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({"name":"cool_down","trigger":{"type":"temperature_above","device_name":"thermo","threshold":25.0},"action":{"type":"temperature","device_name":"thermo","temperature":22.0}}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/automation/run", None).await;
        assert_eq!(s, StatusCode::OK);
        let (s, json) = request(&app, "GET", "/devices/thermo", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(json["temperature"], 22.0);
    }

    #[tokio::test]
    async fn status_clients_and_events_flow() {
        let app = app();
        let (s, body) = request(&app, "GET", "/status", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["devices"], 0);
        let (s, body) = request(&app, "POST", "/clients/connect", Some(json!({"name":"mobile-app"}))).await;
        assert_eq!(s, StatusCode::OK);
        let client_id = body["client_id"].as_str().unwrap().to_string();
        let (s, clients) = request(&app, "GET", "/clients", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(clients.as_array().unwrap().len(), 1);
        let uri = format!("/clients/{}/disconnect", client_id);
        let (s, body) = request(&app, "POST", &uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["connected"], false);
        let (s, _) = request(&app, "POST", "/clients/unknown/disconnect", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
        let (s, events) = request(&app, "GET", "/events?limit=1", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(events.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn stop_server_endpoint_behaviors() {
        let app_no_shutdown = build_with_legacy_routes(AppState::new(None), &[]);
        let (s, _) = request(&app_no_shutdown, "POST", "/server/stop", None).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (tx, rx) = oneshot::channel();
        let app = build_with_legacy_routes(AppState::new(Some(tx)), &[]);
        let (s, body) = request(&app, "POST", "/server/stop", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["message"], "shutdown initiated");
        assert!(rx.await.is_ok());
    }

    #[tokio::test]
    async fn device_lifecycle_and_error_paths() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, err) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CONFLICT);
        assert_eq!(err["code"], "conflict");
        let (s, _) = request(&app, "PATCH", "/devices/lamp", Some(json!({"state":"on","brightness":80,"connected":true}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, err) = request(&app, "PATCH", "/devices/lamp", Some(json!({}))).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert_eq!(err["code"], "bad_request");
        let (s, _) = request(&app, "PATCH", "/devices/lamp/state", Some(json!({"state":"invalid"}))).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (s, _) = request(&app, "PATCH", "/devices/unknown/state", Some(json!({"state":"on"}))).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
        let (s, _) = request(&app, "POST", "/devices/lamp/connect", None).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/devices/lamp/disconnect", None).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/devices/lamp/error", Some(json!({"message":"   "}))).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        let (s, _) = request(&app, "POST", "/devices/lamp/error", Some(json!({"message":"offline"}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/devices/lamp/error/clear", None).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "DELETE", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }
}
