## ADDED Requirements

### Requirement: POST /api/restore endpoint
The system SHALL expose `POST /api/restore` that accepts a `BackupDocument` JSON body. The handler SHALL acquire write locks on all registries in canonical order (home → automation → scripts → scenes → presence → dashboard), clear each registry, re-import all entities from the backup, and release locks together. If a database is configured, all existing records SHALL be deleted and re-inserted. The response SHALL be `200` with a summary of restored counts: `{"restored": {"devices": N, "rules": N, "scripts": N, "scenes": N, "persons": N, "dashboards": N}}`.

#### Scenario: Restore replaces all state
- **WHEN** `POST /api/restore` is called with a valid `BackupDocument` containing one device and one rule
- **THEN** the response is `200`, `GET /api/devices` returns exactly that one device, and `GET /api/automation/rules` returns exactly that one rule

#### Scenario: Restore with empty backup clears state
- **WHEN** `POST /api/restore` is called with a `BackupDocument` where all arrays are empty
- **THEN** the response is `200`, and all registries are empty

#### Scenario: Restore with invalid body returns 400
- **WHEN** `POST /api/restore` is called with a body missing the `version` field
- **THEN** the response is `400` or `422`

### Requirement: Restore events recorded
After a successful restore, the system SHALL call `record_event` with `EventKind::Server` and a message summarizing the restore counts. This event SHALL be visible in `GET /api/events`.

#### Scenario: Restore event is recorded
- **WHEN** `POST /api/restore` completes successfully
- **THEN** `GET /api/events` contains a `Server` event with a message mentioning the restore
