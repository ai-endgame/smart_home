use axum::{Router, routing::{delete, get, patch, post}};

use crate::http::{
    handlers::{automation, devices, discovery, system},
    middleware::cors_layer,
};
use crate::state::AppState;

/// Full router: `/api/*` prefixed routes + system routes.
pub fn build(state: AppState, cors_origins: &[String]) -> Router {
    Router::new()
        // System
        .route("/health", get(system::health))
        .route("/status", get(system::status))
        .route("/server/stop", post(system::stop_server))
        .route("/events", get(system::list_events))
        .route("/clients", get(system::list_clients))
        .route("/clients/connect", post(system::connect_client))
        .route("/clients/{client_id}/disconnect", post(system::disconnect_client))
        // Devices (prefixed)
        .route("/api/devices", get(devices::list_devices).post(devices::create_device))
        .route("/api/devices/{name}", get(devices::get_device).delete(devices::remove_device).patch(devices::update_device))
        .route("/api/devices/{name}/state", patch(devices::set_device_state))
        .route("/api/devices/{name}/brightness", patch(devices::set_device_brightness))
        .route("/api/devices/{name}/temperature", patch(devices::set_device_temperature))
        .route("/api/devices/{name}/commands", post(devices::send_device_command))
        .route("/api/devices/{name}/connect", post(devices::connect_device))
        .route("/api/devices/{name}/disconnect", post(devices::disconnect_device))
        .route("/api/devices/{name}/error", post(devices::report_device_error))
        .route("/api/devices/{name}/error/clear", post(devices::clear_device_error))
        .route("/api/devices/{name}/events", get(devices::list_device_events))
        // Automation (prefixed)
        .route("/api/automation/rules", get(automation::list_rules).post(automation::add_rule))
        .route("/api/automation/rules/{name}", delete(automation::remove_rule))
        .route("/api/automation/rules/{name}/toggle", post(automation::toggle_rule))
        .route("/api/automation/run", post(automation::run_automation))
        // Discovery (prefixed)
        .route("/api/discovery/devices", get(discovery::list_discovered))
        .route("/api/discovery/devices/add", post(discovery::add_discovered_device))
        // Legacy unprefixed aliases — backward compat with tests and old ui.html
        .route("/devices", get(devices::list_devices).post(devices::create_device))
        .route("/devices/{name}", get(devices::get_device).delete(devices::remove_device).patch(devices::update_device))
        .route("/devices/{name}/state", patch(devices::set_device_state))
        .route("/devices/{name}/brightness", patch(devices::set_device_brightness))
        .route("/devices/{name}/temperature", patch(devices::set_device_temperature))
        .route("/devices/{name}/commands", post(devices::send_device_command))
        .route("/devices/{name}/connect", post(devices::connect_device))
        .route("/devices/{name}/disconnect", post(devices::disconnect_device))
        .route("/devices/{name}/error", post(devices::report_device_error))
        .route("/devices/{name}/error/clear", post(devices::clear_device_error))
        .route("/devices/{name}/events", get(devices::list_device_events))
        .route("/automation/rules", get(automation::list_rules).post(automation::add_rule))
        .route("/automation/rules/{name}", delete(automation::remove_rule))
        .route("/automation/rules/{name}/toggle", post(automation::toggle_rule))
        .route("/automation/run", post(automation::run_automation))
        .route("/discovery/devices", get(discovery::list_discovered))
        .route("/discovery/devices/add", post(discovery::add_discovered_device))
        .layer(cors_layer(cors_origins))
        .with_state(state)
}

/// Alias kept for test compatibility.
pub fn build_with_legacy_routes(state: AppState, cors_origins: &[String]) -> Router {
    build(state, cors_origins)
}
