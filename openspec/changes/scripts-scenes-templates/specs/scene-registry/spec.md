## ADDED Requirements

### Requirement: Scene creation
The system SHALL allow users to create named scenes. A scene SHALL have a name (unique, case-insensitive) and a map of device states (`device_id → { state?, brightness?, temperature? }`). The system SHALL support two creation modes: explicit (states provided in the request body) and snapshot (states captured from current SmartHome state for a given list of device IDs).

#### Scenario: Explicit scene creation
- **WHEN** a `POST /api/scenes` request is received with a name and a non-empty states map
- **THEN** the system stores the scene, assigns a UUID, persists it to DB, and returns `201` with the scene object

#### Scenario: Snapshot scene creation
- **WHEN** a `POST /api/scenes/snapshot` request is received with a name and a list of device IDs
- **THEN** the system reads current state of each listed device from SmartHome, creates a scene from those states, and returns `201`

#### Scenario: Duplicate scene name rejected
- **WHEN** a scene creation request uses a name matching an existing scene
- **THEN** the system returns `409 Conflict`

### Requirement: Scene apply
The system SHALL apply a scene via `POST /api/scenes/{id}/apply`. Applying a scene SHALL update each device in the scene's state map by mutating SmartHome state and persisting each changed device. Protocol dispatch (Matter/MQTT) SHALL be triggered for each device, identical to how individual `set_device_state` handler works. Partial failures SHALL be collected and returned in the response; remaining devices SHALL still be applied.

#### Scenario: Successful scene apply
- **WHEN** `POST /api/scenes/{id}/apply` is called and all devices exist
- **THEN** all device states in the scene are applied and `200` is returned with `{ applied: N, errors: [] }`

#### Scenario: Partial failure on scene apply
- **WHEN** `POST /api/scenes/{id}/apply` is called and one device in the scene does not exist in SmartHome
- **THEN** the missing device is skipped, all others are applied, and `200` is returned with `{ applied: N-1, errors: ["device X not found"] }`

#### Scenario: Apply unknown scene
- **WHEN** `POST /api/scenes/{id}/apply` is called with a non-existent ID
- **THEN** the system returns `404 Not Found`

### Requirement: Scene CRUD
The system SHALL support: `GET /api/scenes` (list all), `GET /api/scenes/{id}` (get one), `PUT /api/scenes/{id}` (replace states map), `DELETE /api/scenes/{id}`.

#### Scenario: List scenes
- **WHEN** `GET /api/scenes` is called
- **THEN** the system returns all scenes ordered by name

#### Scenario: Update scene states
- **WHEN** `PUT /api/scenes/{id}` is called with a new states map
- **THEN** the scene's states are replaced in memory and DB, and `200` is returned with the updated scene
