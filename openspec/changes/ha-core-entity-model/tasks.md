## 1. Domain — EntityKind, Entity, Device::entities()

- [x] 1.1 Add `EntityKind` enum to `backend/src/domain/device.rs` with variants: `Switch`, `Light`, `Sensor`, `BinarySensor`, `Cover`, `Climate`, `MediaPlayer`, `Lock`, `Camera`, `Number`, `Select`, `Button`
- [x] 1.2 Add `Display` impl for `EntityKind` returning lowercase HA domain string
- [x] 1.3 Add `Entity` struct: `entity_id: String`, `kind: EntityKind`, `device_id: String`, `name: String`, `state: String`, `unit_of_measurement: Option<String>`, `attributes: serde_json::Value`
- [x] 1.4 Add helper `slugify(s: &str) -> String` — lowercase, spaces/special chars → hyphens
- [x] 1.5 Add `Device::entities(&self) -> Vec<Entity>` with derivation rules per `DeviceType`
- [x] 1.6 Add unit tests for `EntityKind::Display`, `Device::entities()` for Light and Thermostat

## 2. Domain — Room → Area rename

- [x] 2.1 Rename `Room` → `Area` in `domain/device.rs`; add fields `area_id: String`, `floor: Option<u8>`, `icon: Option<String>`
- [x] 2.2 Update `Area::new(name)` to auto-derive `area_id` via `slugify`
- [x] 2.3 Update all usages of `Room` in `domain/manager.rs` → `Area`; rename `rooms` field → `areas`; update `pub use` in `domain/mod.rs`
- [x] 2.4 Update `list_rooms()` return type; add `get_area(&self, area_id: &str) -> Option<&Area>`
- [x] 2.5 Add unit tests for `Area::new` slug derivation and `SmartHome` area operations

## 3. Infrastructure — DB area columns

- [x] 3.1 Add `ADD COLUMN IF NOT EXISTS area_floor SMALLINT` and `ADD COLUMN IF NOT EXISTS area_icon TEXT` to `db.rs` migrate()
- [x] 3.2 Update `load_all_devices` to read `area_floor` and `area_icon` columns and reconstruct `Area` metadata in `SmartHome` after loading devices
- [x] 3.3 Update `upsert_device` to persist `area_floor` and `area_icon` alongside existing fields

## 4. HTTP — Types

- [x] 4.1 Add `EntityResponse` DTO to `http/types.rs`: `entity_id`, `kind`, `device_id`, `name`, `state`, `unit_of_measurement`, `attributes`
- [x] 4.2 Add `EntitiesQuery` DTO: `kind: Option<String>`
- [x] 4.3 Add `AreaResponse` DTO: `area_id`, `name`, `floor`, `icon`, `device_count`
- [x] 4.4 Add `AreaDetailResponse` DTO: all `AreaResponse` fields + `devices: Vec<DeviceResponse>`

## 5. HTTP — Handlers

- [x] 5.1 Create `backend/src/http/handlers/entities.rs` with `list_entities` (GET /api/entities) and `list_device_entities` (GET /api/devices/{name}/entities)
- [x] 5.2 `list_entities`: read home, call `device.entities()` for all devices, flatten, filter by `?kind=`
- [x] 5.3 `list_device_entities`: look up device by name, call `.entities()`, return 404 if not found
- [x] 5.4 Create `backend/src/http/handlers/areas.rs` with `list_areas` (GET /api/areas) and `get_area` (GET /api/areas/{area_id})`
- [x] 5.5 `list_areas`: iterate `home.areas`, build `AreaResponse` with device counts
- [x] 5.6 `get_area`: look up area by id, return `AreaDetailResponse` with full device list, 404 if missing
- [x] 5.7 Register all 4 routes in `router.rs` with `/api/` prefix + legacy aliases
- [x] 5.8 Add `pub mod entities; pub mod areas;` to `http/handlers/mod.rs`

## 6. OpenAPI Contract

- [x] 6.1 Add `EntityKind` schema (enum of HA domain strings)
- [x] 6.2 Add `Entity` schema
- [x] 6.3 Add `AreaResponse` and `AreaDetailResponse` schemas
- [x] 6.4 Add `GET /api/devices/{name}/entities` path spec
- [x] 6.5 Add `GET /api/entities` path spec with `kind` query param
- [x] 6.6 Add `GET /api/areas` path spec
- [x] 6.7 Add `GET /api/areas/{area_id}` path spec

## 7. Frontend

- [x] 7.1 Add `EntityKind`, `Entity`, `AreaResponse`, `AreaDetailResponse` types to `frontend/lib/api/types.ts`
- [x] 7.2 Create `frontend/lib/api/areas.ts` with `getAreas()` and `getArea(areaId)` fetch wrappers
- [x] 7.3 Create `frontend/lib/api/entities.ts` with `getDeviceEntities(name)` and `getEntities(kind?)` fetch wrappers
- [x] 7.4 Create `frontend/lib/hooks/use-areas.ts` SWR hook for `/api/areas`
- [x] 7.5 Create `frontend/app/areas/page.tsx` — grid of area cards showing name, floor, icon, device count
- [x] 7.6 Add "Areas" link to `frontend/components/layout/nav.tsx`

## 8. Tests

- [x] 8.1 Add integration test: `GET /api/entities` on empty home returns `[]`
- [x] 8.2 Add integration test: `GET /api/entities` with seeded light — assert 2 entities returned
- [x] 8.3 Add integration test: `GET /api/entities?kind=sensor` filters correctly
- [x] 8.4 Add integration test: `GET /api/devices/{name}/entities` returns 404 for missing device
- [x] 8.5 Add integration test: `GET /api/areas` returns areas after room creation
- [x] 8.6 Add integration test: `GET /api/areas/{area_id}` returns 404 for unknown area
