## ADDED Requirements

### Requirement: Room is renamed to Area with additional metadata fields
The `Room` struct SHALL be renamed to `Area` and SHALL add: `area_id: String` (kebab-slug derived from name, e.g. "Living Room" → `"living-room"`), `floor: Option<u8>`, `icon: Option<String>`. Existing `name: String` and `device_ids: Vec<String>` fields SHALL be preserved.

#### Scenario: Area is created with auto-derived area_id
- **WHEN** `Area::new("Living Room")` is called
- **THEN** `area.area_id` SHALL be `"living-room"` and `area.name` SHALL be `"Living Room"`

#### Scenario: Area slug handles special characters
- **WHEN** `Area::new("Ground Floor / Hall")` is called
- **THEN** `area.area_id` SHALL be a valid kebab-slug with no slashes or spaces

### Requirement: SmartHome area operations use Area instead of Room
All `SmartHome` methods previously operating on `Room` SHALL operate on `Area`. Method signatures and behaviour SHALL be unchanged except the return type `Room` becomes `Area`.

#### Scenario: add_room creates an Area
- **WHEN** `home.add_room("Kitchen")` is called
- **THEN** an `Area` with `area_id: "kitchen"` SHALL exist in `home.areas`

#### Scenario: list_rooms returns Areas sorted by name
- **WHEN** `home.list_rooms()` is called with areas "Bedroom" and "Attic"
- **THEN** the result SHALL be sorted alphabetically: `["Attic", "Bedroom"]`

### Requirement: GET /api/areas returns all areas with device membership
The system SHALL expose `GET /api/areas` returning a JSON array of all areas, each with `area_id`, `name`, `floor`, `icon`, and `device_count`.

#### Scenario: Returns all areas
- **WHEN** two areas exist and `GET /api/areas` is called
- **THEN** response SHALL be HTTP 200 with a JSON array of 2 area objects

#### Scenario: Empty home returns empty array
- **WHEN** no areas exist and `GET /api/areas` is called
- **THEN** response SHALL be HTTP 200 with an empty JSON array

### Requirement: GET /api/areas/{area_id} returns area detail with device list
The system SHALL expose `GET /api/areas/{area_id}` returning the area metadata plus the full device list for that area.

#### Scenario: Returns area with devices
- **WHEN** area `"kitchen"` has 2 devices and `GET /api/areas/kitchen` is called
- **THEN** response SHALL include `area_id: "kitchen"` and a `devices` array with 2 device objects

#### Scenario: Returns 404 for unknown area
- **WHEN** `GET /api/areas/nonexistent` is called
- **THEN** response SHALL be HTTP 404

### Requirement: Area metadata persists to and loads from the database
The system SHALL persist `floor` and `icon` per device row using `area_floor` and `area_icon` columns added via `ADD COLUMN IF NOT EXISTS`.

#### Scenario: Area floor persists across restart
- **WHEN** a device is assigned to area "Bedroom" with floor 2 and the server restarts
- **THEN** the loaded device SHALL still be in "Bedroom" and area "Bedroom" SHALL have `floor: Some(2)`
