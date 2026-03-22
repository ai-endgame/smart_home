## 1. Config & AppState — API key plumbing

- [x] 1.1 Add `api_key: Option<String>` to `Config` in `backend/src/config.rs`; read from `API_KEY` env var (empty string treated as None)
- [x] 1.2 Add `api_key: Option<String>` to `AppState` in `backend/src/state.rs`; initialize from `Config` in `run_server_full`

## 2. API key auth middleware

- [x] 2.1 Add `auth_middleware` async function to `backend/src/http/middleware.rs`: extract `api_key` from `AppState`; if `None`, call `next.run(req)` unchanged; if `Some(key)`, check method — GET/OPTIONS pass through; POST/PATCH/DELETE/PUT check `Authorization: Bearer <key>` or `X-API-Key: <key>` header; return 401 `ApiError::Unauthorized` on mismatch
- [x] 2.2 Add `ApiError::Unauthorized(String)` variant to `backend/src/http/errors.rs` → 401 status, `"code": "unauthorized"`
- [x] 2.3 Register auth middleware on the router in `backend/src/http/router.rs` using `.layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth_middleware))` inside the CORS layer (CORS must be outermost)
- [x] 2.4 Add unit tests in `backend/src/http/mod.rs` for auth middleware: correct key → 200, wrong key → 401, missing header → 401, GET with key set → 200 (no header needed), API_KEY absent → 200 (no header needed)

## 3. Request body size limit

- [x] 3.1 Add `"limit"` to `tower-http` features in `backend/Cargo.toml`
- [x] 3.2 Add `RequestBodyLimitLayer::new(64 * 1024)` to the router in `router.rs` as the outermost non-CORS layer (applies before auth and handlers)

## 4. Input name/title length validation

- [x] 4.1 Add `pub const MAX_NAME_LEN: usize = 120;` to `backend/src/http/helpers.rs` and a helper `fn validate_name(s: &str) -> Result<(), ApiError>` that returns `ApiError::BadRequest` if `s.chars().count() > MAX_NAME_LEN`
- [x] 4.2 Call `validate_name(&name)` in `create_dashboard` handler (backend/src/http/handlers/dashboards.rs) and `add_view` handler
- [x] 4.3 Call `validate_name` in device creation handler (`backend/src/http/handlers/devices.rs`) for `name` field
- [x] 4.4 Call `validate_name` in `create_person` handler (`backend/src/http/handlers/presence.rs`) for `name` field
- [x] 4.5 Call `validate_name` in script and scene handlers (`scripts.rs`, `scenes.rs`) for `name` fields
- [x] 4.6 Call `validate_name` in automation rule creation (`automation.rs`) for `name` field
- [x] 4.7 Add unit test: name at limit (120 chars) → 201; name 121 chars → 400

## 5. State backup endpoint

- [x] 5.1 Add `BackupDocument` struct to `backend/src/http/types.rs`: `version: &'static str`, `exported_at: DateTime<Utc>`, `devices: Vec<DeviceResponse>`, `automation_rules: Vec<RuleResponse>`, `scripts: Vec<Script>`, `scenes: Vec<Scene>`, `persons: Vec<PersonTracker>`, `dashboards: Vec<Dashboard>`
- [x] 5.2 Add `get_backup` handler to `backend/src/http/handlers/system.rs`: acquire read locks on all registries, build `BackupDocument`, return `Json(doc)`
- [x] 5.3 Register `GET /api/backup` route in `router.rs`
- [x] 5.4 Add `GET /api/backup` path + `BackupDocument` schema to `contracts/openapi.yaml`
- [x] 5.5 Add HTTP integration test `backup_returns_all_state`: create a device + script + scene + person + dashboard → GET /api/backup → 200 with non-empty arrays for all fields

## 6. Automation safe mode — domain

- [x] 6.1 Add `pub safe_mode: bool` field to `AutomationRule` struct in `backend/src/domain/automation.rs` with `#[serde(default)]`; initialize as `false` in `AutomationEngine::add_rule`
- [x] 6.2 Add `AutomationEngine::toggle_safe_mode(name) -> Result<bool, DomainError>` method (same pattern as `toggle_rule`)
- [x] 6.3 Update `execute_actions(actions, home, safe_mode: bool)` signature: when `safe_mode` is `true`, skip `Action::SetState`, `Action::SetBrightness`, `Action::SetTemperature`, `Action::Lock`/`Action::Unlock` variants and call `log::warn!("safe_mode: suppressing actuator action {:?}", action)` for each
- [x] 6.4 Update all callers of `execute_actions` (CLI in `cli.rs`, automation loop in `infrastructure/automation_loop.rs`, HTTP handler in `http/handlers/automation.rs`) to pass `rule.safe_mode`
- [x] 6.5 Add `safe_mode` field to `RuleResponse` in `http/types.rs` (already has `enabled`, add `safe_mode: bool` alongside it)
- [x] 6.6 Add unit tests in `domain/automation.rs`: safe mode skips state action; safe mode allows notify action; toggle_safe_mode flips the flag

## 7. Automation safe mode — HTTP

- [x] 7.1 Add `toggle_safe_mode` handler to `backend/src/http/handlers/automation.rs`: `POST /api/automation/rules/{name}/safe-mode`; calls `engine.toggle_safe_mode(name)`, persists nothing (in-memory only); returns `Json(json!({"name": name, "safe_mode": new_val}))` or 404
- [x] 7.2 Register `POST /api/automation/rules/{name}/safe-mode` route in `router.rs` and add legacy alias
- [x] 7.3 Update `contracts/openapi.yaml`: add `safe_mode: boolean` to the Rule schema; add `POST /api/automation/rules/{name}/safe-mode` path

## 8. Frontend — backup download button

- [x] 8.1 Add `downloadBackup()` function to `frontend/lib/api/` (or inline in the component): fetches `GET /api/backup`, creates a Blob, triggers browser download of `smart-home-backup-<date>.json`
- [x] 8.2 Add a "Download Backup" button to `frontend/app/page.tsx` (dashboard overview) or a dedicated System section; clicking it calls `downloadBackup()`

## 9. Frontend — safe mode toggle in automation UI

- [x] 9.1 Add a "Safe" badge/toggle to the automation rule card in `frontend/app/automation/page.tsx`: when `rule.safe_mode` is true, display an amber "SAFE" badge next to the rule name; add a button to call `POST /api/automation/rules/{name}/safe-mode` and refresh

## 10. Integration tests & build

- [x] 10.1 Add test `auth_middleware_blocks_writes_when_key_set` in `backend/src/http/mod.rs`
- [x] 10.2 Add test `auth_middleware_allows_gets_without_key`
- [x] 10.3 Add test `backup_returns_all_state`: create device, script, person, dashboard → GET /api/backup → 200, verify arrays non-empty
- [x] 10.4 Add test `safe_mode_toggle_and_suppression`: create rule with state action, toggle safe mode → run automation → device state unchanged
- [x] 10.5 Run `cargo test` and `cargo clippy -- -D warnings`; fix all issues
