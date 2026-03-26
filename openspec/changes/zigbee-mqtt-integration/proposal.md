## Why

Session 3 of the mastery plan teaches Zigbee through Zigbee2MQTT: a coordinator bridges Zigbee radio to MQTT, devices publish state to `zigbee2mqtt/{name}` and receive commands via `zigbee2mqtt/{name}/set`. The codebase currently has no MQTT support — device state only flows through HTTP. Without MQTT, none of the Zigbee mesh concepts are visible in running code. Adding MQTT turns the theory into a live bridge the student can observe.

## What Changes

- Add `rumqttc` as an async MQTT client dependency
- Add `MQTT_URL` to `Config` (e.g. `mqtt://localhost:1883`); MQTT is optional — server starts fine without it
- Add `ZigbeeRole` enum to `Device`: `Coordinator`, `Router`, `EndDevice` — models Zigbee mesh topology
- Add `backend/src/infrastructure/mqtt.rs` — async subscriber loop that:
  - Subscribes to `zigbee2mqtt/#`
  - On state messages (`zigbee2mqtt/{name}`): parses JSON payload and syncs `state`, `brightness`, `temperature` into `AppState.home`
  - On bridge messages (`zigbee2mqtt/bridge/devices`): pushes newly-seen devices into `DiscoveryStore` as auto-discovered
  - On `linkquality` field: stores as device attribute (mesh signal strength)
- Add MQTT command publishing: when a device state/brightness/temperature is updated via HTTP, also publish the corresponding `zigbee2mqtt/{name}/set` JSON if an MQTT client is connected
- Expose `GET /api/mqtt/status` — connection state, broker URL (redacted), topics received, last message timestamp
- Add `mqtt_status` to `AppState` as `Arc<RwLock<MqttStatus>>`
- Update `Makefile` with a `mqtt-broker` target that starts a Mosquitto container for local dev

## Capabilities

### New Capabilities

- `mqtt-bridge`: MQTT subscriber loop, `zigbee2mqtt/#` topic handling, state sync into home, discovery push, `ZigbeeRole` on Device
- `mqtt-command-publish`: Publish `zigbee2mqtt/{name}/set` when device state/brightness/temperature changes via HTTP API
- `mqtt-status-endpoint`: `GET /api/mqtt/status` — broker connection info, message stats

### Modified Capabilities

- None — MQTT is purely additive; no existing HTTP routes change

## Impact

- `backend/Cargo.toml` — add `rumqttc` dependency
- `backend/src/config.rs` — add `mqtt_url: Option<String>`
- `backend/src/domain/device.rs` — add `ZigbeeRole` enum; add `zigbee_role: Option<ZigbeeRole>` to `Device`
- `backend/src/infrastructure/mqtt.rs` — new module (subscriber loop + publish helpers)
- `backend/src/infrastructure/mod.rs` — add `pub mod mqtt`
- `backend/src/state.rs` — add `mqtt_status: Arc<RwLock<MqttStatus>>`, `mqtt_client: Option<AsyncClient>`
- `backend/src/http/mod.rs` — start MQTT loop if `mqtt_url` configured
- `backend/src/http/handlers/mqtt.rs` — new handler for status endpoint
- `backend/src/http/router.rs` — register `GET /api/mqtt/status`
- `contracts/openapi.yaml` — add `MqttStatus`, `ZigbeeRole` schemas and new path
- `Makefile` — add `mqtt-broker` target (Mosquitto container) and `run-server-mqtt` target
- `frontend/lib/api/types.ts` — add `MqttStatus`, `ZigbeeRole` types; add `zigbee_role` to `Device`
- `frontend/app/ecosystem/page.tsx` — show MQTT connection badge
