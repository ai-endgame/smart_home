## 1. Domain — Dashboard model and registry

- [x] 1.1 Create `backend/src/domain/dashboard.rs` with `CardContent` enum (`EntityCard`, `GaugeCard`, `ButtonCard`, `StatCard`, `HistoryCard`) using `#[serde(tag = "card_type", rename_all = "snake_case")]`
- [x] 1.2 Add `Card` struct: `id: String`, `title: Option<String>`, `content: CardContent`; implement `Card::new(content)`
- [x] 1.3 Add `View` struct: `id: String`, `title: String`, `icon: Option<String>`, `cards: Vec<Card>`; implement `View::new(title, icon)`
- [x] 1.4 Add `Dashboard` struct: `id: String`, `name: String`, `icon: Option<String>`, `views: Vec<View>`, `created_at: DateTime<Utc>`; implement `Dashboard::new(name, icon)`
- [x] 1.5 Add `DashboardRegistry` with `by_id: HashMap<String, Dashboard>` and `by_name: HashMap<String, String>` (name_key → id); implement `add`, `get`, `get_mut`, `remove`, `list`; duplicate name returns `DomainError::Conflict`
- [x] 1.6 Register `pub mod dashboard` in `backend/src/domain/mod.rs`
- [x] 1.7 Add unit tests: create dashboard, duplicate name rejected, remove returns dashboard, card round-trips all types

## 2. Infrastructure — DB persistence

- [x] 2.1 Add `dashboards` table migration to `infrastructure/db.rs::migrate()`: `id TEXT PK, name TEXT UNIQUE NOT NULL, icon TEXT, views JSONB NOT NULL DEFAULT '[]', created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- [x] 2.2 Implement `db::upsert_dashboard(pool, dashboard)` serializing `dashboard.views` as JSONB
- [x] 2.3 Implement `db::delete_dashboard(pool, id)`
- [x] 2.4 Implement `db::load_all_dashboards(pool) -> Vec<Dashboard>` deserializing the JSONB views column

## 3. State — AppState wiring and startup seeding

- [x] 3.1 Add `pub dashboard: Arc<RwLock<DashboardRegistry>>` to `AppState` in `state.rs`; initialize `DashboardRegistry::new()` in `AppState::new()`
- [x] 3.2 Load dashboards from DB at server startup in `http/mod.rs::run_server_full` (same pattern as persons/scenes)
- [x] 3.3 After loading, if registry is empty and DB is available, insert the default "Home" dashboard with one "Overview" view and persist it via `db::upsert_dashboard`

## 4. HTTP — DTOs and types

- [x] 4.1 Add `CardContentResponse` (mirrors `CardContent`), `CardResponse`, `ViewResponse`, `DashboardResponse` to `http/types.rs`
- [x] 4.2 Add `CreateDashboardRequest { name: String, icon: Option<String> }` to `http/types.rs`
- [x] 4.3 Add `CreateViewRequest { title: String, icon: Option<String> }` to `http/types.rs`
- [x] 4.4 Add `CreateCardRequest` (flat JSON with `card_type` discriminator matching `CardContent` fields) to `http/types.rs` with `to_domain()` conversion
- [x] 4.5 Add `dashboard_to_response(dashboard) -> DashboardResponse` helper to `http/helpers.rs`

## 5. HTTP — Dashboard handlers

- [x] 5.1 Create `backend/src/http/handlers/dashboards.rs` with `list_dashboards` and `create_dashboard` (201/409)
- [x] 5.2 Add `get_dashboard` (200/404) and `delete_dashboard` (204/404) to `dashboards.rs`
- [x] 5.3 Add `add_view` (200/404) and `delete_view` (200/404) to `dashboards.rs`
- [x] 5.4 Add `add_card` (200/404) and `delete_card` (200/404) to `dashboards.rs`
- [x] 5.5 Register `pub mod dashboards` in `http/handlers/mod.rs`
- [x] 5.6 Add routes to `http/router.rs` under `/api/dashboards`: list, create, get, delete; views: add, delete; cards: add, delete

## 6. Entity model — Single entity lookup

- [x] 6.1 Add `get_entity` handler to `http/handlers/entities.rs`: load all entities, find by `entity_id` param, return 200 or 404
- [x] 6.2 Register `GET /api/entities/{entity_id}` route in `http/router.rs`

## 7. OpenAPI contract

- [x] 7.1 Add `dashboards` tag to `contracts/openapi.yaml`
- [x] 7.2 Add schemas: `CardContent` (oneOf discriminated by `card_type`), `Card`, `View`, `Dashboard`, `CreateDashboardRequest`, `CreateViewRequest`, `CreateCardRequest`
- [x] 7.3 Add paths: `GET/POST /api/dashboards`, `GET/DELETE /api/dashboards/{id}`, `POST /api/dashboards/{id}/views`, `DELETE /api/dashboards/{id}/views/{view_id}`, `POST /api/dashboards/{id}/views/{view_id}/cards`, `DELETE /api/dashboards/{id}/views/{view_id}/cards/{card_id}`
- [x] 7.4 Add `GET /api/entities/{entity_id}` path to openapi.yaml

## 8. Frontend — Types and API client

- [x] 8.1 Add `CardContent` (discriminated union), `Card`, `View`, `Dashboard`, `CreateDashboardRequest`, `CreateViewRequest`, `CreateCardRequest` to `frontend/lib/api/types.ts`
- [x] 8.2 Create `frontend/lib/api/dashboards.ts` with `listDashboards`, `createDashboard`, `deleteDashboard`, `addView`, `deleteView`, `addCard`, `deleteCard`
- [x] 8.3 Add `getEntity(entityId)` to `frontend/lib/api/entities.ts` (or create it): `GET /api/entities/{entityId}`
- [x] 8.4 Create `frontend/lib/hooks/use-dashboards.ts` SWR hook with `create`, `remove`, `addView`, `removeView`, `addCard`, `removeCard` mutation helpers
- [x] 8.5 Create `frontend/lib/hooks/use-entity.ts` SWR hook: `useEntity(entityId)` with `refreshInterval: 3000`

## 9. Frontend — Card components

- [x] 9.1 Create `frontend/components/dashboard/EntityCard.tsx`: renders entity kind label and state value; uses `useEntity`
- [x] 9.2 Create `frontend/components/dashboard/GaugeCard.tsx`: renders a percentage fill bar using `(value - min) / (max - min)`; uses `useEntity`
- [x] 9.3 Create `frontend/components/dashboard/ButtonCard.tsx`: renders a button that PATCHes the entity state on click; uses `useEntity`; supports `action = "toggle"` (flips on/off) and `action = "script:<name>"` (calls script run endpoint)
- [x] 9.4 Create `frontend/components/dashboard/StatCard.tsx`: renders count/sum/avg of entity states; uses multiple `useEntity` calls
- [x] 9.5 Create `frontend/components/dashboard/HistoryCard.tsx`: renders a "History unavailable" placeholder with entity_id label
- [x] 9.6 Create `frontend/components/dashboard/CardRenderer.tsx`: switch on `card.content.card_type` and render the appropriate card component

## 10. Frontend — Dashboard Viewer page

- [x] 10.1 Create `frontend/app/dashboards/page.tsx`: lists dashboards from `useDashboards`; clicking one shows its views as tabs; clicking a tab renders its cards via `CardRenderer`; empty states for no dashboards, no views, no cards
- [x] 10.2 Add dashboard selector sidebar or dropdown when multiple dashboards exist
- [x] 10.3 Add `/dashboards` link to `frontend/components/layout/nav.tsx`

## 11. Frontend — Dashboard Builder page

- [x] 11.1 Create `frontend/app/dashboards/builder/page.tsx` with a form to create a new dashboard (name + optional icon)
- [x] 11.2 Add "Add View" form (title + optional icon) that calls `addView` on the selected dashboard
- [x] 11.3 Add "Add Card" form: card type selector (entity/gauge/button/stat/history) + dynamic fields per type; calls `addCard`
- [x] 11.4 Add delete buttons for dashboards, views, and cards in the builder UI
- [x] 11.5 Add link to builder from the viewer page ("Edit" button)

## 12. Integration tests

- [x] 12.1 Add HTTP integration test `dashboard_crud` in `backend/src/http/mod.rs`: list empty → create (201) → conflict (409) → get (200) → get unknown (404) → delete (204) → gone (404)
- [x] 12.2 Add test `dashboard_view_and_card_operations`: create dashboard → add view → add entity card → delete card → delete view
- [x] 12.3 Add test `entity_single_lookup`: create device → get entity → 200 with correct entity_id; get unknown entity_id → 404
- [x] 12.4 Run `cargo test` and `cargo clippy -- -D warnings`; fix all issues
