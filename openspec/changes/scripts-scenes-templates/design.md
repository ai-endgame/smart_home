## Context

The backend currently has `AutomationRule` (trigger + action, single pair) in `domain/automation.rs` and a flat `SmartHome` in-memory store. There are no script or scene abstractions. Automation conditions and action values are all static (hard-coded at rule creation time). The DB has a `devices` table; automations live only in memory. The frontend has a basic automations list page with toggle/delete but no editor for complex rules.

## Goals / Non-Goals

**Goals:**
- Introduce `Script` domain entity: named, has parameter slots, has a list of `ScriptStep` (state/brightness/temperature/delay/scene_apply)
- Introduce `Scene` domain entity: named, has a `HashMap<device_id, SceneState>` snapshot; `apply()` iterates and mutates SmartHome
- Introduce `TemplateExpr`: parses `{{ expr }}` tokens in string fields, evaluates against a `TemplateContext` (device states, current time)
- Extend `AutomationRule` with `conditions: Vec<Condition>` (state_equals, template_eval) and a new `script_call` action variant
- Persist scripts and scenes to Postgres (two new tables, JSON steps/states columns)
- REST CRUD for scripts and scenes; one-shot `POST /api/scripts/{id}/run` and `POST /api/scenes/{id}/apply` endpoints
- Frontend: `/scripts` and `/scenes` management pages; DeviceControlModal "Save as Scene" shortcut

**Non-Goals:**
- Full Jinja2 compatibility — only `{{ expr }}` arithmetic and comparison tokens, no loops/macros
- Script scheduling (cron-based triggers) — out of scope for this session
- Script versioning or rollback
- Real-time script execution log streaming (errors only, via last_error field)

## Decisions

### D1 — Template engine: custom mini-evaluator, not a Jinja2 crate

**Decision:** Implement a 50-line recursive evaluator for `{{ expr }}` patterns rather than pulling in `minijinja` or `tera`.

**Rationale:** The only template use-cases needed are:
- `{{ device("living_room_light").brightness + 10 }}`
- `{{ now().hour >= 22 }}`
- `{{ state("sensor_1") == "on" }}`

A full Jinja2 crate adds 200 kB and a dependency chain that's hard to audit. The mini-evaluator lives in `infrastructure/template.rs`, is tested in isolation, and can be replaced later.

**Alternative considered:** `minijinja` — ruled out for binary size and overkill for this scope.

### D2 — Scene state storage: JSON column, not normalized rows

**Decision:** Store `HashMap<device_id, SceneState>` as a `JSONB` column in the `scenes` table rather than a join table `scene_device_states`.

**Rationale:** Scenes are small (≤ 50 devices) and always read/written atomically. JSONB avoids a N-query fetch per scene apply. The structure is already `serde`-friendly. Querying individual device values inside scenes is not needed.

### D3 — Script steps: typed enum, not a DSL string

**Decision:** `ScriptStep` is a Rust enum (`SetState`, `SetBrightness`, `SetTemperature`, `Delay`, `ApplyScene`, `CallScript`) serialized to JSON, not a mini-language string.

**Rationale:** Keeps validation at the Rust type level. Frontend can send structured JSON; no parsing errors at runtime. `CallScript` enables script composition (calls another script) with a depth limit of 5 to prevent infinite recursion.

### D4 — Automation conditions: evaluated before action dispatch

**Decision:** `Vec<Condition>` is checked synchronously after trigger fires; if any condition fails the rule is skipped. Conditions are: `StateEquals { device, state }`, `BrightnessAbove/Below { device, value }`, `TemplateEval { expr }`.

**Rationale:** Mirrors HA's condition model exactly. No new async path needed — condition evaluation is pure reads against SmartHome snapshot.

### D5 — Persistence: scripts/scenes in DB; automation conditions in-memory only

**Decision:** Scripts and scenes are persisted to DB (they are user-authored assets). Automation rule conditions are NOT persisted to DB in this session — they extend the in-memory `AutomationRule` struct only.

**Rationale:** Automation DB persistence is not yet implemented (rules live in memory on the current branch). Adding conditions persistence alongside full automation persistence would be a larger scope jump. This session adds the capability; DB persistence for automations can be a follow-up.

## Risks / Trade-offs

- **[Risk] Template eval security** — user-supplied expressions are evaluated at runtime. Mitigation: expression input is sandboxed (no filesystem/network access, no `std` calls); evaluated inside a function with no closures over external state; depth-limited recursion.
- **[Risk] Script infinite recursion via `CallScript`** → Mitigation: enforce `max_depth = 5`, return `ScriptError::MaxDepthExceeded` if exceeded.
- **[Risk] Scene apply partial failure** — if a device is offline mid-apply, remaining devices still proceed. Mitigation: collect errors into `Vec<String>` and return them in the response; never abort early.
- **[Trade-off] Conditions not persisted** — means a server restart loses any conditions added via API. This is acceptable because automations themselves aren't persisted yet; both need a full automation persistence pass.

## Migration Plan

1. Add `scripts` and `scenes` tables in `db.rs` `migrate()` (both use `IF NOT EXISTS`)
2. Load scripts and scenes into `AppState` on server start
3. All new tables are additive — no existing schema changes, no data migration needed
4. Rollback: drop the two new tables; remove the two new handler modules from `router.rs`

## Open Questions

- Should `ScriptStep::Delay` use wall-clock `tokio::time::sleep` (async) or be a no-op in the sync evaluator? → Decision: async sleep in the script executor (scripts run in a spawned task), capped at 60s per step.
- Should scenes support partial apply (only specified devices) or always full? → Decision: partial apply — only devices listed in the scene snapshot are touched; others are left as-is.
