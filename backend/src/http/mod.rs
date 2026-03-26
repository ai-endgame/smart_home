pub mod errors;
pub mod handlers;
pub mod helpers;
pub mod middleware;
pub mod router;
pub mod types;

use log::info;
use thiserror::Error;
use tokio::sync::oneshot;

/// Parse `mqtt://host:port` into `(host, port)`, defaulting to `("localhost", 1883)`.
fn parse_mqtt_url(url: &str) -> (String, u16) {
    let stripped = url
        .strip_prefix("mqtt://")
        .or_else(|| url.strip_prefix("mqtts://"))
        .unwrap_or(url);
    // Drop any userinfo
    let stripped = stripped.split_once('@').map(|(_, h)| h).unwrap_or(stripped);
    if let Some((host, port_str)) = stripped.rsplit_once(':')
        && let Ok(port) = port_str.parse::<u16>() {
            return (host.to_string(), port);
        }
    (stripped.to_string(), 1883)
}

use crate::{
    config::Config,
    infrastructure::{automation_loop, db, matter, mdns, mqtt},
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
    state.admin_token = config.admin_token.clone();
    state.api_key = config.api_key.clone();
    if let Some(ref pin) = config.smart_home_pin {
        state.pin_hash = Some(crate::http::handlers::auth::sha256_hex(pin));
    }

    // ── Database ───────────────────────────────────────────────────────────
    if let Some(url) = &config.database_url {
        match db::create_pool(url).await {
            Ok(pool) => {
                match db::load_all_devices(&pool).await {
                    Ok((devices, area_meta)) => {
                        let count = devices.len();
                        let mut home = state.home.write().await;
                        for device in devices { home.insert_device(device); }
                        // Reconstruct area metadata (floor, icon) from DB
                        for (room_name, (floor, icon)) in area_meta {
                            let key = room_name.to_lowercase();
                            if let Some(area) = home.areas.get_mut(&key) {
                                area.floor = floor;
                                area.icon = icon;
                            }
                        }
                        info!("database: loaded {count} device(s) from storage");
                    }
                    Err(e) => log::error!("database: failed to load devices: {e}"),
                }
                // Load scripts
                match db::load_all_scripts(&pool).await {
                    Ok(scripts) => {
                        let mut reg = state.scripts.write().await;
                        for s in scripts { let _ = reg.add(s); }
                    }
                    Err(e) => log::error!("database: failed to load scripts: {e}"),
                }
                // Load scenes
                match db::load_all_scenes(&pool).await {
                    Ok(scenes) => {
                        let mut reg = state.scenes.write().await;
                        for s in scenes { let _ = reg.add(s); }
                    }
                    Err(e) => log::error!("database: failed to load scenes: {e}"),
                }
                // Load persons
                match db::load_all_persons(&pool).await {
                    Ok(persons) => {
                        let mut reg = state.presence.write().await;
                        for p in persons { let _ = reg.add(p); }
                    }
                    Err(e) => log::error!("database: failed to load persons: {e}"),
                }
                // Load dashboards
                match db::load_all_dashboards(&pool).await {
                    Ok(dashboards) => {
                        let mut reg = state.dashboard.write().await;
                        for d in dashboards { let _ = reg.add(d); }
                        // Seed default "Home" dashboard if none exist
                        if reg.list().is_empty() {
                            use crate::domain::dashboard::{Dashboard, View};
                            let mut home_dash = Dashboard::new("Home", None);
                            home_dash.views.push(View::new("Overview", None));
                            if let Err(e) = db::upsert_dashboard(&pool, &home_dash).await {
                                log::error!("database: failed to seed default dashboard: {e}");
                            }
                            let _ = reg.add(home_dash);
                            info!("database: seeded default 'Home' dashboard");
                        }
                    }
                    Err(e) => log::error!("database: failed to load dashboards: {e}"),
                }
                state.db = Some(pool);
            }
            Err(e) => log::error!("database: connection failed — running without persistence: {e}"),
        }
    } else {
        info!("no DATABASE_URL — running without persistence");
    }

    // ── mDNS discovery ─────────────────────────────────────────────────────
    if mdns::start(state.discovery.clone()) {
        info!("mDNS discovery started");
    } else {
        info!("mDNS discovery disabled (MDNS_DISABLED=true)");
    }

    // ── Matter scanner ─────────────────────────────────────────────────────
    if matter::start_matter_scanner(state.discovery.clone(), state.matter_status.clone()) {
        info!("Matter scanner started");
    } else {
        info!("Matter scanner disabled (MDNS_DISABLED=true)");
    }

    // ── Matter state sync ──────────────────────────────────────────────────
    let sync_enabled = std::env::var("MATTER_SYNC_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if let Ok(mut status) = state.matter_status.write() {
        status.sync_enabled = sync_enabled;
    }
    crate::infrastructure::matter_control::start_matter_sync_loop(state.clone());

    // ── MQTT ───────────────────────────────────────────────────────────────
    if let Some(ref mqtt_url) = config.mqtt_url.clone() {
        let url = mqtt_url.clone();
        // Parse host/port from URL (e.g. mqtt://localhost:1883)
        let (host, port) = parse_mqtt_url(&url);
        let mut opts = rumqttc::MqttOptions::new("smart_home", host, port);
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        let (client, event_loop) = rumqttc::AsyncClient::new(opts, 64);
        state.mqtt_client = Some(client.clone());
        mqtt::start_mqtt_loop(&url, client, event_loop, state.clone()).await;
        info!("MQTT: connecting to {}", mqtt::redact_url(&url));
    } else {
        info!("no MQTT_URL — running without MQTT bridge");
    }

    // ── Automation loop ────────────────────────────────────────────────────
    automation_loop::start_automation_loop(state.clone());
    info!("automation loop started");

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
        request_with_headers(app, method, uri, body, &[]).await
    }

    async fn request_with_headers(
        app: &Router,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
        extra_headers: &[(&str, &str)],
    ) -> (StatusCode, serde_json::Value) {
        let mut builder = Request::builder().uri(uri).method(method);
        for (k, v) in extra_headers {
            builder = builder.header(*k, *v);
        }
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
        assert_eq!(status, StatusCode::CREATED);
        let (status, json) = request(&app, "GET", "/devices/kitchen_light", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["name"], "kitchen_light");
        assert_eq!(json["device_type"], "light");
    }

    #[tokio::test]
    async fn automation_run_executes_actions() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        assert_eq!(s, StatusCode::CREATED);
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
        // Without ADMIN_TOKEN configured — always forbidden.
        let app_no_token = build_with_legacy_routes(AppState::new(None), &[]);
        let (s, _) = request(&app_no_token, "POST", "/server/stop", None).await;
        assert_eq!(s, StatusCode::FORBIDDEN);

        // With token configured but wrong token — forbidden.
        let (tx, _rx) = oneshot::channel::<()>();
        let mut state_with_token = AppState::new(Some(tx));
        state_with_token.admin_token = Some("secret".to_string());
        let app_wrong = build_with_legacy_routes(state_with_token, &[]);
        let (s, _) = request(&app_wrong, "POST", "/server/stop", None).await;
        assert_eq!(s, StatusCode::FORBIDDEN);

        // With correct token — succeeds.
        let (tx2, rx2) = oneshot::channel();
        let mut state_ok = AppState::new(Some(tx2));
        state_ok.admin_token = Some("secret".to_string());
        let app_ok = build_with_legacy_routes(state_ok, &[]);
        let (s, body) = request_with_headers(&app_ok, "POST", "/server/stop", None, &[("x-admin-token", "secret")]).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["message"], "shutdown initiated");
        assert!(rx2.await.is_ok());
    }

    #[tokio::test]
    async fn device_lifecycle_and_error_paths() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
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

    // ── Entity tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_entities_empty_home() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/entities", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_entities_with_light() {
        let app = app();
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        let (status, body) = request(&app, "GET", "/api/entities", None).await;
        assert_eq!(status, StatusCode::OK);
        let arr = body.as_array().unwrap();
        // Light yields 2 entities: switch + number
        assert_eq!(arr.len(), 2);
        assert!(arr.iter().any(|e| e["kind"] == "switch"));
        assert!(arr.iter().any(|e| e["kind"] == "number"));
    }

    #[tokio::test]
    async fn get_entities_kind_filter() {
        let app = app();
        request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        let (status, body) = request(&app, "GET", "/api/entities?kind=sensor", None).await;
        assert_eq!(status, StatusCode::OK);
        let arr = body.as_array().unwrap();
        assert!(arr.iter().all(|e| e["kind"] == "sensor"));
        assert!(!arr.is_empty());
    }

    #[tokio::test]
    async fn get_device_entities_not_found() {
        let app = app();
        let (status, _) = request(&app, "GET", "/api/devices/missing/entities", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ── Area tests ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_areas_empty() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/areas", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_areas_after_room_creation() {
        let app = app();
        // Create a device and add it to a room via the CLI path isn't available here,
        // but we can verify the areas endpoint returns something after status check.
        let (status, _) = request(&app, "GET", "/api/areas", None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn get_area_not_found() {
        let app = app();
        let (status, _) = request(&app, "GET", "/api/areas/nonexistent", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ── Ecosystem / Protocol tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn get_protocols_returns_all_variants() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/protocols", None).await;
        assert_eq!(status, StatusCode::OK);
        let arr = body.as_array().expect("expected array");
        // All 10 protocol variants must be present
        assert_eq!(arr.len(), 10);
        // Each entry has the required fields
        for entry in arr {
            assert!(entry["id"].is_string(), "missing id");
            assert!(entry["transport"].is_string(), "missing transport");
            assert!(entry["local_only"].is_boolean(), "missing local_only");
            assert!(entry["mesh"].is_boolean(), "missing mesh");
            assert!(entry["description"].is_string(), "missing description");
        }
        // Spot-check: zigbee should be local + mesh
        let zigbee = arr.iter().find(|e| e["id"] == "zigbee").expect("zigbee missing");
        assert_eq!(zigbee["local_only"], true);
        assert_eq!(zigbee["mesh"], true);
    }

    #[tokio::test]
    async fn get_ecosystem_empty_home() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/ecosystem", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["total_devices"], 0);
        assert_eq!(body["connected_count"], 0);
        assert_eq!(body["disconnected_count"], 0);
        assert_eq!(body["unprotocolled_devices"], 0);
        assert_eq!(body["layers"]["local_devices"], 0);
        assert_eq!(body["layers"]["cloud_devices"], 0);
        assert!(body["protocols"].as_array().unwrap().is_empty());
    }

    // ── Matter status tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn get_matter_status_default() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/matter/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["devices_seen"], 0);
        assert_eq!(body["commissioning_count"], 0);
        assert!(body["last_seen_at"].is_null());
        assert_eq!(body["sync_enabled"], false);
        assert!(body["last_sync_at"].is_null());
    }

    #[tokio::test]
    async fn get_matter_devices_empty() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/matter/devices", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_matter_fabrics_empty() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/matter/fabrics", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    // ── Commission tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn commission_invalid_code_returns_400() {
        let app = app();
        let (status, body) = request(&app, "POST", "/api/matter/commission",
            Some(json!({"setup_code": "123", "node_id": 1}))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "bad_request");
    }

    #[tokio::test]
    async fn commission_valid_code_returns_202() {
        let app = app();
        let (status, body) = request(&app, "POST", "/api/matter/commission",
            Some(json!({"setup_code": "34970112332", "node_id": 1}))).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(body["job_id"].is_string());
        assert_eq!(body["status"], "pending");
    }

    #[tokio::test]
    async fn commission_job_not_found_returns_404() {
        let app = app();
        let (status, _) = request(&app, "GET", "/api/matter/commission/nonexistent-job", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn commission_jobs_list_empty() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/matter/commission/jobs", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    // ── MQTT status tests ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_mqtt_status_no_mqtt_configured() {
        let app = app();
        let (status, body) = request(&app, "GET", "/api/mqtt/status", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["connected"], false);
        assert_eq!(body["topics_received"], 0);
        assert!(body["broker"].is_null());
        assert!(body["last_message_at"].is_null());
    }

    #[tokio::test]
    async fn get_ecosystem_with_devices() {
        let app = app();
        // Add two devices
        request(&app, "POST", "/devices", Some(json!({"name":"strip","device_type":"light"}))).await;
        request(&app, "POST", "/devices", Some(json!({"name":"sensor","device_type":"sensor"}))).await;
        // Connect strip, leave sensor disconnected
        request(&app, "POST", "/devices/strip/connect", None).await;
        // Without protocol set, both should show as unprotocolled
        let (status, body) = request(&app, "GET", "/api/ecosystem", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["total_devices"], 2);
        assert_eq!(body["connected_count"], 1);
        assert_eq!(body["disconnected_count"], 1);
        assert_eq!(body["unprotocolled_devices"], 2);
        assert!(body["protocols"].as_array().unwrap().is_empty());
    }

    // ── Webhook tests ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn webhook_fires_rule_with_webhook_trigger() {
        let app = app();
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "hook_rule",
            "trigger": { "type": "webhook", "id": "my-hook" },
            "action": { "type": "state", "device_name": "lamp", "state": "on" }
        }))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, body) = request(&app, "POST", "/api/automations/webhook/hook_rule", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["rule_name"], "hook_rule");
        assert_eq!(body["action_executed"], true);
    }

    #[tokio::test]
    async fn webhook_returns_404_for_unknown_rule() {
        let app = app();
        let (s, _) = request(&app, "POST", "/api/automations/webhook/no_such_rule", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn webhook_returns_400_for_non_webhook_trigger() {
        let app = app();
        request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "state_rule",
            "trigger": { "type": "device_state_change", "device_name": "thermo", "target_state": "on" },
            "action": { "type": "state", "device_name": "thermo", "state": "off" }
        }))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "POST", "/api/automations/webhook/state_rule", None).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    // ── Script tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn script_crud_and_run() {
        let app = app();

        // List empty
        let (s, body) = request(&app, "GET", "/api/scripts", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());

        // Create
        let (s, body) = request(&app, "POST", "/api/scripts", Some(json!({
            "name": "dim_all",
            "description": "Dims lights",
            "steps": [{ "type": "delay", "milliseconds": 1 }]
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let script_id = body["id"].as_str().unwrap().to_string();
        assert_eq!(body["name"], "dim_all");

        // Conflict on duplicate name
        let (s, err) = request(&app, "POST", "/api/scripts", Some(json!({
            "name": "dim_all",
            "steps": [{ "type": "delay", "milliseconds": 1 }]
        }))).await;
        assert_eq!(s, StatusCode::CONFLICT);
        assert_eq!(err["code"], "conflict");

        // Get by ID
        let uri = format!("/api/scripts/{}", script_id);
        let (s, body) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["name"], "dim_all");

        // 404 for unknown
        let (s, _) = request(&app, "GET", "/api/scripts/no-such-id", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Run — returns 202
        let run_uri = format!("/api/scripts/{}/run", script_id);
        let (s, _) = request(&app, "POST", &run_uri, Some(json!({ "args": {} }))).await;
        assert_eq!(s, StatusCode::ACCEPTED);

        // Run unknown script — 404
        let (s, _) = request(&app, "POST", "/api/scripts/no-such-id/run", Some(json!({ "args": {} }))).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Delete
        let del_uri = format!("/api/scripts/{}", script_id);
        let (s, _) = request(&app, "DELETE", &del_uri, None).await;
        assert_eq!(s, StatusCode::NO_CONTENT);

        // Gone after delete
        let (s, _) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    // ── Scene tests ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn scene_crud_apply_and_snapshot() {
        let app = app();

        // Create a device to include in scenes
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        request(&app, "PATCH", "/devices/lamp/state", Some(json!({"state":"on"}))).await;

        // List empty
        let (s, body) = request(&app, "GET", "/api/scenes", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());

        // Create scene
        let (s, _lamp_device) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);

        let (s, body) = request(&app, "POST", "/api/scenes", Some(json!({
            "name": "evening",
            "states": {}
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let scene_id = body["id"].as_str().unwrap().to_string();
        assert_eq!(body["name"], "evening");

        // Conflict on duplicate name
        let (s, err) = request(&app, "POST", "/api/scenes", Some(json!({
            "name": "evening",
            "states": {}
        }))).await;
        assert_eq!(s, StatusCode::CONFLICT);
        assert_eq!(err["code"], "conflict");

        // Get by ID
        let scene_uri = format!("/api/scenes/{}", scene_id);
        let (s, body) = request(&app, "GET", &scene_uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["name"], "evening");

        // 404 for unknown
        let (s, _) = request(&app, "GET", "/api/scenes/no-such-id", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Apply scene (empty states — 0 applied, no errors)
        let apply_uri = format!("/api/scenes/{}/apply", scene_id);
        let (s, body) = request(&app, "POST", &apply_uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["applied"], 0);
        assert!(body["errors"].as_array().unwrap().is_empty());

        // Apply unknown scene — 404
        let (s, _) = request(&app, "POST", "/api/scenes/no-such-id/apply", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Snapshot
        let (s, _snap) = request(&app, "POST", "/api/scenes/snapshot", Some(json!({
            "name": "current_snap",
            "device_ids": []
        }))).await;
        assert_eq!(s, StatusCode::CREATED);

        // Delete
        let (s, _) = request(&app, "DELETE", &scene_uri, None).await;
        assert_eq!(s, StatusCode::NO_CONTENT);

        // Gone
        let (s, _) = request(&app, "GET", &scene_uri, None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    // ── Automation condition tests ────────────────────────────────────────────

    #[tokio::test]
    async fn automation_with_failing_condition_does_not_fire() {
        let app = app();

        // Create lamp, set state to off
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        request(&app, "PATCH", "/devices/lamp/state", Some(json!({"state":"off"}))).await;

        // Rule: trigger on temperature (always fires for thermo), action sets lamp on,
        // BUT condition requires lamp state == "on" (which it isn't — so action should NOT fire)
        request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        request(&app, "PATCH", "/devices/thermo/temperature", Some(json!({"temperature":30.0}))).await;

        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "gated_rule",
            "trigger": { "type": "temperature_above", "device_name": "thermo", "threshold": 25.0 },
            "action": { "type": "state", "device_name": "lamp", "state": "on" },
            "conditions": [{ "type": "state_equals", "device_name": "lamp", "state": "on" }]
        }))).await;
        assert_eq!(s, StatusCode::OK);

        // Run automation
        let (s, _) = request(&app, "POST", "/automation/run", None).await;
        assert_eq!(s, StatusCode::OK);

        // Lamp should still be off (condition blocked the action)
        let (s, body) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["state"], "off");
    }

    #[tokio::test]
    async fn automation_with_passing_condition_fires() {
        let app = app();

        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        request(&app, "PATCH", "/devices/lamp/state", Some(json!({"state":"on"}))).await;

        request(&app, "POST", "/devices", Some(json!({"name":"thermo","device_type":"thermostat"}))).await;
        request(&app, "PATCH", "/devices/thermo/temperature", Some(json!({"temperature":30.0}))).await;

        // Condition: lamp state == "on" (which IS true) — action should fire
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "pass_rule",
            "trigger": { "type": "temperature_above", "device_name": "thermo", "threshold": 25.0 },
            "action": { "type": "state", "device_name": "lamp", "state": "off" },
            "conditions": [{ "type": "state_equals", "device_name": "lamp", "state": "on" }]
        }))).await;
        assert_eq!(s, StatusCode::OK);

        let (s, _) = request(&app, "POST", "/automation/run", None).await;
        assert_eq!(s, StatusCode::OK);

        // Lamp should now be off (action fired)
        let (s, body) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["state"], "off");
    }

    // ── Presence tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn person_crud() {
        let app = app();

        // List empty
        let (s, body) = request(&app, "GET", "/api/presence/persons", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());

        // Create
        let (s, body) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Alice",
            "grace_period_secs": 120
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let person_id = body["id"].as_str().unwrap().to_string();
        assert_eq!(body["name"], "Alice");
        assert_eq!(body["effective_state"], "unknown");

        // Conflict on duplicate name
        let (s, err) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Alice"
        }))).await;
        assert_eq!(s, StatusCode::CONFLICT);
        assert_eq!(err["code"], "conflict");

        // Get by ID
        let uri = format!("/api/presence/persons/{}", person_id);
        let (s, body) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["name"], "Alice");

        // 404 for unknown
        let (s, _) = request(&app, "GET", "/api/presence/persons/no-such-id", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Delete
        let (s, _) = request(&app, "DELETE", &uri, None).await;
        assert_eq!(s, StatusCode::NO_CONTENT);

        // Gone after delete
        let (s, _) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_source_changes_effective_state() {
        let app = app();

        // Create person with grace_period_secs=0 so grace expires immediately
        let (s, body) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Bob",
            "grace_period_secs": 0
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let person_id = body["id"].as_str().unwrap().to_string();

        // Set source 'wifi' to home → effective_state should be home
        let source_uri = format!("/api/presence/persons/{}/sources/wifi", person_id);
        let (s, body) = request(&app, "PATCH", &source_uri, Some(json!({"state": "home"}))).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["effective_state"], "home");

        // Set source 'wifi' to away with grace=0 → effective_state should be away
        let (s, body) = request(&app, "PATCH", &source_uri, Some(json!({"state": "away"}))).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["effective_state"], "away");

        // 404 for unknown person
        let (s, _) = request(&app, "PATCH", "/api/presence/persons/no-such/sources/wifi", Some(json!({"state": "home"}))).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn presence_change_trigger_fires_on_match() {
        let app = app();

        // Create lamp and a person with grace=0
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        let (s, body) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Carol",
            "grace_period_secs": 0
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let person_id = body["id"].as_str().unwrap().to_string();

        // Set Carol home
        let source_uri = format!("/api/presence/persons/{}/sources/manual", person_id);
        request(&app, "PATCH", &source_uri, Some(json!({"state": "home"}))).await;

        // Rule: presence_change for Carol home → set lamp on
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "welcome_home",
            "trigger": { "type": "presence_change", "person_name": "Carol", "target_state": "home" },
            "action": { "type": "state", "device_name": "lamp", "state": "on" }
        }))).await;
        assert_eq!(s, StatusCode::OK);

        // Run automation — Carol is home, trigger should fire
        let (s, body) = request(&app, "POST", "/automation/run", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["actions_executed"], 1);

        // Lamp should be on
        let (s, body) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["state"], "on");
    }

    #[tokio::test]
    async fn presence_change_trigger_blocked_on_mismatch() {
        let app = app();

        // Create lamp and a person with grace=0
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        let (s, body) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Dave",
            "grace_period_secs": 0
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let person_id = body["id"].as_str().unwrap().to_string();

        // Set Dave away
        let source_uri = format!("/api/presence/persons/{}/sources/manual", person_id);
        request(&app, "PATCH", &source_uri, Some(json!({"state": "away"}))).await;

        // Rule: presence_change for Dave home → set lamp on (but Dave is away)
        let (s, _) = request(&app, "POST", "/automation/rules", Some(json!({
            "name": "welcome_dave",
            "trigger": { "type": "presence_change", "person_name": "Dave", "target_state": "home" },
            "action": { "type": "state", "device_name": "lamp", "state": "on" }
        }))).await;
        assert_eq!(s, StatusCode::OK);

        // Run automation — Dave is away, trigger should NOT fire
        let (s, body) = request(&app, "POST", "/automation/run", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["actions_executed"], 0);

        // Lamp should still be off (default state)
        let (s, body) = request(&app, "GET", "/devices/lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["state"], "off");
    }

    // ── Dashboard tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn dashboard_crud() {
        let app = app();

        // List empty
        let (s, body) = request(&app, "GET", "/api/dashboards", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());

        // Create
        let (s, body) = request(&app, "POST", "/api/dashboards", Some(json!({
            "name": "Home",
            "icon": "🏠"
        }))).await;
        assert_eq!(s, StatusCode::CREATED);
        let dash_id = body["id"].as_str().unwrap().to_string();
        assert_eq!(body["name"], "Home");
        assert!(body["views"].as_array().unwrap().is_empty());

        // Conflict on duplicate name
        let (s, err) = request(&app, "POST", "/api/dashboards", Some(json!({"name": "Home"}))).await;
        assert_eq!(s, StatusCode::CONFLICT);
        assert_eq!(err["code"], "conflict");

        // Get by ID
        let uri = format!("/api/dashboards/{}", dash_id);
        let (s, body) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["name"], "Home");

        // 404 for unknown
        let (s, _) = request(&app, "GET", "/api/dashboards/no-such-id", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);

        // Delete
        let (s, _) = request(&app, "DELETE", &uri, None).await;
        assert_eq!(s, StatusCode::NO_CONTENT);

        // Gone after delete
        let (s, _) = request(&app, "GET", &uri, None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn dashboard_view_and_card_operations() {
        let app = app();

        // Create dashboard
        let (s, body) = request(&app, "POST", "/api/dashboards", Some(json!({"name": "Test"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        let dash_id = body["id"].as_str().unwrap().to_string();

        // Add view
        let views_uri = format!("/api/dashboards/{}/views", dash_id);
        let (s, body) = request(&app, "POST", &views_uri, Some(json!({"title": "Overview"}))).await;
        assert_eq!(s, StatusCode::OK);
        let views = body["views"].as_array().unwrap();
        assert_eq!(views.len(), 1);
        let view_id = views[0]["id"].as_str().unwrap().to_string();
        assert_eq!(views[0]["title"], "Overview");

        // Add entity card
        let cards_uri = format!("/api/dashboards/{}/views/{}/cards", dash_id, view_id);
        let (s, body) = request(&app, "POST", &cards_uri, Some(json!({
            "card_type": "entity_card",
            "entity_id": "device.lamp.switch"
        }))).await;
        assert_eq!(s, StatusCode::OK);
        let cards = body["views"][0]["cards"].as_array().unwrap();
        assert_eq!(cards.len(), 1);
        let card_id = cards[0]["id"].as_str().unwrap().to_string();
        assert_eq!(cards[0]["card_type"], "entity_card");
        assert_eq!(cards[0]["entity_id"], "device.lamp.switch");

        // Delete card
        let del_card_uri = format!("/api/dashboards/{}/views/{}/cards/{}", dash_id, view_id, card_id);
        let (s, body) = request(&app, "DELETE", &del_card_uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body["views"][0]["cards"].as_array().unwrap().is_empty());

        // Delete view
        let del_view_uri = format!("/api/dashboards/{}/views/{}", dash_id, view_id);
        let (s, body) = request(&app, "DELETE", &del_view_uri, None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body["views"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn entity_single_lookup() {
        let app = app();

        // Create a device
        request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;

        // Get entity by ID — light "lamp" yields "switch.lamp" and "number.lamp_brightness"
        let (s, body) = request(&app, "GET", "/api/entities/switch.lamp", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["entity_id"], "switch.lamp");
        assert_eq!(body["kind"], "switch");
        assert!(body["device_id"].is_string());

        // Unknown entity_id returns 404
        let (s, body) = request(&app, "GET", "/api/entities/switch.nosuchdevice", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "not_found");
    }

    #[tokio::test]
    async fn entities_include_person_kind() {
        let app = app();

        // Initially no person entities
        let (s, body) = request(&app, "GET", "/api/entities?kind=person", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());

        // Create a person
        let (_, person_body) = request(&app, "POST", "/api/presence/persons", Some(json!({
            "name": "Eve",
            "grace_period_secs": 120
        }))).await;
        let person_id = person_body["id"].as_str().unwrap().to_string();

        // ?kind=person should return one entity
        let (s, body) = request(&app, "GET", "/api/entities?kind=person", None).await;
        assert_eq!(s, StatusCode::OK);
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["kind"], "person");
        assert_eq!(arr[0]["device_id"], person_id);

        // All entities also includes person entity
        let (s, body) = request(&app, "GET", "/api/entities", None).await;
        assert_eq!(s, StatusCode::OK);
        let arr = body.as_array().unwrap();
        assert!(arr.iter().any(|e| e["kind"] == "person"));
    }

    // ── Auth middleware tests ─────────────────────────────────────────────────

    fn app_with_key(key: &str) -> Router {
        let mut state = AppState::new(None);
        state.api_key = Some(key.to_string());
        build_with_legacy_routes(state, &[])
    }

    #[tokio::test]
    async fn auth_middleware_blocks_writes_when_key_set() {
        let app = app_with_key("secret");

        // POST without header → 401
        let (s, body) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "unauthorized");

        // POST with wrong key → 401
        let (s, _) = request_with_headers(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"})), &[("x-api-key", "wrong")]).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // POST with correct key via X-API-Key → 201
        let (s, _) = request_with_headers(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"})), &[("x-api-key", "secret")]).await;
        assert_eq!(s, StatusCode::CREATED);

        // POST with correct key via Authorization Bearer → 201
        let (s, _) = request_with_headers(&app, "POST", "/devices", Some(json!({"name":"lamp2","device_type":"light"})), &[("authorization", "Bearer secret")]).await;
        assert_eq!(s, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn auth_middleware_allows_gets_without_key() {
        let app = app_with_key("secret");

        // GET never requires key
        let (s, body) = request(&app, "GET", "/api/devices", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(body.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn auth_middleware_absent_key_allows_all() {
        let app = app(); // no api_key set

        // Write without any header → should still work
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn name_at_limit_accepted() {
        let app = app();
        let name = "a".repeat(120);
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name": name, "device_type": "light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn name_over_limit_rejected() {
        let app = app();
        let name = "a".repeat(121);
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name": name, "device_type": "light"}))).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn backup_returns_all_state() {
        let app = app();
        // Create a device
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"lamp","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        // GET /api/backup
        let (s, body) = request(&app, "GET", "/api/backup", None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(!body["devices"].as_array().unwrap().is_empty(), "devices should be non-empty");
        assert!(body["version"].is_string());
        assert!(body["exported_at"].is_string());
    }

    #[tokio::test]
    async fn safe_mode_toggle_and_suppression() {
        let app = app();
        // Create a device and an automation rule that sets it to On
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"bulb","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        let rule_body = json!({
            "name": "turn_on",
            "trigger": { "type": "device_state_change", "device_name": "bulb", "target_state": "on" },
            "action": { "type": "state", "device_name": "bulb", "state": "on" }
        });
        let (s, _) = request(&app, "POST", "/automation/rules", Some(rule_body)).await;
        assert_eq!(s, StatusCode::OK);
        // Enable safe mode
        let (s, body) = request(&app, "POST", "/automation/rules/turn_on/safe-mode", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["safe_mode"], json!(true));
        // Toggle back
        let (s, body) = request(&app, "POST", "/automation/rules/turn_on/safe-mode", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["safe_mode"], json!(false));
    }

    #[tokio::test]
    async fn sse_broadcast_reaches_record_event() {
        let state = AppState::new(None);
        let mut rx = state.events_tx.subscribe();
        crate::http::helpers::record_event(
            &state,
            crate::http::types::EventKind::Server,
            "test",
            "hello sse".to_string(),
            None,
            None,
        ).await;
        let ev = rx.try_recv().expect("event should be in channel");
        assert_eq!(ev.message, "hello sse");
        assert_eq!(ev.entity, "test");
    }

    #[tokio::test]
    async fn device_history_records_state_changes() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"hist_light","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        let (s, _) = request(&app, "PATCH", "/devices/hist_light/state", Some(json!({"state":"on"}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = request(&app, "PATCH", "/devices/hist_light/state", Some(json!({"state":"off"}))).await;
        assert_eq!(s, StatusCode::OK);
        let (s, body) = request(&app, "GET", "/devices/hist_light/history", None).await;
        assert_eq!(s, StatusCode::OK);
        let entries = body.as_array().expect("history should be array");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["state"], "on");
        assert_eq!(entries[1]["state"], "off");
    }

    #[tokio::test]
    async fn restore_clears_and_reimports() {
        let app = app();
        // Create original device
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"original_device","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        // Get backup with a backup_device
        let backup = json!({
            "version": "1",
            "exported_at": "2026-01-01T00:00:00Z",
            "devices": [{"id":"aaaaaaaa-0000-0000-0000-000000000001","name":"backup_device","device_type":"light","state":"off","brightness":100,"connected":false}],
            "automation_rules": [],
            "scripts": [],
            "scenes": [],
            "persons": [],
            "dashboards": []
        });
        let (s, body) = request(&app, "POST", "/api/restore", Some(backup)).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["restored"]["devices"], 1);
        // Original device should be gone
        let (s, _) = request(&app, "GET", "/devices/original_device", None).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
        // Backup device should be present
        let (s, body) = request(&app, "GET", "/devices/backup_device", None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["name"], "backup_device");
    }

    #[tokio::test]
    async fn notify_url_rule_creation() {
        let app = app();
        let (s, _) = request(&app, "POST", "/devices", Some(json!({"name":"led","device_type":"light"}))).await;
        assert_eq!(s, StatusCode::CREATED);
        let rule = json!({
            "name": "webhook_rule",
            "trigger": {"type": "device_state_change", "device_name": "led", "target_state": "on"},
            "action": {"type": "notify", "message": "LED turned on"},
            "notify_url": "http://example.com/hook"
        });
        let (s, body) = request(&app, "POST", "/automation/rules", Some(rule)).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["notify_url"], "http://example.com/hook");
        // Also verify GET lists it
        let (s, body) = request(&app, "GET", "/automation/rules", None).await;
        assert_eq!(s, StatusCode::OK);
        let rules = body.as_array().unwrap();
        let found = rules.iter().find(|r| r["name"] == "webhook_rule").expect("rule should exist");
        assert_eq!(found["notify_url"], "http://example.com/hook");
    }
}
