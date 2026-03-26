## ADDED Requirements

### Requirement: Person entity kind
The system SHALL add `person` as a valid `EntityKind` variant alongside the existing device-derived entity kinds (`switch`, `number`, `sensor`, `climate`, `cover`, `lock`, `text_sensor`).

#### Scenario: Person entity kind serializes correctly
- **WHEN** `EntityKind::Person` is displayed or serialized
- **THEN** it SHALL produce the string `"person"`

### Requirement: GET /api/entities includes person entities
The system SHALL include one entity per `PersonTracker` in the `GET /api/entities` response. Each person entity SHALL have `kind = "person"`, `entity_id = "person.<name_slug>"`, `device_id = person.id`, `state = effective_state as string`, and no `unit_of_measurement`.

#### Scenario: Person entity appears in entity list
- **WHEN** a person named "Alice" exists with `effective_state = "home"`
- **THEN** `GET /api/entities` SHALL include `{ "kind": "person", "entity_id": "person.alice", "state": "home" }`

#### Scenario: Kind filter works for person
- **WHEN** `GET /api/entities?kind=person` is called
- **THEN** only person entities SHALL be returned
