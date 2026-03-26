## ADDED Requirements

### Requirement: GET /api/mqtt/status returns connection info
The server SHALL expose `GET /api/mqtt/status` returning a JSON object with MQTT connection state, a redacted broker URL, total topics received, and the timestamp of the last message.

#### Scenario: MQTT connected
- **WHEN** `GET /api/mqtt/status` is called with a live broker connection
- **THEN** response is `200` with `{"connected":true,"broker":"mqtt://localhost:***","topics_received":N,"last_message_at":"<ISO8601>"}`

#### Scenario: MQTT not configured
- **WHEN** `GET /api/mqtt/status` is called with no `MQTT_URL`
- **THEN** response is `200` with `{"connected":false,"broker":null,"topics_received":0,"last_message_at":null}`

#### Scenario: Broker URL redaction
- **WHEN** `MQTT_URL=mqtt://user:secret@broker.local:1883` is configured
- **THEN** the `broker` field in the response omits credentials (e.g. `"mqtt://broker.local:***"`)

### Requirement: MqttStatus tracked in AppState
`AppState` SHALL contain `mqtt_status: Arc<RwLock<MqttStatus>>`. The subscriber loop SHALL update `topics_received` on every message and `last_message_at` on every message.

#### Scenario: topics_received increments per message
- **WHEN** 5 messages arrive on `zigbee2mqtt/#`
- **THEN** `GET /api/mqtt/status` returns `topics_received >= 5`
