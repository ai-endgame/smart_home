## ADDED Requirements

### Requirement: HTTP state change publishes MQTT command
When a device's `state`, `brightness`, or `temperature` is updated via the HTTP API and an MQTT client is connected, the server SHALL publish a JSON payload to `zigbee2mqtt/{name}/set` with only the changed fields.

#### Scenario: Toggle state publishes set command
- **WHEN** `POST /api/devices/living-room-light/toggle` is called and MQTT is connected
- **THEN** `{"state":"ON"}` or `{"state":"OFF"}` is published to `zigbee2mqtt/living-room-light/set`

#### Scenario: Update device publishes changed fields
- **WHEN** `PUT /api/devices/living-room-light` sets `brightness=50` and MQTT is connected
- **THEN** `{"brightness":50}` is published to `zigbee2mqtt/living-room-light/set`

#### Scenario: Publish is fire-and-forget
- **WHEN** the MQTT broker is unreachable at publish time
- **THEN** the HTTP response still returns 200; the error is logged but not surfaced to the caller

### Requirement: No publish when MQTT not configured
When no MQTT client is present in `AppState`, state-change HTTP handlers SHALL NOT attempt any MQTT publish.

#### Scenario: MQTT_URL absent — no publish attempt
- **WHEN** `MQTT_URL` is not set and `PUT /api/devices/lamp/update` is called
- **THEN** no MQTT publish is attempted and the handler returns normally
