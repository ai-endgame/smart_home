## ADDED Requirements

### Requirement: HistoryEntry domain type
The system SHALL define a `HistoryEntry` struct in `backend/src/domain/device.rs` with fields: `timestamp: DateTime<Utc>`, `state: DeviceState`, `brightness: u8`, `temperature: Option<f64>`. It SHALL derive `Clone`, `Serialize`, `Deserialize`.

#### Scenario: HistoryEntry serializes to JSON
- **WHEN** a `HistoryEntry` is serialized
- **THEN** it produces `{"timestamp":"...","state":"...","brightness":0,"temperature":null}` in ISO 8601 format

### Requirement: Per-device state history registry in AppState
`AppState` SHALL contain `history: Arc<RwLock<HashMap<String, VecDeque<HistoryEntry>>>>`. The constant `MAX_HISTORY_PER_DEVICE = 500` SHALL be defined in `state.rs`. When the ring buffer for a device reaches 500 entries, the oldest entry SHALL be removed before inserting the new one.

#### Scenario: History capped at 500 entries
- **WHEN** 501 state changes are recorded for a device
- **THEN** `GET /api/devices/{name}/history` returns exactly 500 entries, the oldest having been evicted

### Requirement: History recorded on every device state change
`http/helpers.rs` SHALL expose `record_history(state, device_name, entry)`. This function SHALL be called in `set_device_state`, `set_device_brightness`, `set_device_temperature` handlers immediately after the state mutation succeeds. The entry SHALL capture the device's new state, brightness, and temperature at the time of the change.

#### Scenario: State change appends history
- **WHEN** `PATCH /api/devices/{name}/state` succeeds
- **THEN** `GET /api/devices/{name}/history` returns one more entry than before, with the new state

### Requirement: GET /api/devices/{name}/history endpoint
The system SHALL expose `GET /api/devices/{name}/history` that returns the device's history as a JSON array of `HistoryEntry`, ordered oldest-to-newest. If the device does not exist, the endpoint SHALL return 404. An optional query parameter `limit` SHALL cap the response to the most recent N entries.

#### Scenario: History for existing device
- **WHEN** `GET /api/devices/lamp/history` is called after several state changes
- **THEN** the response is `200` with an array of `HistoryEntry` objects in chronological order

#### Scenario: History for non-existent device
- **WHEN** `GET /api/devices/nonexistent/history` is called
- **THEN** the response is `404`

#### Scenario: History with limit parameter
- **WHEN** `GET /api/devices/lamp/history?limit=5` is called
- **THEN** the response contains at most 5 entries, the 5 most recent
