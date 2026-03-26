## 1. SSE Broadcast Channel — AppState & record_event

- [x] 1.1 Add `use tokio::sync::broadcast;` and `pub events_tx: broadcast::Sender<ServerEvent>` to `AppState` in `backend/src/state.rs`; create channel with capacity 128 via `broadcast::channel(128)` in `AppState::new()`, storing only the `Sender` (discard the initial `Receiver`)
- [x] 1.2 In `backend/src/http/helpers.rs`, after writing to `state.events`, call `let _ = state.events_tx.send(event.clone());` so every `record_event` fans out to SSE subscribers
- [x] 1.3 Derive (or verify) `Clone` on `ServerEvent` in `backend/src/http/types.rs` — needed for broadcast

## 2. SSE Endpoint — Backend

- [x] 2.1 Add `pub async fn event_stream(State(state): State<AppState>) -> impl IntoResponse` to `backend/src/http/handlers/system.rs`; subscribe with `state.events_tx.subscribe()`, build an `axum::response::sse::Sse` stream that yields each event as `Event::default().data(serde_json::to_string(&ev).unwrap_or_default())`, with `.keep_alive(KeepAlive::default())` and retry set to 3 s
- [x] 2.2 Register `GET /api/events/stream` in `backend/src/http/router.rs` (no legacy alias needed — this is a new endpoint)
- [ ] 2.3 Add `GET /api/events/stream` to `contracts/openapi.yaml` under the `system` tag

## 3. Device State History — Domain & AppState

- [x] 3.1 Add `HistoryEntry` struct to `backend/src/domain/device.rs`: `pub struct HistoryEntry { pub timestamp: DateTime<Utc>, pub state: DeviceState, pub brightness: u8, pub temperature: Option<f64> }` with `#[derive(Debug, Clone, Serialize, Deserialize)]`
- [x] 3.2 Add `pub const MAX_HISTORY_PER_DEVICE: usize = 500;` to `backend/src/state.rs` and `pub history: Arc<RwLock<HashMap<String, VecDeque<HistoryEntry>>>>` to `AppState`; initialize to `Arc::new(RwLock::new(HashMap::new()))` in `AppState::new()`
- [x] 3.3 Add `pub async fn record_history(state: &AppState, device_name: &str, entry: HistoryEntry)` to `backend/src/http/helpers.rs`: acquire write lock on `state.history`, get or insert the `VecDeque` for the device name (lowercased), pop_front if len ≥ MAX_HISTORY_PER_DEVICE, then push_back the entry

## 4. Device State History — HTTP Handlers

- [x] 4.1 Call `record_history` in `set_device_state` handler (`backend/src/http/handlers/devices.rs`) immediately after the state mutation succeeds, building a `HistoryEntry` from the updated device
- [x] 4.2 Call `record_history` in `set_device_brightness` handler similarly
- [x] 4.3 Call `record_history` in `set_device_temperature` handler similarly
- [x] 4.4 Add `pub async fn get_device_history` handler to `backend/src/http/handlers/devices.rs`: accepts optional `?limit=N` query param; reads history map; returns 404 if device not found (check `state.home`), otherwise returns the history slice as JSON
- [x] 4.5 Add `HistoryQuery { limit: Option<usize> }` struct to `backend/src/http/types.rs`
- [x] 4.6 Register `GET /api/devices/{name}/history` in `backend/src/http/router.rs` and add the legacy alias `/devices/{name}/history`
- [ ] 4.7 Add `GET /api/devices/{name}/history` path and `HistoryEntry` schema to `contracts/openapi.yaml`

## 5. Backup Restore — Backend

- [x] 5.1 Add `pub async fn restore_backup` handler to `backend/src/http/handlers/system.rs`; accept `Json<BackupDocument>` body; acquire write locks in canonical order (home → automation → scripts → scenes → presence → dashboard), clear each, re-import from backup fields, release; if DB present, delete all and re-insert; return `Json(json!({"restored": {...counts}}))`
- [x] 5.2 After restore completes, call `record_event` with `EventKind::Server` and a message like `"restore: N devices, M rules, ..."`
- [x] 5.3 Register `POST /api/restore` in `backend/src/http/router.rs`
- [ ] 5.4 Add `POST /api/restore` path and response schema to `contracts/openapi.yaml`
- [x] 5.5 Add integration test `restore_clears_and_reimports`: create device + rule → POST /api/restore with different backup → verify original device gone, backup device present

## 6. Outbound Webhook — Domain

- [x] 6.1 Add `pub notify_url: Option<String>` to `AutomationRule` in `backend/src/domain/automation.rs`; initialize to `None` in `add_rule`
- [x] 6.2 Add `notify_url: Option<String>` to `AddRuleRequest` in `backend/src/http/types.rs` and `notify_url: Option<String>` to `RuleResponse`
- [x] 6.3 Update `rule_to_response` in `backend/src/http/helpers.rs` to copy `rule.notify_url.clone()` into `RuleResponse`
- [x] 6.4 Update `add_rule` handler in `backend/src/http/handlers/automation.rs` to pass `payload.notify_url` through to the domain

## 7. Outbound Webhook — Dispatch

- [x] 7.1 Add `pub async fn dispatch_webhook(url: &str, rule_name: &str, message: &str)` to a new `backend/src/infrastructure/webhook.rs`; POST JSON `{"rule": rule_name, "message": message, "timestamp": Utc::now().to_rfc3339()}` to `url` with a 5-second timeout using `reqwest`; log WARN on error
- [x] 7.2 Add `pub mod webhook;` to `backend/src/infrastructure/mod.rs`
- [x] 7.3 In `fire_rule` (`backend/src/infrastructure/automation_loop.rs`): after `execute_actions` returns, if the rule has a `notify_url` and the returned notifications vec is non-empty, spawn a `dispatch_webhook` task
- [x] 7.4 In the time-based automation loop in `automation_loop.rs`: after evaluate_rules, for each triggered rule that has a `notify_url`, spawn `dispatch_webhook` for each notification message
- [x] 7.5 In `run_automation` HTTP handler (`backend/src/http/handlers/automation.rs`): after `execute_actions`, similarly dispatch webhooks for rules with `notify_url`
- [ ] 7.6 Add `POST /api/automation/rules/{name}/safe-mode` to `contracts/openapi.yaml` if not done; add `notify_url` field to `AutomationRule` schema in `contracts/openapi.yaml`

## 8. Frontend — SSE Live Updates

- [x] 8.1 Add `frontend/lib/api/events.ts` with `createEventSource(): EventSource` that returns `new EventSource('/api/events/stream')`
- [x] 8.2 Add `frontend/lib/hooks/use-sse-events.ts`: opens the EventSource, parses each message as `ServerEvent`, and calls a provided `onEvent(ev: ServerEvent) => void` callback; cleans up on unmount
- [x] 8.3 Update `frontend/lib/hooks/use-devices.ts`: integrate `use-sse-events` so that when a `device_updated` event arrives, call `mutate()` to refresh devices; remove or lower the `refreshInterval`
- [x] 8.4 Update `frontend/lib/hooks/use-automation.ts`: call `mutate()` on `automation` events from SSE

## 9. Frontend — Restore Button

- [x] 9.1 Add `restoreBackup(file: File): Promise<void>` to `frontend/lib/api/system.ts`: reads the file as text, POSTs to `/api/restore` with `Content-Type: application/json`, returns the restored counts
- [x] 9.2 Add a "Restore Backup" file-input button to `frontend/app/page.tsx` next to "Download Backup"; on file selection, call `restoreBackup` and show a success/error toast

## 10. Integration Tests & Build

- [x] 10.1 Add unit test `sse_broadcast_reaches_record_event`: call `record_event` on a test `AppState`, subscribe before the call, assert the receiver gets the event
- [x] 10.2 Add integration test `device_history_records_state_changes`: PATCH state twice → GET /api/devices/{name}/history → assert 2 entries in order
- [x] 10.3 Add integration test `restore_clears_and_reimports` (from task 5.5)
- [x] 10.4 Add integration test `notify_url_rule_creation`: POST rule with notify_url → GET rules → assert notify_url present in response
- [x] 10.5 Run `cargo test` and `cargo clippy -- -D warnings`; fix all issues
- [x] 10.6 Run `cd frontend && npm run build`; fix all TypeScript errors
