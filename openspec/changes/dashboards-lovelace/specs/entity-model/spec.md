## ADDED Requirements

### Requirement: Single entity lookup endpoint
The system SHALL expose `GET /api/entities/{entity_id}` that returns the entity matching the given `entity_id` string, or 404 if no entity with that ID exists.

#### Scenario: Known entity_id returns 200
- **WHEN** `GET /api/entities/device.lamp.switch` is called and a lamp switch entity exists
- **THEN** 200 is returned with the entity object including `entity_id`, `kind`, `state`, and `device_id`

#### Scenario: Unknown entity_id returns 404
- **WHEN** `GET /api/entities/device.nosuch.switch` is called and no matching entity exists
- **THEN** 404 is returned with `"code": "not_found"`
