## ADDED Requirements

### Requirement: MQTT subscriber loop starts on boot
When `MQTT_URL` is configured, the server SHALL start an async subscriber loop that connects to the broker and subscribes to `zigbee2mqtt/#` before handling requests.

#### Scenario: Loop starts with valid URL
- **WHEN** server starts with `MQTT_URL=mqtt://localhost:1883`
- **THEN** a connection to the broker is established and `zigbee2mqtt/#` is subscribed

#### Scenario: Server starts without MQTT_URL
- **WHEN** `MQTT_URL` is not set in the environment
- **THEN** the server starts normally with no MQTT connection and no errors logged

### Requirement: State messages sync device fields
On receiving a `zigbee2mqtt/{name}` topic, the loop SHALL parse the JSON payload and update `state`, `brightness`, and `temperature` fields on the matching device in `AppState.home`.

#### Scenario: State message updates existing device
- **WHEN** a message arrives on `zigbee2mqtt/living-room-light` with `{"state":"ON","brightness":80}`
- **THEN** the device named `living-room-light` has `state=On` and `brightness=80` in `AppState.home`

#### Scenario: State message for unknown device is silently ignored
- **WHEN** a message arrives on `zigbee2mqtt/unknown-device` with a valid JSON payload
- **THEN** no error is returned; the loop continues processing

#### Scenario: linkquality stored in attributes
- **WHEN** a state message includes `{"linkquality": 127}`
- **THEN** the device's `attributes` JSON value contains `{"linkquality": 127}`

### Requirement: Bridge devices message pushes to DiscoveryStore
On receiving `zigbee2mqtt/bridge/devices`, the loop SHALL iterate the JSON array and push any device names not already in `AppState.home` into `DiscoveryStore` as auto-discovered entries.

#### Scenario: New Zigbee device appears in bridge/devices
- **WHEN** `zigbee2mqtt/bridge/devices` arrives with a device named `temp-sensor-1`
- **THEN** `temp-sensor-1` appears in `GET /api/discovery` as a discovered device

#### Scenario: Already-home device not duplicated in discovery
- **WHEN** `zigbee2mqtt/bridge/devices` lists a device that already exists in `AppState.home`
- **THEN** `DiscoveryStore` is not modified for that device

### Requirement: ZigbeeRole on Device
The `Device` struct SHALL have a `zigbee_role: Option<ZigbeeRole>` field with variants `Coordinator`, `Router`, `EndDevice`. The `devices` table SHALL store this as a nullable `zigbee_role TEXT` column.

#### Scenario: ZigbeeRole set from bridge/devices
- **WHEN** `zigbee2mqtt/bridge/devices` includes `{"type":"Router","friendly_name":"plug-1"}`
- **THEN** the discovered device entry has `zigbee_role=Router`

#### Scenario: Device without Zigbee role
- **WHEN** a device is created via HTTP with no `zigbee_role`
- **THEN** `zigbee_role` is `null` in the API response
