## Why

Session 2 of the mastery plan centres on the HA data model's most important concept: **devices and entities are different things**. A device is a physical piece of hardware; an entity is a single, observable/controllable attribute it exposes (e.g. a thermostat device has a `temperature` entity, a `target_temperature` entity, and an `hvac_mode` entity). The current codebase conflates the two — `Device` is a monolithic struct that mixes hardware identity with every possible state attribute. Implementing the HA entity model turns that into working code that teaches the concept hands-on.

## What Changes

- Add `EntityKind` enum covering all entity types HA defines: `Switch`, `Light`, `Sensor`, `BinarySensor`, `Cover`, `Climate`, `MediaPlayer`, `Lock`, `Camera`, `Number`, `Select`, `Button`
- Add `Entity` struct: `entity_id` (slug like `light.desk_lamp_brightness`), `kind: EntityKind`, `device_id`, `name`, `state: EntityState`, `attributes: HashMap<String, serde_json::Value>`, `unit_of_measurement: Option<String>`
- Add `Device::entities()` — derives the canonical entity list for a device based on its `DeviceType` and current state (e.g. a `Light` yields a `Switch` entity + a `Number` entity for brightness)
- Expose `GET /api/devices/{name}/entities` — lists all entities for a device
- Expose `GET /api/entities` — flat list of all entities across the home, filterable by `?kind=`
- Upgrade `Room` to `Area`: add `area_id` (kebab slug), `floor: Option<u8>`, `icon: Option<String>`; keep backward-compatible name-based routing
- Add `GET /api/areas` and `GET /api/areas/{area_id}` — area registry with device list
- **BREAKING** (additive only): existing `/api/devices` responses gain no new required fields; entity and area endpoints are purely additive

## Capabilities

### New Capabilities

- `entity-model`: `EntityKind` enum, `Entity` struct, `Device::entities()` derivation, `GET /api/devices/{name}/entities`, `GET /api/entities`
- `area-registry`: Upgraded `Area` struct (extends `Room`), `GET /api/areas`, `GET /api/areas/{area_id}` with device membership

### Modified Capabilities

- None — entity and area endpoints are purely additive; no existing route changes

## Impact

- `backend/src/domain/device.rs` — add `EntityKind`, `EntityState`, `Entity`; add `Device::entities()` method
- `backend/src/domain/manager.rs` — upgrade `Room` → `Area` (add fields); update room management methods
- `backend/src/http/types.rs` — add `EntityResponse`, `AreaResponse` DTOs
- `backend/src/http/handlers/` — new `entities.rs` handler; new `areas.rs` handler
- `backend/src/http/router.rs` — register new routes
- `backend/src/infrastructure/db.rs` — add area metadata columns (`area_id`, `floor`, `icon`) to devices table via idempotent `ADD COLUMN IF NOT EXISTS`
- `contracts/openapi.yaml` — add `Entity`, `EntityKind`, `Area` schemas and new paths
- `frontend/lib/api/types.ts` — add `Entity`, `EntityKind`, `Area` types
- `frontend/app/ecosystem/page.tsx` — no change (entities visible via device detail)
- `frontend/app/devices/` — device card links to entity list
