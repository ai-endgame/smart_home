use axum::{Json, extract::State};

use crate::http::types::MqttStatusResponse;
use crate::state::AppState;

pub async fn get_mqtt_status(State(state): State<AppState>) -> Json<MqttStatusResponse> {
    let status = state.mqtt_status.read().await;
    Json(MqttStatusResponse {
        connected: status.connected,
        broker: status.broker.clone(),
        topics_received: status.topics_received,
        last_message_at: status.last_message_at.clone(),
    })
}
