## Why

The smart home backend now has a rich entity model, presence detection, areas, scripts, scenes, and automations — but no way to compose them into a visual, purpose-built view. Users need a Lovelace-inspired dashboard system where they can create named views with cards that display real-time entity state, so the app becomes a daily-use control panel rather than a raw data explorer.

## What Changes

- Add a `Dashboard` domain model with named views and typed cards (entity, gauge, button, stat, history)
- Persist dashboards in the database as JSONB (one row per dashboard)
- Expose a full REST API: dashboard CRUD + view CRUD + card CRUD
- Build a frontend Dashboard Builder (create/edit dashboards, arrange cards) and Dashboard Viewer (live rendering of cards from entity state)
- Add a default "Home" dashboard seeded at first startup
- Add `/dashboards` route to the nav

## Capabilities

### New Capabilities
- `dashboard-registry`: Backend domain + DB persistence for dashboards (views, cards, JSONB storage)
- `dashboard-api`: HTTP handlers + routes for dashboard/view/card CRUD
- `dashboard-ui`: Frontend pages — Dashboard Viewer, Dashboard Builder, card components

### Modified Capabilities
- `entity-model`: Expose a `GET /api/entities/{entity_id}` single-entity lookup endpoint (needed by card real-time rendering)

## Impact

- **Backend**: new `domain/dashboard.rs`, `infrastructure/db.rs` extended, `http/handlers/dashboards.rs`, router additions
- **Frontend**: new `app/dashboards/`, `components/dashboard/` tree
- **DB schema**: new `dashboards` table (`id, name, icon, views JSONB, created_at`)
- **OpenAPI**: new `Dashboard`, `View`, `Card`, card-type discriminated union schemas and paths
- **No breaking changes** to existing routes
