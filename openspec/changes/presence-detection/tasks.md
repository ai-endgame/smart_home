## 1. Domain — PersonTracker and PresenceRegistry

- [x] 1.1 Create `backend/src/domain/presence.rs` with `SourceState` enum (`Home`, `Away`, `Unknown`) with serde
- [x] 1.2 Add `PresenceState` enum (`Home`, `Away`, `Unknown`) with serde and Display
- [x] 1.3 Add `PersonTracker` struct: id, name, grace_period_secs, sources (`HashMap<String, SourceState>`), away_since (`Option<DateTime<Utc>>`)
- [x] 1.4 Implement `PersonTracker::new(name, grace_period_secs)` generating a UUID id
- [x] 1.5 Implement `PersonTracker::effective_state(&self, now: DateTime<Utc>) -> PresenceState` using "any home wins" + grace period logic
- [x] 1.6 Implement `PersonTracker::update_source(&mut self, source, state, now)` that sets `away_since` when all sources go away/unknown, clears it when any source is home
- [x] 1.7 Add `PresenceRegistry` with dual index (`by_id: HashMap<String, PersonTracker>`, `by_name: HashMap<String, String>` name_key→id), and `add`, `get`, `get_by_name`, `get_mut`, `remove`, `list` methods; duplicate name returns `DomainError::Conflict`
- [x] 1.8 Add unit tests: new person unknown state, any-home-wins, grace period not elapsed stays home, grace period elapsed becomes away, duplicate name rejected, update_source clears away_since

## 2. Domain — Automation trigger extension

- [x] 2.1 Add `Trigger::PresenceChange { person_name: String, target_state: PresenceState }` to `domain/automation.rs`
- [x] 2.2 Add Display arm for `Trigger::PresenceChange`
- [x] 2.3 Update `evaluate_rules` signature to accept `presence: &PresenceRegistry` parameter
- [x] 2.4 Implement `PresenceChange` trigger evaluation: look up person by name, call `effective_state(now)`, compare to target
- [x] 2.5 Update ALL callers of `evaluate_rules` (CLI `src/cli.rs`, `infrastructure/automation_loop.rs`) to pass an empty/real `PresenceRegistry`
- [x] 2.6 Add unit tests for `PresenceChange` trigger: fires on match, does not fire on mismatch, does not fire for unknown person

## 3. Infrastructure — DB persistence

- [x] 3.1 Add `persons` table migration to `infrastructure/db.rs::migrate()` (before devices table): `id TEXT PK, name TEXT UNIQUE NOT NULL, grace_period_secs INT NOT NULL DEFAULT 120, sources JSONB NOT NULL DEFAULT '{}', away_since TIMESTAMPTZ, created_at TIMESTAMPTZ`
- [x] 3.2 Implement `db::upsert_person(pool, person)` using `sqlx::query!` with JSONB serialization for sources
- [x] 3.3 Implement `db::delete_person(pool, id)`
- [x] 3.4 Implement `db::load_all_persons(pool) -> Vec<PersonTracker>`

## 4. State — AppState wiring

- [x] 4.1 Add `pub presence: Arc<RwLock<PresenceRegistry>>` to `AppState` in `state.rs`
- [x] 4.2 Initialize `PresenceRegistry::new()` in `AppState::new()`
- [x] 4.3 Load persons from DB at server startup in `http/mod.rs::run_server_full` (same pattern as scripts/scenes)

## 5. HTTP — DTOs and handlers

- [x] 5.1 Add `PresenceStateStr`, `SourceStateStr` type aliases and `PersonResponse`, `CreatePersonRequest`, `UpdateSourceRequest` to `http/types.rs`
- [x] 5.2 Add `person_to_response(person, now) -> PersonResponse` helper to `http/helpers.rs`
- [x] 5.3 Create `backend/src/http/handlers/presence.rs` with: `list_persons`, `create_person` (201/409), `get_person` (404), `delete_person` (204/404), `update_source` (200/404)
- [x] 5.4 Register `pub mod presence` in `http/handlers/mod.rs`
- [x] 5.5 Add 5 prefixed routes to `http/router.rs` under `/api/presence/persons` (list, create, get, delete, update-source) plus legacy aliases

## 6. HTTP — Automation handler update

- [x] 6.1 Add `presence_change` variant to `TriggerInput` in `http/types.rs` with `person_name` and `target_state` fields
- [x] 6.2 Implement `TriggerInput::PresenceChange::to_domain()` conversion
- [x] 6.3 Update `TriggerInput` serialization arm in `trigger_to_response` helper
- [x] 6.4 Update `run_automation` handler to pass `state.presence.read().await` to `evaluate_rules`

## 7. Entity model — Person entity kind

- [x] 7.1 Add `EntityKind::Person` variant to `domain/device.rs` with Display `"person"`
- [x] 7.2 Update `GET /api/entities` handler (`http/handlers/entities.rs`) to include one entity per person from the `PresenceRegistry`: `kind=person, entity_id=person.<slug>, device_id=person.id, state=effective_state`
- [x] 7.3 Verify `?kind=person` filter works with new variant

## 8. OpenAPI contract

- [x] 8.1 Add `presence` tag to `contracts/openapi.yaml`
- [x] 8.2 Add schemas: `PresenceState`, `SourceState`, `Person`, `CreatePersonRequest`, `UpdateSourceRequest`
- [x] 8.3 Add paths: `GET/POST /api/presence/persons`, `GET/DELETE /api/presence/persons/{id}`, `PATCH /api/presence/persons/{id}/sources/{source}`
- [x] 8.4 Add `presence_change` trigger type to `TriggerType` enum and `Trigger` schema in OpenAPI

## 9. Frontend — Types and API client

- [x] 9.1 Add `PresenceState`, `SourceState`, `Person`, `CreatePersonRequest`, `UpdateSourceRequest` to `frontend/lib/api/types.ts`
- [x] 9.2 Create `frontend/lib/api/presence.ts` with `listPersons`, `createPerson`, `deletePerson`, `updateSource`
- [x] 9.3 Create `frontend/lib/hooks/use-presence.ts` SWR hook with `add`, `remove`, `updateSource` mutation helpers

## 10. Frontend — Presence page and components

- [x] 10.1 Create `frontend/components/presence/add-person-modal.tsx` with Name + Grace Period fields, error display on 409
- [x] 10.2 Create `frontend/app/presence/page.tsx` with person cards: name, effective_state badge, source breakdown, manual override buttons (Set Home / Set Away)
- [x] 10.3 Add `/presence` link to `frontend/components/layout/nav.tsx`

## 11. Integration tests

- [x] 11.1 Add HTTP integration tests for person CRUD (201, 409, 404, 204) in `backend/src/http/mod.rs`
- [x] 11.2 Add test for `update_source` endpoint: source update changes effective state
- [x] 11.3 Add test for `presence_change` automation trigger: rule fires on matching state, blocked on mismatch
- [x] 11.4 Add test for entity endpoint including person entities
- [x] 11.5 Run `cargo test` and `cargo clippy -- -D warnings`; fix all issues
