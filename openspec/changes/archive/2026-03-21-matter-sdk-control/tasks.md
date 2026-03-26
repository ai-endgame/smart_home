## 1. Infrastructure — chip-tool Sidecar

- [x] 1.1 Add `chip-tool` service to `docker-compose.yml` using `connectedhomeip/chip-tool` image with `/tmp/chip_tool_config` volume mount
- [x] 1.2 Add `make commission-sidecar` target to `Makefile` that starts the chip-tool container
- [x] 1.3 Create `backend/src/infrastructure/matter_control.rs` module
- [x] 1.4 Implement `run_chip_tool(args: &[&str]) -> Result<String, String>` — invokes `docker exec chip-tool chip-tool <args>` via `tokio::process::Command` with 10s timeout
- [x] 1.5 Implement `dispatch_onoff(node_id: u64, on: bool)` — calls `run_chip_tool` with `onoff on|off <node_id> 1`
- [x] 1.6 Implement `dispatch_level(node_id: u64, brightness: u8)` — scales 0–100 to 0–254 and calls `levelcontrol move-to-level`
- [x] 1.7 Implement `dispatch_color_temp(node_id: u64, mireds: u16)` — calls `colorcontrol move-to-color-temperature`
- [x] 1.8 Implement `read_onoff(node_id: u64) -> Result<bool, String>` — calls `chip-tool read on-off <node_id> 1`, parses output
- [x] 1.9 Implement `read_level(node_id: u64) -> Result<u8, String>` — calls `chip-tool read current-level <node_id> 1`, parses output, scales 0–254 to 0–100
- [x] 1.10 Add `pub mod matter_control;` to `backend/src/infrastructure/mod.rs`

## 2. Infrastructure — Commissioning

- [x] 2.1 Add `CommissionJob` struct: `job_id: String`, `status: CommissionStatus`, `message: String`, `device_id: Option<String>`, `error: Option<String>`
- [x] 2.2 Add `CommissionStatus` enum: `Pending`, `InProgress`, `Done`, `Failed`; derive `Serialize`, `Clone`
- [x] 2.3 Add `CommissionStore = Arc<RwLock<HashMap<String, CommissionJob>>>` to `AppState`
- [x] 2.4 Initialize `commission_jobs: Arc::new(RwLock::new(HashMap::new()))` in `AppState::new()`
- [x] 2.5 Implement `start_commission_job(store, setup_code, node_id, state)` — spawns `tokio::task`, runs `chip-tool pairing code <node_id> <setup_code>`, updates job status, adds device to SmartHome on success
- [x] 2.6 Add `node_id: Option<u64>` field to `Device` struct in `domain/device.rs` for storing the chip-tool node ID
- [x] 2.7 Add `ADD COLUMN IF NOT EXISTS node_id BIGINT` to `db.rs` migrate(), update SELECT, load, upsert for `node_id`

## 3. HTTP — Commissioning Handlers

- [x] 3.1 Create `backend/src/http/handlers/commission.rs`
- [x] 3.2 Implement `start_commission` handler — `POST /api/matter/commission`, validates setup code (11 digits), spawns job, returns `202` with `job_id`
- [x] 3.3 Implement `get_commission_job` handler — `GET /api/matter/commission/{job_id}`, returns job status
- [x] 3.4 Implement `list_commission_jobs` handler — `GET /api/matter/commission/jobs`, returns last 20 jobs
- [x] 3.5 Add `pub mod commission;` to `backend/src/http/handlers/mod.rs`
- [x] 3.6 Add `CommissionRequest` DTO to `types.rs`: `setup_code: String`, `node_id: u64`
- [x] 3.7 Add `CommissionJobResponse` DTO to `types.rs`: `job_id`, `status`, `message`, `device_id`, `error`
- [x] 3.8 Register `POST /api/matter/commission`, `GET /api/matter/commission/{job_id}`, `GET /api/matter/commission/jobs` in `router.rs` (plus legacy aliases)

## 4. HTTP — Cluster Command Dispatch

- [x] 4.1 In `backend/src/http/handlers/devices.rs` `set_device_state`: after state mutation, if `control_protocol == Matter`, spawn `matter_control::dispatch_onoff(node_id, on)`
- [x] 4.2 In `set_device_brightness`: if Matter, spawn `matter_control::dispatch_level(node_id, brightness)`
- [x] 4.3 In `set_device_temperature`: if Matter, spawn `matter_control::dispatch_color_temp(node_id, mireds)`
- [x] 4.4 On chip-tool error: update `device.last_error` and emit `device_error` SSE event
- [x] 4.5 In `send_device_command`: route Matter devices through cluster dispatch instead of MQTT publish

## 5. Infrastructure — State Sync Loop

- [x] 5.1 Add `sync_enabled: bool` and `last_sync_at: Option<String>` fields to `MatterStatus`
- [x] 5.2 Implement `start_matter_sync_loop(state: AppState)` in `matter_control.rs` — spawns `tokio::interval` task gated on `MATTER_SYNC_ENABLED=true`
- [x] 5.3 Sync loop: iterate SmartHome devices with `control_protocol == matter` and `node_id.is_some()`; call `read_onoff` and `read_level`; update state if changed
- [x] 5.4 Sync loop: on state change, call `persist_device` and emit `device_updated` SSE
- [x] 5.5 Sync loop: on chip-tool error, update `device.last_error`, continue to next device
- [x] 5.6 Call `start_matter_sync_loop` in `run_server_full()` after Matter scanner start
- [x] 5.7 Update `get_matter_status` handler to include `sync_enabled` and `last_sync_at` in response

## 6. OpenAPI Contract

- [x] 6.1 Add `CommissionRequest` schema to `contracts/openapi.yaml`
- [x] 6.2 Add `CommissionJobResponse` schema with `CommissionStatus` enum
- [x] 6.3 Add `POST /api/matter/commission`, `GET /api/matter/commission/{job_id}`, `GET /api/matter/commission/jobs` paths
- [x] 6.4 Update `MatterStatus` schema to include `sync_enabled` and `last_sync_at`
- [x] 6.5 Add `node_id` field to `Device` schema (integer, nullable)

## 7. Frontend — Device Control Modal

- [x] 7.1 Create `frontend/components/devices/device-control-modal.tsx` with `DeviceControlModal` component
- [x] 7.2 Modal: show device name, type badge, connected status, toggle button with optimistic update
- [x] 7.3 Modal: show brightness slider for `light` and `switch` device types (0–100 range)
- [x] 7.4 Modal: show color temperature slider for `light` device type (2700K–6500K range, converted to mireds)
- [x] 7.5 Modal: show "Attributes" section rendering all `device.attributes` key-value pairs; hide if empty
- [x] 7.6 Modal: show "Disconnected" warning badge and disable controls when `device.connected == false`
- [x] 7.7 Update `DeviceCard` to open `DeviceControlModal` on click instead of navigating
- [x] 7.8 Update `DeviceCard` to show green/grey state dot and amber disconnected ring

## 8. Frontend — Commissioning Wizard

- [x] 8.1 Create `frontend/components/devices/commission-modal.tsx` with 3-step `CommissionModal`
- [x] 8.2 Step 1: pairing code input (11-digit validation), "Start Commissioning" button
- [x] 8.3 Step 2: spinner + progress message + 60s countdown timer; poll `GET /api/matter/commission/{job_id}` every 2s
- [x] 8.4 Step 3 success: "Device added!" message + "Go to Devices" navigation button
- [x] 8.5 Step 3 failure: error message + "Try Again" button resetting to step 1
- [x] 8.6 Add `getMatterCommission`, `pollCommissionJob` fetch wrappers to `frontend/lib/api/matter.ts`
- [x] 8.7 Add `CommissionJobResponse` type to `frontend/lib/api/types.ts`

## 9. Frontend — Discovery Page Updates

- [x] 9.1 Update `frontend/app/discovery/page.tsx` to import and render `CommissionModal`
- [x] 9.2 Add "Commission" button to discovered device cards where `protocol == "matter"`
- [x] 9.3 Pass device name and id as initial values to `CommissionModal` when opened from Discovery

## 10. Tests

- [x] 10.1 Unit test: `dispatch_onoff` produces correct chip-tool args
- [x] 10.2 Unit test: `dispatch_level` scales brightness 0→0, 100→254, 80→203
- [x] 10.3 Integration test: `POST /api/matter/commission` with invalid code returns 400
- [x] 10.4 Integration test: `POST /api/matter/commission` with valid code returns 202 with job_id
- [x] 10.5 Integration test: `GET /api/matter/commission/{job_id}` returns 404 for unknown job
- [x] 10.6 Integration test: `GET /api/matter/commission/jobs` returns empty array initially
- [x] 10.7 Integration test: `GET /api/matter/status` includes `sync_enabled` and `last_sync_at` fields
