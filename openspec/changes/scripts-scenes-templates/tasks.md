## 1. Domain — Script entity

- [x] 1.1 Add `ScriptStep` enum (`SetState`, `SetBrightness`, `SetTemperature`, `Delay`, `ApplyScene`, `CallScript`) with serde in `domain/script.rs`
- [x] 1.2 Add `ScriptParam` struct (name, description, default value) and `Script` struct (id, name, description, params, steps)
- [x] 1.3 Add `ScriptRegistry` (HashMap keyed by UUID + name index) with `add`, `get`, `get_by_name`, `remove`, `list` methods
- [x] 1.4 Add unit tests for `ScriptRegistry`: duplicate name rejected, case-insensitive lookup

## 2. Domain — Scene entity

- [x] 2.1 Add `SceneState` struct (`state`, `brightness`, `temperature` all optional) and `Scene` struct (id, name, `HashMap<String, SceneState>`) in `domain/scene.rs`
- [x] 2.2 Add `SceneRegistry` with `add`, `get`, `get_by_name`, `remove`, `list` methods
- [x] 2.3 Add `SceneRegistry::apply(&scene, &mut SmartHome) -> (usize, Vec<String>)` method (partial apply, collect errors)
- [x] 2.4 Add unit tests: duplicate name conflict, partial apply with missing device returns error list

## 3. Domain — Template engine

- [x] 3.1 Add `infrastructure/template.rs` with `TemplateContext` struct (holds `&SmartHome` snapshot + current hour)
- [x] 3.2 Implement `eval_template(expr: &str, ctx: &TemplateContext) -> Result<TemplateValue, TemplateError>` — tokenizer + recursive evaluator for `{{ ... }}` blocks
- [x] 3.3 Support `state("name")`, `brightness("name")`, `now_hour()`, integer/float literals, `+`, `-`, `*`, `/`, `==`, `!=`, `>`, `<`, `>=`, `<=`
- [x] 3.4 Add unit tests: state lookup, brightness arithmetic, time comparison, unknown device returns error

## 4. Domain — Automation conditions & script_call action

- [x] 4.1 Add `Condition` enum (`StateEquals`, `BrightnessAbove`, `BrightnessBelow`, `TemplateEval`) in `domain/automation.rs`
- [x] 4.2 Add `conditions: Vec<Condition>` field to `AutomationRule`; update `add_rule` signature
- [x] 4.3 Add `Action::ScriptCall { script_name, args }` variant to the `Action` enum
- [x] 4.4 Update `evaluate_rules` to check all conditions (AND semantics) before including an action
- [x] 4.5 Update `execute_actions` to handle `Action::ScriptCall` (look up registry, spawn executor)
- [x] 4.6 Add unit tests: all conditions pass → action fires; one condition false → action skipped; empty conditions → always fires

## 5. Infrastructure — DB persistence (scripts & scenes)

- [x] 5.1 Add `scripts` table migration in `db.rs` (`id UUID PK`, `name TEXT UNIQUE`, `description TEXT`, `params JSONB`, `steps JSONB`, `created_at`)
- [x] 5.2 Add `scenes` table migration in `db.rs` (`id UUID PK`, `name TEXT UNIQUE`, `states JSONB`, `created_at`)
- [x] 5.3 Implement `db::upsert_script`, `db::delete_script`, `db::load_all_scripts`
- [x] 5.4 Implement `db::upsert_scene`, `db::delete_scene`, `db::load_all_scenes`

## 6. Infrastructure — Script executor

- [x] 6.1 Add `infrastructure/script_executor.rs` with `run_script(script, args, state, depth) -> Vec<String>`
- [x] 6.2 Implement step execution: `SetState`, `SetBrightness`, `SetTemperature` mutate `SmartHome` + persist via `persist_device`
- [x] 6.3 Implement `Delay` step: `tokio::time::sleep` capped at 60 seconds
- [x] 6.4 Implement `ApplyScene` step: look up scene registry and call `apply`
- [x] 6.5 Implement `CallScript` step: recursive call with `depth + 1`; return `ScriptError::MaxDepthExceeded` if depth ≥ 5
- [x] 6.6 Resolve template expressions in string/numeric fields before executing each step
- [x] 6.7 Add unit tests: max depth protection, delay capped at 60s, template resolution in steps

## 7. State — Wire registries into AppState

- [x] 7.1 Add `scripts: Arc<RwLock<ScriptRegistry>>` and `scenes: Arc<RwLock<SceneRegistry>>` fields to `AppState`
- [x] 7.2 Load scripts and scenes from DB in `run_server_full()` startup sequence (after devices load)

## 8. HTTP — Script handlers

- [x] 8.1 Create `http/handlers/scripts.rs` with `list_scripts`, `get_script`, `create_script`, `update_script`, `delete_script` handlers
- [x] 8.2 Add `create_script` handler: validate uniqueness, persist, return `201`; return `409` on duplicate name
- [x] 8.3 Add `run_script` handler (`POST /api/scripts/{id}/run`): spawn async task, return `202` immediately; return `404` for unknown ID
- [x] 8.4 Add HTTP types: `CreateScriptRequest`, `ScriptResponse`, `RunScriptRequest`, `RunScriptResponse` in `http/types.rs`

## 9. HTTP — Scene handlers

- [x] 9.1 Create `http/handlers/scenes.rs` with `list_scenes`, `get_scene`, `create_scene`, `update_scene`, `delete_scene` handlers
- [x] 9.2 Add `create_scene` handler: store, persist, `201`; `409` on duplicate
- [x] 9.3 Add `snapshot_scene` handler (`POST /api/scenes/snapshot`): read current device states for given IDs, create scene, return `201`
- [x] 9.4 Add `apply_scene` handler (`POST /api/scenes/{id}/apply`): call `SceneRegistry::apply`, return `{ applied, errors }`; `404` for unknown ID
- [x] 9.5 Add HTTP types: `CreateSceneRequest`, `SnapshotSceneRequest`, `SceneResponse`, `ApplySceneResponse` in `http/types.rs`

## 10. HTTP — Router & automation handler updates

- [x] 10.1 Register all script and scene routes in `router.rs` (with `/api/*` prefix and legacy aliases)
- [x] 10.2 Update automation `add_rule` handler to accept and map `conditions` array from request body
- [x] 10.3 Add `ConditionInput` enum and `to_domain()` converter in `http/types.rs`
- [x] 10.4 Update `RuleResponse` and `AddRuleRequest` in `http/types.rs` to include `conditions`

## 11. OpenAPI contract

- [x] 11.1 Add `Script`, `ScriptStep`, `ScriptParam`, `CreateScriptRequest`, `RunScriptRequest`, `RunScriptResponse` schemas to `contracts/openapi.yaml`
- [x] 11.2 Add `Scene`, `SceneState`, `CreateSceneRequest`, `SnapshotSceneRequest`, `SceneResponse`, `ApplySceneResponse` schemas
- [x] 11.3 Add all script and scene route paths (`/api/scripts`, `/api/scripts/{id}`, `/api/scripts/{id}/run`, `/api/scenes`, etc.)
- [x] 11.4 Extend `Condition` schema and add `conditions` field to `AutomationRule` and `CreateRuleRequest`
- [x] 11.5 Extend `ActionType` enum with `script_call`; add `script_name` and `args` fields to `Action` schema

## 12. Frontend — TypeScript types & API client

- [x] 12.1 Add `Script`, `ScriptStep`, `CreateScriptRequest`, `RunScriptRequest` types to `frontend/lib/api/types.ts`
- [x] 12.2 Add `Scene`, `SceneState`, `CreateSceneRequest`, `SnapshotSceneRequest`, `ApplySceneResponse` types
- [x] 12.3 Add `Condition` type and `conditions?: Condition[]` to `AutomationRule` and `CreateRuleRequest`
- [x] 12.4 Create `frontend/lib/api/scripts.ts` with `listScripts`, `createScript`, `deleteScript`, `runScript`
- [x] 12.5 Create `frontend/lib/api/scenes.ts` with `listScenes`, `createScene`, `deleteScene`, `applyScene`, `snapshotScene`
- [x] 12.6 Create `frontend/lib/hooks/use-scripts.ts` (SWR hook with mutate helpers)
- [x] 12.7 Create `frontend/lib/hooks/use-scenes.ts` (SWR hook with mutate helpers)

## 13. Frontend — Scripts page

- [x] 13.1 Create `frontend/app/scripts/page.tsx` with scripts list, run button, delete button
- [x] 13.2 Create `frontend/components/scripts/add-script-modal.tsx` — name, description, step builder (type selector + fields per step type)
- [x] 13.3 Add `/scripts` link to `frontend/components/layout/nav.tsx`

## 14. Frontend — Scenes page

- [x] 14.1 Create `frontend/app/scenes/page.tsx` with scenes list, apply button, delete button, applied count display
- [x] 14.2 Create `frontend/components/scenes/add-scene-modal.tsx` — name + device state map editor
- [x] 14.3 Add "Save as Scene" button to `DeviceControlModal` that calls `snapshotScene` for the current device
- [x] 14.4 Add `/scenes` link to `frontend/components/layout/nav.tsx`

## 15. Integration tests

- [x] 15.1 Add HTTP integration tests for script CRUD and `/run` endpoint (202, 404, 409) in `backend/src/http/mod.rs`
- [x] 15.2 Add HTTP integration tests for scene CRUD, `/snapshot`, and `/apply` (200, 404, 409, partial failure)
- [x] 15.3 Add test for automation with conditions: rule with failing condition does not fire
- [x] 15.4 Run `cargo test` and `cargo clippy -- -D warnings`; fix all issues
