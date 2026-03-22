## Context

The backend currently has no MQTT support — all device state flows through HTTP only. Session 3 of the mastery plan introduces Zigbee2MQTT: a coordinator bridges Zigbee radio to MQTT, publishing state to `zigbee2mqtt/{name}` and receiving commands via `zigbee2mqtt/{name}/set`. Without MQTT, Zigbee mesh concepts remain invisible in running code.

The existing infrastructure layer has two adapters: `db.rs` (SQLx/Postgres) and `mdns.rs` (mDNS via `std::thread`). MQTT fits the same pattern: a new `mqtt.rs` module in `infrastructure/` that runs its own async task and writes into shared `AppState`.

## Goals / Non-Goals

**Goals:**
- Subscribe to `zigbee2mqtt/#` and sync state into `AppState.home` in real time
- Publish `zigbee2mqtt/{name}/set` when device state changes via HTTP
- Expose `GET /api/mqtt/status` for observability
- Add `ZigbeeRole` to `Device` to model mesh topology
- Keep MQTT optional — server starts without `MQTT_URL`, all existing tests unaffected

**Non-Goals:**
- Full Zigbee pairing/commissioning UI
- MQTT authentication / TLS (local dev only for now)
- Persistent MQTT message history
- Multiple MQTT brokers

## Decisions

### rumqttc as the MQTT client

**Decision**: Use `rumqttc` (async variant — `AsyncClient` + `EventLoop`).

**Why**: `rumqttc` is the de-facto standard async MQTT client in the Rust ecosystem. Its `AsyncClient` is `Clone + Send + Sync`, making it trivial to pass into `AppState` and use from Axum handlers. The `EventLoop` runs in a dedicated `tokio::spawn` loop, matching the mDNS pattern in `mdns.rs`.

**Alternatives considered**:
- `paho-mqtt` — C FFI, harder to cross-compile; no ergonomic async API
- `mqtt-async-client` — less maintained, fewer MQTT 3.1.1 features
- Hand-rolled TCP — not worth the complexity

### Optional MQTT via Config

**Decision**: `Config.mqtt_url: Option<String>`. When `None`, skip broker connection entirely; no `AsyncClient` stored in state. All MQTT-dependent code paths are guarded by `if let Some(client) = &state.mqtt_client`.

**Why**: Keeps CI and existing integration tests passing without a broker. Mirrors how `DATABASE_URL` is optional.

### State stored in AppState

**Decision**: Add `mqtt_status: Arc<RwLock<MqttStatus>>` and `mqtt_client: Option<AsyncClient>` to `AppState`.

**Why**: Handlers need `mqtt_client` to publish commands. The status endpoint needs `mqtt_status`. Both must be cheaply cloneable across Axum handler tasks. `Arc<RwLock<_>>` is the existing pattern (see `events`, `clients` in `state.rs`).

### Sync device state via MQTT subscription into SmartHome

**Decision**: The MQTT subscriber loop holds a write lock on `AppState.home` only for the duration of each state update (microseconds). It calls `home.get_device_mut(name)` and patches fields directly.

**Why**: `SmartHome` is already guarded by `Arc<RwLock<SmartHome>>`. Brief write locks on state messages are safe and consistent with how HTTP handlers update device state. A separate MQTT-owned device map would create drift.

### Auto-discovery push on bridge/devices messages

**Decision**: When `zigbee2mqtt/bridge/devices` arrives, iterate the JSON array and push new devices into `DiscoveryStore` (same store used by mDNS discovery).

**Why**: Reusing `DiscoveryStore` means the `/api/discovery` endpoint surfaces Zigbee devices for free with zero new UI work. Existing "Add to Home" flow works unchanged.

### Command publish on HTTP state change

**Decision**: After persisting a device state change via HTTP, call `mqtt::publish_command(client, name, patch)` which sends `zigbee2mqtt/{name}/set` with a minimal JSON payload.

**Why**: This closes the loop: HTTP in → state update → MQTT out → Zigbee device actuates. The publish is fire-and-forget (`client.publish(...).await` without blocking the response).

## Risks / Trade-offs

- **Race between MQTT and HTTP writes** → Both paths hold the `RwLock` write lock; last-writer-wins. For learning purposes this is acceptable; production would need an event-sourced model.
- **MQTT subscriber crash stops sync** → The `tokio::spawn` loop reconnects automatically via `rumqttc`'s built-in retry; worst case is a brief gap in state.
- **`linkquality` not in Device schema** → Stored in `attributes: serde_json::Value` (already on `Device`) to avoid a schema migration.
- **`zigbee_role` column not in DB** → Added as `TEXT NULL` via `ADD COLUMN IF NOT EXISTS`; safe for existing rows.

## Migration Plan

1. Add `rumqttc` to `Cargo.toml`
2. Run without `MQTT_URL` — server boots identically to today
3. Start Mosquitto via `make mqtt-broker`, then `make run-server-mqtt` to see live sync
4. Rollback: remove `MQTT_URL` from env; MQTT code is entirely behind `Option` guards

## Open Questions

- Should `ZigbeeRole` be persisted to DB? (Proposed: yes, as TEXT column on `devices` table)
- Should failed MQTT publishes surface as HTTP errors? (Proposed: no — fire-and-forget; log only)
