## ADDED Requirements

### Requirement: GET /api/matter/status returns scanner health
The server SHALL expose `GET /api/matter/status` returning a JSON object with total Matter devices seen, commissioning-mode devices count, and last scan timestamp.

#### Scenario: Status with no Matter devices
- **WHEN** `GET /api/matter/status` is called with no Matter devices discovered
- **THEN** response is `200` with `{"devices_seen": 0, "commissioning_count": 0, "last_seen_at": null}`

#### Scenario: Status after discovery
- **WHEN** one Matter device has been discovered
- **THEN** `devices_seen` is 1 and `last_seen_at` is a non-null ISO 8601 string

### Requirement: MatterStatus tracked in AppState
`AppState` SHALL contain `matter_status: Arc<RwLock<MatterStatus>>`. The scanner SHALL increment `devices_seen` and update `last_seen_at` on every resolved device.

#### Scenario: devices_seen increments
- **WHEN** 3 Matter devices are resolved
- **THEN** `GET /api/matter/status` returns `devices_seen >= 3`
