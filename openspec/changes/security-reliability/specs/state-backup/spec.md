## ADDED Requirements

### Requirement: System state backup endpoint
The system SHALL expose `GET /api/backup` that returns a complete snapshot of all mutable system state as a JSON document with the following structure:

```json
{
  "version": "1.0",
  "exported_at": "<ISO 8601 timestamp>",
  "devices": [...],
  "automation_rules": [...],
  "scripts": [...],
  "scenes": [...],
  "persons": [...],
  "dashboards": [...]
}
```

The response Content-Type SHALL be `application/json`. The response body SHALL be a valid `BackupDocument`.

#### Scenario: Backup with populated state returns all resources
- **GIVEN** the system has at least one device, one automation rule, one script, one scene, one person, and one dashboard
- **WHEN** `GET /api/backup` is called
- **THEN** 200 is returned with a JSON body containing `version: "1.0"`, a non-null `exported_at` timestamp, and non-empty arrays for each resource type

#### Scenario: Backup with empty state returns empty arrays
- **WHEN** `GET /api/backup` is called on a fresh system with no data
- **THEN** 200 is returned with `devices: []`, `automation_rules: []`, `scripts: []`, `scenes: []`, `persons: []`, `dashboards: []`

#### Scenario: Backup is a point-in-time snapshot
- **GIVEN** a device exists
- **WHEN** `GET /api/backup` is called and the response is parsed
- **THEN** the device appears in `devices` with its current state (name, device_type, state fields)

### Requirement: Backup document schema
The `BackupDocument` SHALL be a stable, versioned schema. The `version` field identifies the snapshot format for future restore compatibility. The `exported_at` field is an ISO 8601 UTC timestamp.
