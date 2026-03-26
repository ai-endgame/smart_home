## ADDED Requirements

### Requirement: Request body size limit
The system SHALL reject any request whose body exceeds 65,536 bytes (64 KB) with HTTP 413 Payload Too Large before the handler processes it.

#### Scenario: Request within size limit is processed normally
- **WHEN** `POST /api/devices` is called with a JSON body smaller than 64 KB
- **THEN** the request reaches the handler

#### Scenario: Request exceeding size limit is rejected
- **WHEN** any request is sent with a body larger than 64 KB
- **THEN** 413 is returned before any handler logic runs

### Requirement: Input name/title length validation
The system SHALL enforce a maximum length of 120 UTF-8 characters for all user-supplied name, title, and label fields. Handlers that accept a name or title field SHALL return `400 Bad Request` with `"code": "bad_request"` if the value exceeds 120 characters.

This applies to at minimum:
- `POST /api/devices` — `name` field
- `POST /api/automation/rules` — `name` field
- `POST /api/scripts` — `name` field
- `POST /api/scenes/snapshot` — `name` field
- `POST /api/presence/persons` — `name` field
- `POST /api/dashboards` — `name` field
- `POST /api/dashboards/{id}/views` — `title` field

#### Scenario: Name within limit is accepted
- **WHEN** `POST /api/devices` is called with `{"name": "lamp", "device_type": "light"}`
- **THEN** 201 is returned

#### Scenario: Name exceeding limit is rejected
- **WHEN** `POST /api/devices` is called with a `name` field of 121 characters
- **THEN** 400 is returned with `"code": "bad_request"`

#### Scenario: Name at exactly the limit is accepted
- **WHEN** `POST /api/devices` is called with a `name` field of exactly 120 characters
- **THEN** 201 is returned
