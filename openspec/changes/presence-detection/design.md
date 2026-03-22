## Context

The smart home system currently has no concept of human presence. Automations must rely on device state proxies (e.g., lights on = someone home). Session 7 of the mastery plan introduces proper presence detection. The existing architecture is well-suited: `AppState` holds registries keyed by `Arc<RwLock<…>>`, the automation engine evaluates triggers against a snapshot of the home, and the HTTP layer is thin handlers over domain logic.

## Goals / Non-Goals

**Goals:**
- Model presence as a first-class domain entity (`PersonTracker`) with multi-source evidence aggregation
- Implement a grace-period debounce so momentary signal loss doesn't trigger "away"
- Expose full CRUD REST API for persons and source updates
- Add `PresenceChange` automation trigger
- Add a `/presence` frontend page with manual override
- Persist persons to the DB (`persons` table)

**Non-Goals:**
- Actual network-ping infrastructure (sources are updated via API; a real ping loop is out of scope for this session)
- BLE scanning, GPS integration, or HA Companion App webhooks (future sessions)
- Multi-user conflict resolution or presence zones beyond `home`/`away`

## Decisions

### D1: Source-aggregation strategy — "any home wins"

**Decision:** A person is `home` if **any** source reports `home`. They become `away` only when **all** sources report `away` (or `unknown`) AND the grace period has elapsed.

**Rationale:** False negatives (thinking someone left when they didn't) are worse UX than false positives. A single strong signal (e.g., router ping) should override weak `unknown` readings.

**Alternatives considered:**
- Majority vote — too complex for 2–3 sources; edge cases with even counts.
- Weighted scoring — over-engineered for this stage.

### D2: Grace period stored in-domain, not in config

**Decision:** Each `PersonTracker` has a `grace_period_secs: u32` field (default 120). Stored as part of the `PersonTracker` struct and persisted in the DB `persons` table as a JSON column.

**Rationale:** Different persons may have different grace periods (e.g., a child tracked via BLE needs a shorter window than GPS-based adult tracker). Putting it in domain keeps logic cohesive and testable.

**Alternatives considered:**
- Global server config — inflexible, doesn't allow per-person tuning.

### D3: Grace-period tracking — wall-clock timestamps in domain struct

**Decision:** `PersonTracker` stores `away_since: Option<DateTime<Utc>>`. Set when all sources go `away`/`unknown`; cleared when any source returns `home`. The `effective_state()` method computes the public-facing state using this timestamp.

**Rationale:** Avoids a separate timer infrastructure. `effective_state()` is pure (takes `now: DateTime<Utc>`) — easy to unit test.

### D4: DB persistence — JSONB for sources

**Decision:** `persons` table: `id TEXT PK, name TEXT UNIQUE, grace_period_secs INT, sources JSONB, away_since TIMESTAMPTZ, created_at TIMESTAMPTZ`. Sources serialized as `HashMap<String, SourceState>` JSON.

**Rationale:** Sources are sparse and schema-flexible (future source types shouldn't require migrations). Consistent with scripts/scenes JSONB pattern.

### D5: Presence trigger evaluation — snapshot-based

**Decision:** `evaluate_rules` receives the effective `PresenceState` per person via a `&PresenceRegistry` parameter added to the signature, not a second `&SmartHome`.

**Rationale:** `SmartHome` only knows devices. Presence is orthogonal. Keeping registries separate avoids polluting the device domain.

**Alternatives considered:**
- Store presence in `SmartHome` — mixes device and person concerns.
- Pass a `HashMap<String, PresenceState>` snapshot — less type-safe, duplicates registry logic.

## Risks / Trade-offs

- **Grace period race on restart**: `away_since` is persisted, so a server restart during a grace window will resume correctly. If the DB is unavailable, the in-memory state starts fresh (all `unknown`) — persons appear `unknown` until sources update. This is acceptable.
- **Stale source readings**: There's no TTL per source. A source set to `home` and never updated stays `home` forever. Callers must explicitly set sources `away`. A future enhancement could add `last_seen` TTL eviction.
- **evaluate_rules signature change**: Adding `presence: &PresenceRegistry` to `evaluate_rules` is a breaking internal API change. All callers (CLI and `automation_loop`) must be updated. Low risk since both are internal.

## Migration Plan

1. Add `persons` table migration in `db.rs::migrate()` before the devices table block.
2. Load persons at server startup in `http/mod.rs::run_server_full`, same pattern as scripts/scenes.
3. All new routes are additive under `/api/presence/…`; no existing routes change.
4. Rollback: drop `persons` table; revert source files. No data loss to existing devices/automations.
