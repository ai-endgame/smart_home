## 1. Dependencies & Config

- [x] 1.1 Add `rumqttc = { version = "0.24", features = ["use-rustls"] }` to `backend/Cargo.toml`
- [x] 1.2 Add `mqtt_url: Option<String>` to `Config` in `backend/src/config.rs`, read from `MQTT_URL` env var

## 2. Domain — ZigbeeRole

- [x] 2.1 Add `ZigbeeRole` enum to `backend/src/domain/device.rs` with variants `Coordinator`, `Router`, `EndDevice`; derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`
- [x] 2.2 Add `Display` impl for `ZigbeeRole` returning lowercase string (`coordinator`, `router`, `end_device`)
- [x] 2.3 Add `zigbee_role: Option<ZigbeeRole>` field to `Device` struct
- [x] 2.4 Export `ZigbeeRole` via `pub use` in `backend/src/domain/mod.rs`

## 3. Infrastructure — DB

- [x] 3.1 Add `ADD COLUMN IF NOT EXISTS zigbee_role TEXT` to `db.rs` `migrate()`
- [x] 3.2 Update `load_all_devices` to read `zigbee_role` column and parse via `serde_json::from_str` or match string
- [x] 3.3 Update `upsert_device` to persist `zigbee_role` as nullable TEXT

## 4. Infrastructure — MQTT Module

- [x] 4.1 Create `backend/src/infrastructure/mqtt.rs`
- [x] 4.2 Add `MqttStatus` struct: `connected: bool`, `broker: Option<String>`, `topics_received: u64`, `last_message_at: Option<String>`; derive `Serialize`, `Clone`, `Default`
- [x] 4.3 Implement `redact_url(url: &str) -> String` — strips userinfo and replaces port with `***`
- [x] 4.4 Implement `start_mqtt_loop(mqtt_url: &str, client: AsyncClient, event_loop: EventLoop, state: AppState)` async fn that:
  - Subscribes to `zigbee2mqtt/#` with QoS 0
  - Spawns `tokio::spawn` for the event loop
  - Handles `Incoming::Publish` messages in a loop
- [x] 4.5 Implement state message handler: parse topic `zigbee2mqtt/{name}` (not bridge subtopics), parse JSON payload, update `state`/`brightness`/`temperature`/`attributes.linkquality` on matching device in `home`
- [x] 4.6 Implement bridge devices handler: parse `zigbee2mqtt/bridge/devices` JSON array, push new device names into `DiscoveryStore` with `zigbee_role` from `type` field
- [x] 4.7 Update `MqttStatus.topics_received` and `last_message_at` on every incoming publish
- [x] 4.8 Implement `publish_command(client: &AsyncClient, name: &str, patch: serde_json::Value)` async fn — publishes to `zigbee2mqtt/{name}/set` with QoS 0, logs error on failure
- [x] 4.9 Add `pub mod mqtt;` to `backend/src/infrastructure/mod.rs`

## 5. State

- [x] 5.1 Add `mqtt_status: Arc<RwLock<MqttStatus>>` to `AppState` in `backend/src/state.rs`
- [x] 5.2 Add `mqtt_client: Option<AsyncClient>` to `AppState`
- [x] 5.3 Update `AppState::new()` / builder to initialize both fields (status = default, client = None)

## 6. HTTP — Server Init

- [x] 6.1 In `backend/src/http/mod.rs` `run_server_full()`: if `config.mqtt_url` is `Some`, create `AsyncClient`, store in `AppState.mqtt_client`, call `start_mqtt_loop`

## 7. HTTP — Command Publish Integration

- [x] 7.1 In `backend/src/http/handlers/devices.rs` toggle handler: after state update, if `state.mqtt_client.is_some()` call `publish_command` with `{"state": <new_state>}`
- [x] 7.2 In `backend/src/http/handlers/devices.rs` update handler: after state update, build patch JSON from changed fields and call `publish_command` if MQTT client present

## 8. HTTP — Status Endpoint

- [x] 8.1 Create `backend/src/http/handlers/mqtt.rs` with `get_mqtt_status` handler returning `AppState.mqtt_status` as JSON
- [x] 8.2 Add `pub mod mqtt;` to `backend/src/http/handlers/mod.rs`
- [x] 8.3 Register `GET /api/mqtt/status` (and legacy `/mqtt/status`) in `backend/src/http/router.rs`

## 9. HTTP Types

- [x] 9.1 Add `MqttStatusResponse` DTO to `backend/src/http/types.rs`: `connected`, `broker`, `topics_received`, `last_message_at`
- [x] 9.2 Add `zigbee_role: Option<ZigbeeRole>` to `DeviceResponse` in `http/types.rs`
- [x] 9.3 Update `device_to_response()` in `helpers.rs` to populate `zigbee_role`

## 10. OpenAPI Contract

- [x] 10.1 Add `ZigbeeRole` schema (enum) to `contracts/openapi.yaml`
- [x] 10.2 Add `MqttStatus` schema to `contracts/openapi.yaml`
- [x] 10.3 Add `GET /api/mqtt/status` path to `contracts/openapi.yaml`
- [x] 10.4 Add `zigbee_role` field to `Device` schema in `contracts/openapi.yaml`

## 11. Makefile

- [x] 11.1 Add `mqtt-broker` target: `docker run -d --name mosquitto -p 1883:1883 eclipse-mosquitto`
- [x] 11.2 Add `run-server-mqtt` target: starts with both `DATABASE_URL` and `MQTT_URL=mqtt://localhost:1883`

## 12. Frontend

- [x] 12.1 Add `MqttStatus` and `ZigbeeRole` types to `frontend/lib/api/types.ts`; add `zigbee_role?: ZigbeeRole` to `Device`
- [x] 12.2 Create `frontend/lib/api/mqtt.ts` with `getMqttStatus()` fetch wrapper
- [x] 12.3 Create `frontend/app/ecosystem/page.tsx` update (or create if not present) — add MQTT connection badge using `getMqttStatus()`

## 13. Tests

- [x] 13.1 Add unit test for `redact_url` — verifies credentials and port are stripped
- [x] 13.2 Add unit test for `ZigbeeRole` Display impl
- [x] 13.3 Add integration test: `GET /api/mqtt/status` with no MQTT configured returns `{"connected":false,...}`
