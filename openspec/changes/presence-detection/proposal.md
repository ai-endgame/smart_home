## Why

Smart home automations that depend on occupancy — "nobody home" scenes, welcome lighting, HVAC setback — are unreliable without a structured presence model. A single GPS tracker is slow and battery-intensive; a single ping has false negatives. We need a layered presence engine that aggregates multiple evidence sources into a confident `home`/`away` state with grace-period debouncing.

## What Changes

- Introduce a `PersonTracker` domain entity that holds multiple `TrackerSource` readings (network ping, BLE beacon, manual override) and computes a merged `PresenceState` (`home` / `away` / `unknown`).
- Add a `PresenceRegistry` to `AppState` — stores all tracked persons; exposes `get`, `list`, `update_source`.
- Add a configurable grace-period engine: `not_home` transition only fires after N consecutive seconds of `away` evidence across all sources.
- Expose REST endpoints for person CRUD, source updates, and presence queries (`GET /api/presence/persons`, `POST /api/presence/persons`, `PATCH /api/presence/persons/{id}/sources/{source}`).
- Extend the `Trigger` enum with `PresenceChange { person_name, target_state }` so automations can react to arrivals and departures.
- Add a `person` entity kind to the entity model (kind = `"person"`) alongside the device entity kinds.
- Frontend: `/presence` page showing each person's current state, source breakdown, and a manual override toggle.

## Capabilities

### New Capabilities

- `person-tracker`: Domain entity `PersonTracker` with multi-source aggregation logic, grace-period debouncing, and `PresenceRegistry`
- `presence-api`: REST endpoints for person CRUD and source updates; OpenAPI contract extensions
- `presence-automation-trigger`: `PresenceChange` trigger variant wired into `evaluate_rules`
- `presence-frontend`: `/presence` page, `usePresence` SWR hook, API client

### Modified Capabilities

- `entity-model`: Add `person` entity kind alongside existing device entity kinds

## Impact

- **Backend**: `domain/presence.rs` (new), `domain/automation.rs` (new trigger), `state.rs` (registry), `http/handlers/presence.rs` (new), `http/router.rs` (new routes), `http/types.rs` (DTOs), `infrastructure/db.rs` (persons table migration)
- **Frontend**: `lib/api/presence.ts`, `lib/hooks/use-presence.ts`, `app/presence/page.tsx`, `components/presence/`, `components/layout/nav.tsx`
- **Contract**: `contracts/openapi.yaml` — new `presence` tag + paths + schemas
- **Dependencies**: No new crates; chrono already included for grace-period timing
