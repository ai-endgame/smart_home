## 1. Domain — ThreadRole & MatterFabric

- [x] 1.1 Add `ThreadRole` enum to `backend/src/domain/device.rs` with variants `BorderRouter`, `Router`, `EndDevice`, `Sleepy`; derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`
- [x] 1.2 Add `Display` impl for `ThreadRole` returning lowercase string (`border_router`, `router`, `end_device`, `sleepy`)
- [x] 1.3 Add `ThreadRole::from_txt_bitmask(bits: u16) -> Option<ThreadRole>` — maps `_T` TXT field bitmask (bit 5 = BorderRouter, others = Router/EndDevice)
- [x] 1.4 Add `MatterFabric` struct: `fabric_id: String`, `vendor_id: u16`, `commissioner: String`; derive `Serialize`, `Deserialize`, `Clone`, `Debug`
- [x] 1.5 Add `thread_role: Option<ThreadRole>` and `matter_fabric: Option<MatterFabric>` fields to `Device` struct
- [x] 1.6 Update `Device::new()` to initialize both fields to `None`
- [x] 1.7 Export `ThreadRole`, `MatterFabric` via `pub use` in `backend/src/domain/mod.rs`
- [x] 1.8 Add unit tests for `ThreadRole::Display` and `ThreadRole::from_txt_bitmask`

## 2. Infrastructure — DB

- [x] 2.1 Add `ADD COLUMN IF NOT EXISTS thread_role TEXT` to `db.rs` `migrate()`
- [x] 2.2 Add `ADD COLUMN IF NOT EXISTS matter_fabric TEXT` to `db.rs` `migrate()`
- [x] 2.3 Update SELECT query in `load_all_devices` to include `thread_role` and `matter_fabric`
- [x] 2.4 Update Device construction in `load_all_devices` to parse `thread_role` via `ThreadRole::from_str` and `matter_fabric` via `serde_json::from_str`
- [x] 2.5 Update `upsert_device` INSERT/UPDATE to include `thread_role` and `matter_fabric` columns

## 3. Infrastructure — Matter Scanner

- [x] 3.1 Create `backend/src/infrastructure/matter.rs`
- [x] 3.2 Add `MatterStatus` struct: `devices_seen: u64`, `commissioning_count: u64`, `last_seen_at: Option<String>`; derive `Serialize`, `Clone`, `Default`
- [x] 3.3 Add `MatterStatusStore = Arc<RwLock<MatterStatus>>` type alias and `new_status_store()` constructor
- [x] 3.4 Implement `start_matter_scanner(store: DiscoveryStore, status: MatterStatusStore)` — spawns a `std::thread` that browses `_matter._tcp.local.` and `_matterc._tcp.local.`
- [x] 3.5 Implement `handle_matter_event` — on `ServiceResolved`: parse TXT fields `D`, `VP`, `CM`, `RI`, `_T`; build `DiscoveredDevice` with `protocol: Some(Protocol::Matter)` and metadata in `properties`
- [x] 3.6 On each resolved device: update `MatterStatus.devices_seen`, `last_seen_at`; increment `commissioning_count` if `CM != 0`
- [x] 3.7 Add `pub mod matter;` to `backend/src/infrastructure/mod.rs`

## 4. State

- [x] 4.1 Add `matter_status: matter::MatterStatusStore` to `AppState` in `backend/src/state.rs`
- [x] 4.2 Import `crate::infrastructure::matter` in `state.rs`
- [x] 4.3 Initialize `matter_status: matter::new_status_store()` in `AppState::new()`

## 5. HTTP — Server Init

- [x] 5.1 In `backend/src/http/mod.rs` `run_server_full()`: call `matter::start_matter_scanner(state.discovery.clone(), state.matter_status.clone())` after mDNS start

## 6. HTTP — Handlers

- [x] 6.1 Create `backend/src/http/handlers/matter.rs` with `get_matter_status` handler — returns `AppState.matter_status` as JSON
- [x] 6.2 Add `list_matter_devices` handler — reads `DiscoveryStore`, filters by `protocol == Some(Protocol::Matter)`, returns list
- [x] 6.3 Add `list_matter_fabrics` handler — reads `AppState.home`, collects unique `matter_fabric` entries with device counts
- [x] 6.4 Add `pub mod matter;` to `backend/src/http/handlers/mod.rs`

## 7. HTTP — Types & Router

- [x] 7.1 Add `MatterStatusResponse` DTO to `backend/src/http/types.rs`: `devices_seen`, `commissioning_count`, `last_seen_at`
- [x] 7.2 Add `MatterDeviceResponse` DTO: `id`, `name`, `host`, `vendor_id`, `product_id`, `discriminator`, `commissioning_mode`, `thread_role`, `protocol`
- [x] 7.3 Add `FabricResponse` DTO: `fabric_id`, `vendor_id`, `commissioner`, `device_count`
- [x] 7.4 Register `GET /api/matter/status`, `GET /api/matter/devices`, `GET /api/matter/fabrics` in `router.rs` (plus legacy aliases)
- [x] 7.5 Add `matter as matter_handler` import to `router.rs`
- [x] 7.6 Update `device_to_response()` in `helpers.rs` to populate `thread_role` and `matter_fabric`
- [x] 7.7 Add `thread_role` and `matter_fabric` fields to `DeviceResponse` in `types.rs`

## 8. OpenAPI Contract

- [x] 8.1 Add `ThreadRole` schema (enum) to `contracts/openapi.yaml`
- [x] 8.2 Add `MatterFabric` schema to `contracts/openapi.yaml`
- [x] 8.3 Add `MatterStatus` schema to `contracts/openapi.yaml`
- [x] 8.4 Add `MatterDeviceResponse` schema to `contracts/openapi.yaml`
- [x] 8.5 Add `FabricResponse` schema to `contracts/openapi.yaml`
- [x] 8.6 Add `GET /api/matter/status`, `GET /api/matter/devices`, `GET /api/matter/fabrics` paths
- [x] 8.7 Add `thread_role` and `matter_fabric` fields to `Device` schema
- [x] 8.8 Add `mqtt` tag to tags list (already done); add `matter` tag

## 9. Frontend

- [x] 9.1 Add `ThreadRole`, `MatterFabric`, `MatterStatus`, `MatterDeviceResponse`, `FabricResponse` types to `frontend/lib/api/types.ts`
- [x] 9.2 Add `thread_role?: ThreadRole` and `matter_fabric?: MatterFabric` to `Device` interface
- [x] 9.3 Create `frontend/lib/api/matter.ts` with `getMatterStatus()`, `getMatterDevices()`, `getMatterFabrics()` fetch wrappers
- [x] 9.4 Update `frontend/app/ecosystem/page.tsx` — add Matter device count stat and fabric badge row

## 10. Tests

- [x] 10.1 Add unit tests for `ThreadRole::from_txt_bitmask` — border router bit, end device, unknown
- [x] 10.2 Add integration test: `GET /api/matter/status` returns `{"devices_seen":0,"commissioning_count":0,"last_seen_at":null}`
- [x] 10.3 Add integration test: `GET /api/matter/devices` returns `[]` on empty discovery store
- [x] 10.4 Add integration test: `GET /api/matter/fabrics` returns `[]` when no devices have fabric set
