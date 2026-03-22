## ADDED Requirements

### Requirement: EntityKind covers all standard HA domains
The system SHALL define an `EntityKind` enum with variants: `Switch`, `Light`, `Sensor`, `BinarySensor`, `Cover`, `Climate`, `MediaPlayer`, `Lock`, `Camera`, `Number`, `Select`, `Button`. Each variant SHALL serialize to its lowercase HA domain string.

#### Scenario: EntityKind serializes to HA domain string
- **WHEN** `EntityKind::Switch` is formatted via Display
- **THEN** it SHALL produce `"switch"`

#### Scenario: EntityKind serializes Climate correctly
- **WHEN** `EntityKind::Climate` is formatted via Display
- **THEN** it SHALL produce `"climate"`

### Requirement: Entity struct carries identity, kind, and state
The `Entity` struct SHALL contain: `entity_id: String` (format: `{domain}.{slug}`), `kind: EntityKind`, `device_id: String`, `name: String`, `state: String`, `unit_of_measurement: Option<String>`, `attributes: serde_json::Value` (object).

#### Scenario: Entity has correctly formatted entity_id
- **WHEN** an entity is derived for a light named "Desk Lamp"
- **THEN** the `entity_id` for its switch entity SHALL be `"switch.desk_lamp"` and for its brightness entity SHALL be `"number.desk_lamp_brightness"`

### Requirement: Device::entities() derives entities from device type and state
`Device::entities()` SHALL be a pure function returning `Vec<Entity>` based on `device_type` and current field values. The derivation rules SHALL be:
- `Light` → `Switch` (on/off state) + `Number` (brightness, unit `%`)
- `Thermostat` → `Climate` (hvac state) + `Sensor` (current temperature, unit `°C`) + `Number` (target temperature, unit `°C`)
- `Sensor` → `Sensor` (temperature if `temperature` is Some, else generic)
- `Lock` → `Lock` (locked/unlocked)
- `Cover` → `Cover` (open/closed) + `Number` (position, unit `%`)
- `Switch`, `Outlet` → `Switch`
- `Fan` → `Switch` (on/off) + `Select` (speed)
- `Camera` → `Camera`
- `Alarm` → `BinarySensor` (triggered/clear)
- `Tv`, `Speaker`, `MediaPlayer` → `MediaPlayer`
- `Hub` → `BinarySensor` (online/offline)

#### Scenario: Light device produces two entities
- **WHEN** `Device::entities()` is called on a light device named "Lamp" with state On and brightness 75
- **THEN** the result SHALL contain exactly 2 entities: one `Switch` with state `"on"` and one `Number` with state `"75"` and unit `"%"`

#### Scenario: Thermostat produces three entities
- **WHEN** `Device::entities()` is called on a thermostat named "Hall Thermo" with temperature `21.5` and state Off
- **THEN** the result SHALL contain 3 entities: `Climate`, `Sensor` (state `"21.5"`, unit `"°C"`), and `Number`

#### Scenario: entities() is pure — same input yields same output
- **WHEN** `Device::entities()` is called twice on the same unchanged device
- **THEN** both results SHALL be identical

### Requirement: GET /api/devices/{name}/entities returns derived entity list
The system SHALL expose `GET /api/devices/{name}/entities` returning `Vec<EntityResponse>`.

#### Scenario: Returns entities for existing device
- **WHEN** `GET /api/devices/lamp/entities` is called and lamp is a light
- **THEN** response SHALL be HTTP 200 with a JSON array of entity objects

#### Scenario: Returns 404 for unknown device
- **WHEN** `GET /api/devices/missing/entities` is called
- **THEN** response SHALL be HTTP 404

### Requirement: GET /api/entities returns all home entities with optional kind filter
The system SHALL expose `GET /api/entities` returning a flat list of all entities across all devices. An optional `?kind=` query parameter SHALL filter by `EntityKind` domain string.

#### Scenario: Returns all entities when no filter applied
- **WHEN** `GET /api/entities` is called with a home containing 1 light and 1 thermostat
- **THEN** the response SHALL contain 5 entities (2 from light + 3 from thermostat)

#### Scenario: Kind filter narrows results
- **WHEN** `GET /api/entities?kind=sensor` is called
- **THEN** response SHALL contain only entities with `kind == "sensor"`

#### Scenario: Empty home returns empty array
- **WHEN** `GET /api/entities` is called on a home with no devices
- **THEN** response SHALL be HTTP 200 with an empty JSON array
