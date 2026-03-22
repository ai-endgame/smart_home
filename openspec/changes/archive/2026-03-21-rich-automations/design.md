## Context

The current `AutomationEngine::evaluate_rules(&self, home: &SmartHome) -> Vec<Action>` is stateless with respect to time — it has no way to evaluate time or sun triggers. The engine is only called manually via `POST /api/automation/run`. There is no background loop. The existing `chrono` crate is already a dependency.

## Goals / Non-Goals

**Goals:**
- Extend the `Trigger` enum with `Time`, `Sun`, `NumericStateAbove`, `NumericStateBelow`, `Webhook` variants
- Extend the `Action` enum with `Notify { message }` variant
- Add `time_range: Option<(String, String)>` to `AutomationRule` (from/to as "HH:MM" strings)
- Update `evaluate_rules` to accept `now: chrono::NaiveTime` so time-matching triggers can be evaluated
- Add `execute_actions` return value `Vec<String>` for notification messages
- Add `infrastructure/sun.rs` — reads `SUNRISE_TIME`/`SUNSET_TIME` env vars with Indonesia defaults
- Add `infrastructure/automation_loop.rs` — 60s tokio interval task evaluating time/sun triggers
- Add webhook handler and route: `POST /api/automations/webhook/{rule_name}`
- Frontend: new trigger/action inputs in AddRuleModal, 3 starter templates

**Non-Goals:**
- Actual solar position calculation (use configurable env var times only)
- Cron-syntax scheduling
- Rule persistence to database (still in-memory)
- Multi-action sequences per rule (Session 6 scope)
- "Choose" branching actions (Session 6 scope)

## Decisions

### D1 — evaluate_rules takes `now: NaiveTime`

**Decision:** Change signature to `evaluate_rules(&self, home: &SmartHome, now: chrono::NaiveTime) -> Vec<Action>` and pass `chrono::Local::now().time()` from the caller.

**Rationale:** Pure function — no hidden `Utc::now()` inside domain logic. Testable with any arbitrary time. Callers (handler + background loop) both supply the current time.

**Impact:** All callers updated. Unit tests pass in an explicit time.

### D2 — Webhook trigger fires only via HTTP, never via auto-eval loop

**Decision:** `Trigger::Webhook` always evaluates to `false` inside `evaluate_rules`. It fires exclusively when the `POST /api/automations/webhook/{rule_name}` endpoint is called, which executes the rule's action directly.

**Rationale:** The loop evaluating webhook triggers periodically would be meaningless. Webhook rules are event-driven by external callers.

### D3 — Sun times via env vars, not astronomical calculation

**Decision:** `SUNRISE_TIME` (default `"06:00"`) and `SUNSET_TIME` (default `"18:00"`) env vars. `Sun { event, offset_minutes }` computes target = base_time + offset, compares to current minute.

**Rationale:** Accurate solar calculation would require a crate (~50 kB) and lat/lon config. For a learning project in Malang (~7°S), fixed defaults are close enough and the user can override. This keeps the dependency footprint zero.

### D4 — execute_actions returns Vec<String> for notify messages

**Decision:** `execute_actions(actions, home) -> Vec<String>` where each `Notify` action appends its message. Callers emit one SSE `automation` event per message string.

**Rationale:** Keeps domain layer pure — no SSE or AppState references inside `execute_actions`. The HTTP handler and background loop both already have access to AppState for SSE emission.

### D5 — time_range stored as (String, String) not NaiveTime pair

**Decision:** Store `time_range` as `Option<(String, String)>` where each string is "HH:MM". Parse with `chrono::NaiveTime::parse_from_str` at evaluation time.

**Rationale:** Simpler serialization (serde derives work on String out-of-the-box). Parse errors at evaluation time are non-fatal (skip condition check, log warning).

### D6 — Background loop evaluates ALL enabled rules, not just time/sun ones

**Decision:** The background loop calls `evaluate_rules(home_snapshot, now)` which naturally returns only triggered actions. All triggers evaluate, but only Time/Sun will match based on time. Webhook returns false. State/numeric triggers will re-trigger if device state happens to match — this is intentional (same as manual run).

**Rationale:** Keeps evaluate_rules as a single entry point with no special-casing in the loop. Users expecting "only fire once" behavior should use the webhook pattern.

## Risks / Trade-offs

- **[Risk] Time trigger fires every minute the condition is true** — e.g. `Time { time: "22:00" }` fires every loop iteration during the 22:00 minute (up to 60 times in theory, once per loop). Mitigation: the loop interval is 60s and checks are at the start of the minute; the trigger fires once per matching minute. Document this behavior.
- **[Risk] evaluate_rules signature change breaks existing callers** — the handler `run_automation` and any test that calls `evaluate_rules` must be updated. These are all internal; no external callers. Mitigation: compile errors will catch all sites.
- **[Trade-off] Sun trigger uses fixed times not actual sun position** — acceptable for a learning project; real solar calculation is a Session 10 upgrade path.

## Migration Plan

1. All changes are additive to existing domain structs — no existing triggers or actions are modified
2. Existing `run_automation` handler updated to pass `chrono::Local::now().time()` to `evaluate_rules`
3. Background loop started after server init — no config needed beyond `SUNRISE_TIME`/`SUNSET_TIME` env vars
4. Rollback: remove the loop call from `run_server_full`, remove the new handler from router

## Open Questions

- Should the automation loop be opt-in (`AUTO_EVAL=true` env var) or always-on? → Decision: always-on when server runs with `run_server_full`; loop only fires time/sun triggers so it's harmless when no such rules exist.
