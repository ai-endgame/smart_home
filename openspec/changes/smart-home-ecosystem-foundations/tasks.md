## 1. Domain — Protocol Enum & ProtocolInfo

- [x] 1.1 Add `Protocol` enum to `backend/src/domain/device.rs` with variants: `Zigbee`, `ZWave`, `Matter`, `Thread`, `WiFi`, `Shelly`, `Tasmota`, `ESPHome`, `WLED`, `Unknown`
- [x] 1.2 Add `Display` impl for `Protocol` returning canonical snake_case strings
- [x] 1.3 Add `Protocol::from_str_loose(s: &str) -> Option<Protocol>` with generous alias matching
- [x] 1.4 Add `ProtocolInfo` struct: `transport: &'static str`, `local_only: bool`, `mesh: bool`, `description: &'static str`
- [x] 1.5 Add `Protocol::info(&self) -> ProtocolInfo` method with a match arm per variant
- [x] 1.6 Change `Device.control_protocol` from `Option<String>` to `Option<Protocol>`
- [x] 1.7 Update `Device::new()` — no default protocol (stays `None`)
- [x] 1.8 Add unit tests for `Protocol::from_str_loose`, `Display`, and `info()` in `device.rs`

## 2. Infrastructure — DB Serialization

- [x] 2.1 Update `db.rs` `load_all_devices`: parse `control_protocol` column via `Protocol::from_str_loose`, log WARN on unknown strings, set field to `None`
- [x] 2.2 Update `db.rs` `upsert_device`: serialize `device.control_protocol` via `protocol.to_string()` (or bind `None`)

## 3. HTTP — Types & Handlers

- [x] 3.1 Update `DeviceResponse` in `http/types.rs` to add `control_protocol: Option<String>` field (serialized from typed enum)
- [x] 3.2 Update `helpers.rs` `device_to_response` to populate `control_protocol` field
- [x] 3.3 Add `ProtocolInfoResponse` DTO to `http/types.rs`: `id`, `transport`, `local_only`, `mesh`, `description`
- [x] 3.4 Add `ProtocolEntry` DTO (for ecosystem): extends `ProtocolInfoResponse` with `device_count: usize`
- [x] 3.5 Add `EcosystemResponse` DTO to `http/types.rs`: `total_devices`, `connected_count`, `disconnected_count`, `unprotocolled_devices`, `layers` (local/cloud counts), `protocols: Vec<ProtocolEntry>`
- [x] 3.6 Create `backend/src/http/handlers/ecosystem.rs` with two handlers: `get_protocols` and `get_ecosystem`
- [x] 3.7 `get_protocols`: return static list of all `Protocol` variants with their `ProtocolInfo`
- [x] 3.8 `get_ecosystem`: read from `AppState.home`, aggregate counts, build `EcosystemResponse`
- [x] 3.9 Register `GET /api/protocols` and `GET /api/ecosystem` (plus legacy aliases) in `router.rs`
- [x] 3.10 Add `pub mod ecosystem;` to `http/handlers/mod.rs`

## 4. OpenAPI Contract

- [x] 4.1 Add `Protocol` schema to `contracts/openapi.yaml` (enum of snake_case strings)
- [x] 4.2 Add `ProtocolInfo` schema
- [x] 4.3 Add `ProtocolEntry` schema
- [x] 4.4 Add `EcosystemResponse` schema
- [x] 4.5 Add `GET /api/protocols` path spec
- [x] 4.6 Add `GET /api/ecosystem` path spec
- [x] 4.7 Update `DeviceResponse` schema to include `control_protocol` field (nullable string)

## 5. Frontend

- [x] 5.1 Add `Protocol` type and `ProtocolInfo`, `ProtocolEntry`, `EcosystemResponse` interfaces to `frontend/lib/api/types.ts`
- [x] 5.2 Add `control_protocol` field to `Device` interface in `types.ts`
- [x] 5.3 Create `frontend/lib/api/ecosystem.ts` with `getEcosystem()` and `getProtocols()` fetch wrappers
- [x] 5.4 Create `frontend/lib/hooks/use-ecosystem.ts` SWR hook for `/api/ecosystem`
- [x] 5.5 Create `frontend/app/ecosystem/page.tsx` — grid of protocol cards showing transport, mesh badge, device count; plus layer summary bar (local vs cloud devices)
- [x] 5.6 Add "Ecosystem" link to `frontend/components/layout/nav.tsx`

## 6. Tests

- [x] 6.1 Add integration test in `http/mod.rs` for `GET /api/protocols` — assert all variants present, fields populated
- [x] 6.2 Add integration test for `GET /api/ecosystem` with a seeded home — assert protocol distribution counts
- [x] 6.3 Add integration test for `GET /api/ecosystem` with empty home — assert zero counts and empty protocols array
