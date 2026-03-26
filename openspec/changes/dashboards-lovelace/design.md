## Context

The project has a complete entity model, presence detection, area registry, scripts, scenes, and automations. All data is exposed via REST and consumed by Next.js pages. However every page is domain-specific (devices page, automation page, etc.) — there is no user-composable view. Lovelace (HA's dashboard system) shows the right model: named dashboards → views (tabs) → cards (widgets). This design adapts that model to the existing Rust/Axum/Next.js stack without pulling in HA code.

## Goals / Non-Goals

**Goals:**
- `Dashboard` model: id, name, icon, `views: Vec<View>`, created_at
- `View` model: id, title, icon, `cards: Vec<Card>`
- `Card` model: id, card_type discriminated union (entity, gauge, button, stat, history), layout position (col/row/w/h for future grid), title override
- JSONB persistence: one `dashboards` row, views+cards stored as `serde_json::Value`
- Full REST CRUD: dashboards + views + cards
- Single-entity lookup endpoint: `GET /api/entities/{entity_id}` (needed by card rendering)
- Frontend: Dashboard Viewer page (live cards from SWR) + Dashboard Builder (add/edit dashboards/views/cards)
- Default "Home" dashboard seeded at startup if none exist

**Non-Goals:**
- Drag-and-drop grid editing (layout stored but not visually editable this session)
- Real-time WebSocket push (cards poll via SWR refreshInterval)
- Custom HACS-style card plugins
- Dashboard sharing / multi-user permissions

## Decisions

**D1: Store views+cards as JSONB inside the dashboard row (not normalized tables)**
- Dashboards are "documents" — a view or card has no meaning outside its dashboard. Normalizing into 3 tables adds join complexity with no query benefit (we always load the full dashboard). JSONB makes the schema migration trivial and allows flexible card shapes.
- Alternative: separate `views` and `cards` tables with FKs. Rejected: more migration surface, no performance benefit for typical dashboard sizes (<50 cards).

**D2: Card type as a serde-tagged enum (`card_type` field)**
- Rust: `#[serde(tag = "card_type", rename_all = "snake_case")]` on `CardContent`. This gives clean JSON (`"card_type": "entity_card"`) and exhaustive matching.
- Alternative: `type` string + `config: serde_json::Value`. Rejected: loses type safety and forces runtime validation everywhere.

**D3: Card types in scope**
- `EntityCard { entity_id: String }` — shows entity state + icon
- `GaugeCard { entity_id: String, min: f64, max: f64, unit: Option<String> }` — numeric gauge
- `ButtonCard { entity_id: String, action: String }` — tap to execute (state toggle, script call)
- `StatCard { title: String, entity_ids: Vec<String>, aggregation: String }` — count/sum/avg over entities
- `HistoryCard { entity_id: String, hours: u32 }` — sparkline placeholder (no time-series DB yet)

**D4: Single-entity lookup via entity_id slug**
- Cards reference `entity_id` strings (e.g., `"device.lamp.switch"`). The frontend needs to fetch one entity by ID without loading all entities. Add `GET /api/entities/{entity_id}` to the entities handler. The entity_id format is already stable from the entity model spec.

**D5: Default dashboard seeded at startup**
- If `load_all_dashboards` returns empty, insert one dashboard named "Home" with a single view named "Overview" and no cards. This gives users something to open on first visit. Seeding is idempotent (only if none exist).

**D6: Frontend card rendering is client-side SWR**
- Each card component calls `useEntity(entity_id)` (a thin SWR hook over `GET /api/entities/{entity_id}`) with `refreshInterval: 3000`. No special streaming needed; 3s poll is acceptable for a dashboard.

## Risks / Trade-offs

- [JSONB migration is irreversible without data loss] → Mitigation: all card shapes are well-typed at the Rust boundary; the DB column is append-friendly. If schema changes, we can migrate JSONB in-place.
- [`HistoryCard` is a stub — no time-series data exists] → Mitigation: render a "history unavailable" placeholder; card type is reserved for future implementation.
- [Card grid layout (col/row/w/h) stored but not rendered as a true grid yet] → Mitigation: frontend renders cards in a simple responsive grid by order; layout fields are preserved for a future drag-and-drop session.
- [Default dashboard creation at startup is not idempotent if DB is unavailable] → Mitigation: seeding only runs when a DB pool is present; in-memory mode gets no default dashboard.

## Migration Plan

1. Add `dashboards` table to `db::migrate()` (JSONB views column)
2. Deploy backend — table created on first connect
3. Startup seeding inserts default "Home" dashboard if table is empty
4. No rollback needed — new table, no changes to existing tables

## Open Questions

- Should views support background images or color themes? (deferred to future session)
- Should `ButtonCard` support script calls directly, or only entity state toggles? (include both: `action: "toggle" | "script:<name>"`)
