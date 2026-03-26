## ADDED Requirements

### Requirement: Protocol enum covers all supported transports
The system SHALL define a `Protocol` enum in the domain layer covering: `Zigbee`, `ZWave`, `Matter`, `Thread`, `WiFi`, `Shelly`, `Tasmota`, `ESPHome`, `WLED`, and `Unknown`.

#### Scenario: Valid protocol strings are parsed
- **WHEN** `Protocol::from_str_loose` receives a known string (e.g. "zigbee", "shelly", "tasmota", "esphome", "wled", "zwave", "matter", "thread", "wifi")
- **THEN** it SHALL return `Some(Protocol::...)` with the correct variant

#### Scenario: Unknown protocol strings are handled gracefully
- **WHEN** `Protocol::from_str_loose` receives an unrecognized string (e.g. "foobar")
- **THEN** it SHALL return `None` and the caller SHALL log a WARN

#### Scenario: Protocol serializes to canonical snake_case string
- **WHEN** `Protocol::to_string()` (Display) is called on any variant
- **THEN** it SHALL return the canonical lowercase snake_case identifier (e.g. `Protocol::ZWave` → `"z_wave"`, `Protocol::ESPHome` → `"esphome"`)

### Requirement: Protocol carries static metadata via ProtocolInfo
The system SHALL provide a `Protocol::info(&self) -> ProtocolInfo` method returning a struct with: `transport: &'static str`, `local_only: bool`, `mesh: bool`, `description: &'static str`.

#### Scenario: Zigbee reports correct metadata
- **WHEN** `Protocol::Zigbee.info()` is called
- **THEN** `transport` SHALL be `"802.15.4"`, `local_only` SHALL be `true`, `mesh` SHALL be `true`

#### Scenario: WiFi-based protocols report non-mesh
- **WHEN** `Protocol::Shelly.info()` or `Protocol::Tasmota.info()` is called
- **THEN** `mesh` SHALL be `false` and `local_only` SHALL be `true`

#### Scenario: Matter reports IP-based transport
- **WHEN** `Protocol::Matter.info()` is called
- **THEN** `transport` SHALL be `"IP/Thread/Wi-Fi"`, `local_only` SHALL be `true`, `mesh` SHALL be `true`

### Requirement: Device stores typed Protocol field
The `Device` struct SHALL store `control_protocol: Option<Protocol>` instead of `Option<String>`.

#### Scenario: Device with protocol persists and reloads correctly
- **WHEN** a device with `control_protocol: Some(Protocol::Shelly)` is upserted to the DB and reloaded
- **THEN** the reloaded device SHALL have `control_protocol: Some(Protocol::Shelly)`

#### Scenario: Device with unknown DB protocol string loads with None
- **WHEN** a DB row has `control_protocol = 'legacy_proprietary'` (unrecognized)
- **THEN** the loaded device SHALL have `control_protocol: None` and a WARN SHALL be logged

### Requirement: GET /api/protocols returns all supported protocols
The system SHALL expose `GET /api/protocols` returning a JSON array of all `Protocol` variants (not just those in use) with their `ProtocolInfo` fields.

#### Scenario: Endpoint returns complete static list
- **WHEN** `GET /api/protocols` is called with an empty home
- **THEN** the response SHALL contain entries for all defined protocol variants

#### Scenario: Each entry includes required fields
- **WHEN** `GET /api/protocols` is called
- **THEN** each item in the array SHALL include: `id` (snake_case), `transport`, `local_only`, `mesh`, `description`
