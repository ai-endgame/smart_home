## ADDED Requirements

### Requirement: List persons endpoint
The system SHALL provide `GET /api/presence/persons` returning an array of `PersonResponse` objects (id, name, grace_period_secs, effective_state, sources map).

#### Scenario: Empty list
- **WHEN** no persons exist
- **THEN** `GET /api/presence/persons` SHALL return `200 OK` with an empty JSON array

#### Scenario: Returns all persons with effective state
- **WHEN** persons exist with various source states
- **THEN** each object SHALL include `effective_state` computed at request time

### Requirement: Create person endpoint
The system SHALL provide `POST /api/presence/persons` accepting `{ name, grace_period_secs? }`. On success it SHALL return `201 Created` with the created `PersonResponse`. Duplicate names SHALL return `409 Conflict`.

#### Scenario: Create succeeds
- **WHEN** `POST /api/presence/persons` is called with a unique name
- **THEN** the response SHALL be `201 Created` with `id`, `name`, and `effective_state: "unknown"`

#### Scenario: Duplicate name returns 409
- **WHEN** `POST /api/presence/persons` is called with a name that already exists
- **THEN** the response SHALL be `409 Conflict` with `{ "code": "conflict" }`

### Requirement: Get person endpoint
The system SHALL provide `GET /api/presence/persons/{id}` returning the `PersonResponse` for the given UUID, or `404 Not Found`.

#### Scenario: Not found
- **WHEN** `GET /api/presence/persons/nonexistent` is called
- **THEN** the response SHALL be `404 Not Found`

### Requirement: Delete person endpoint
The system SHALL provide `DELETE /api/presence/persons/{id}` removing the person. On success it SHALL return `204 No Content`. If not found, `404 Not Found`.

#### Scenario: Delete removes person
- **WHEN** a person is deleted
- **THEN** subsequent `GET /api/presence/persons/{id}` SHALL return `404 Not Found`

### Requirement: Update source endpoint
The system SHALL provide `PATCH /api/presence/persons/{id}/sources/{source}` accepting `{ state: "home" | "away" | "unknown" }` and updating the named source on the person. Returns `200 OK` with updated `PersonResponse`.

#### Scenario: Source update reflected in effective state
- **WHEN** all sources are set to `"away"` and grace period has elapsed
- **THEN** the `effective_state` in the response SHALL be `"away"`

#### Scenario: Home source overrides away sources
- **WHEN** one source is set to `"home"` while others are `"away"`
- **THEN** the `effective_state` in the response SHALL be `"home"`

### Requirement: DB persistence for persons
The system SHALL persist persons to a `persons` table with columns: `id TEXT PK`, `name TEXT UNIQUE`, `grace_period_secs INT`, `sources JSONB`, `away_since TIMESTAMPTZ`, `created_at TIMESTAMPTZ`. Persons SHALL be loaded at server startup.

#### Scenario: Persons survive server restart
- **WHEN** a person is created and the registry is reloaded from the DB
- **THEN** the person SHALL be present with the same id, name, and grace_period_secs
