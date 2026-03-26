## ADDED Requirements

### Requirement: Dispatch OnOff cluster command on device state change
When a device with `control_protocol: matter` has its state toggled, the server SHALL asynchronously dispatch an `onoff on` or `onoff off` cluster command via `chip-tool` using the device's `node_id` from `matter_fabric`.

#### Scenario: Toggle Matter device on
- **WHEN** `PATCH /api/devices/{name}/state` is called with `{"state": "on"}` for a Matter device
- **THEN** server returns `200 OK` immediately and spawns `chip-tool onoff on <node_id> 1` as a background task

#### Scenario: Toggle Matter device off
- **WHEN** `PATCH /api/devices/{name}/state` is called with `{"state": "off"}` for a Matter device
- **THEN** server spawns `chip-tool onoff off <node_id> 1` as a background task

#### Scenario: chip-tool command fails
- **WHEN** the background `chip-tool` subprocess exits with non-zero code
- **THEN** `device.last_error` is updated with the error message and a `device_error` SSE event is emitted

### Requirement: Dispatch LevelControl cluster command on brightness change
When a Matter device brightness is updated, the server SHALL dispatch `chip-tool levelcontrol move-to-level` with the brightness value scaled from 0–100 to 0–254.

#### Scenario: Set brightness on Matter device
- **WHEN** `PATCH /api/devices/{name}/brightness` is called with `{"brightness": 80}` for a Matter device
- **THEN** server spawns `chip-tool levelcontrol move-to-level 203 0 0 0 <node_id> 1` (80% of 254 ≈ 203)

#### Scenario: Zero brightness maps to level 0
- **WHEN** brightness is set to 0
- **THEN** chip-tool is invoked with level 0

### Requirement: Dispatch ColorTemperatureMired cluster command
When a Matter device temperature (color temperature in mireds) is updated, the server SHALL dispatch `chip-tool colorcontrol move-to-color-temperature`.

#### Scenario: Set color temperature on Matter light
- **WHEN** `PATCH /api/devices/{name}/temperature` is called for a Matter light device
- **THEN** server spawns `chip-tool colorcontrol move-to-color-temperature <mireds> 0 0 0 <node_id> 1`

### Requirement: Non-Matter devices are unaffected
The cluster dispatch logic SHALL only activate when `device.control_protocol == Some(Protocol::Matter)`. All other devices MUST follow the existing code path unchanged.

#### Scenario: Toggle non-Matter device
- **WHEN** a device with `control_protocol: zigbee` has its state toggled
- **THEN** no `chip-tool` subprocess is spawned; only MQTT publish fires (existing behavior)
