## ADDED Requirements

### Requirement: Dashboard CRUD endpoints
The system SHALL expose the following HTTP endpoints under `/api/dashboards`:

- `GET /api/dashboards` → 200 `DashboardResponse[]`
- `POST /api/dashboards` → 201 `DashboardResponse` (409 on duplicate name)
- `GET /api/dashboards/{id}` → 200 `DashboardResponse` (404 if missing)
- `DELETE /api/dashboards/{id}` → 204 (404 if missing)

#### Scenario: List returns all dashboards
- **WHEN** `GET /api/dashboards` is called
- **THEN** 200 is returned with an array of all dashboards

#### Scenario: Create returns 201 with new dashboard
- **WHEN** `POST /api/dashboards` is called with `{"name": "Bedroom", "icon": "🛏"}`
- **THEN** 201 is returned with the new dashboard including a generated id and empty views

#### Scenario: Duplicate name returns 409
- **WHEN** `POST /api/dashboards` is called with a name that already exists
- **THEN** 409 is returned with `"code": "conflict"`

#### Scenario: Get unknown dashboard returns 404
- **WHEN** `GET /api/dashboards/{id}` is called with a non-existent id
- **THEN** 404 is returned

### Requirement: View CRUD endpoints
The system SHALL expose view management nested under dashboards:

- `POST /api/dashboards/{id}/views` → 200 `DashboardResponse` (404 if dashboard missing)
- `DELETE /api/dashboards/{id}/views/{view_id}` → 200 `DashboardResponse` (404 if dashboard or view missing)

#### Scenario: Add view to dashboard
- **WHEN** `POST /api/dashboards/{id}/views` is called with `{"title": "Living Room"}`
- **THEN** 200 is returned with the dashboard containing the new view

#### Scenario: Delete view from dashboard
- **WHEN** `DELETE /api/dashboards/{id}/views/{view_id}` is called for an existing view
- **THEN** 200 is returned with the updated dashboard (view absent)

#### Scenario: Add view to non-existent dashboard returns 404
- **WHEN** `POST /api/dashboards/no-such-id/views` is called
- **THEN** 404 is returned

### Requirement: Card CRUD endpoints
The system SHALL expose card management nested under views:

- `POST /api/dashboards/{id}/views/{view_id}/cards` → 200 `DashboardResponse` (404 if dashboard/view missing)
- `DELETE /api/dashboards/{id}/views/{view_id}/cards/{card_id}` → 200 `DashboardResponse` (404 if any ID missing)

#### Scenario: Add entity card to view
- **WHEN** `POST /api/dashboards/{id}/views/{view_id}/cards` is called with `{"card_type": "entity_card", "entity_id": "device.lamp.switch"}`
- **THEN** 200 is returned with the dashboard; the view contains the new card

#### Scenario: Add gauge card with all fields
- **WHEN** a gauge card is added with `{"card_type": "gauge_card", "entity_id": "device.thermo.sensor", "min": 0, "max": 100, "unit": "°C"}`
- **THEN** 200 is returned; the card round-trips all fields

#### Scenario: Delete card from view
- **WHEN** `DELETE /api/dashboards/{id}/views/{view_id}/cards/{card_id}` is called for an existing card
- **THEN** 200 is returned with the dashboard (card absent from view)

### Requirement: Dashboard persistence
The system SHALL persist dashboards to the `dashboards` table with `views` stored as JSONB. On server startup, all dashboards SHALL be loaded from the database into the registry.

#### Scenario: Dashboard survives restart
- **WHEN** a dashboard is created via the API and the registry is re-loaded from DB
- **THEN** the dashboard and all its views and cards are present with original IDs

#### Scenario: Dashboard deletion removes DB row
- **WHEN** `DELETE /api/dashboards/{id}` is called
- **THEN** the row is removed from the `dashboards` table
