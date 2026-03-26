## Context

The project currently has a monolithic `Device` struct that conflates physical hardware identity with every possible state attribute (brightness, temperature, state, connected). Home Assistant's core insight — the one that makes its architecture scale — is the **device/entity split**:

- A **device** is a physical piece of hardware (e.g. a Shelly plug, a Zigbee thermostat)
- An **entity** is a single observable/controllable attribute that device exposes (e.g. `switch.shelly_plug`, `sensor.thermostat_temperature`, `number.thermostat_target_temp`)

Rooms exist as `HashMap<String, Room>` in `SmartHome` but `Room` only carries a name and a `Vec<String>` of device IDs. HA calls these **Areas** and attaches floor, icon, and a stable slug ID.

## Goals / Non-Goals

**Goals:**
- Add `EntityKind` enum and `Entity` struct to the domain layer
- `Device::entities()` derives the entity list from device type + current state — no storage, pure computation
- `GET /api/devices/{name}/entities` exposes derived entities for a device
- `GET /api/entities` exposes a flat home-wide entity list with optional `?kind=` filter
- Rename `Room` → `Area` internally; add `area_id` (kebab slug), `floor: Option<u8>`, `icon: Option<String>`
- `GET /api/areas` and `GET /api/areas/{area_id}` expose the area registry

**Non-Goals:**
- Storing entities in the database (they are always derived — no entity table)
- Entity state writes via entity ID (use existing device endpoints)
- HA-compatible entity registry file format

## Decisions

### 1. Entities are derived, not stored

**Decision:** `Device::entities()` is a pure function that builds the entity list from the device's current struct fields. No entity table, no entity IDs in the DB.

**Rationale:** Entities in HA are also largely derived from device capabilities — the registry tracks them but the values come from integration state. Since our devices are already the source of truth, deriving entities on read is zero-overhead and always consistent. Storing them would add a sync hazard with no benefit.

**Alternative considered:** Separate `entities` table with FK to devices — rejected because it duplicates state that's already in `Device` fields.

### 2. `entity_id` uses HA's `domain.slug` format

**Decision:** Entity IDs are formatted as `{domain}.{device_slug}_{attribute}`, e.g. `light.desk_lamp`, `number.desk_lamp_brightness`, `sensor.thermo_temperature`.

**Rationale:** Matches HA convention exactly — makes the learning exercise concrete. Students can see how `light.desk_lamp` in their project maps to `light.desk_lamp` in a real HA install.

### 3. `Room` is extended to `Area` in place — struct renamed, field names unchanged

**Decision:** Rename `Room` → `Area` in `domain/device.rs` and `manager.rs`. Add `area_id`, `floor`, `icon` fields. Keep `name` and `device_ids`. All manager methods that accepted "room name" now accept "area name" — same strings, same behaviour.

**Rationale:** A rename + field addition is simpler than a parallel struct. The public API routes change from `/rooms` (which didn't exist as HTTP endpoints) to `/areas` — purely additive since no room HTTP endpoints existed before.

**Alternative considered:** Keep `Room` and add a separate `Area` wrapper — rejected as unnecessary indirection.

### 4. `GET /api/entities` aggregates from in-memory home, filtered by kind

**Decision:** Handler reads `AppState.home`, calls `device.entities()` for each device, flattens, optionally filters by `?kind=`. No DB query.

**Rationale:** Same pattern as `/api/ecosystem` — authoritative state is the in-memory home. Consistent, fast, no additional queries.

### 5. Area metadata stored in DB as extra columns on the `devices` table

**Decision:** Add `area_id TEXT`, `area_floor SMALLINT`, `area_icon TEXT` columns to the `devices` table via idempotent `ADD COLUMN IF NOT EXISTS`. Areas are reconstructed from device rows on load.

**Rationale:** Avoids a separate `areas` table while keeping area metadata persistent. Areas exist implicitly whenever at least one device references them. Consistent with the existing schema approach.

## Risks / Trade-offs

- [Risk] Derived entity IDs change if device name changes → Mitigation: document that entity_id stability requires stable device names; same constraint as HA
- [Risk] `Area` rename touches `manager.rs`, `mod.rs`, and all call sites → Mitigation: compiler guides every change; tests catch regressions
- [Risk] `GET /api/entities` can return a large payload for homes with many devices → Mitigation: `?kind=` filter reduces payload; acceptable for this scale

## Migration Plan

1. Add `EntityKind`, `EntityState`, `Entity` to `domain/device.rs`; add `Device::entities()`
2. Rename `Room` → `Area`; add fields; update all usages in `manager.rs` and `mod.rs`
3. Update `db.rs` to persist/load area metadata columns
4. Add `EntityResponse`, `AreaResponse` DTOs to `http/types.rs`
5. Add `entities.rs` and `areas.rs` handlers; register routes
6. Update OpenAPI contract
7. Update frontend types; add `use-areas` hook; add area metadata to device card

Rollback: entity endpoints are additive — removing them requires only route deregistration. Area rename is a pure refactor with no DB schema change (only additive columns).

## Open Questions

- Should `GET /api/areas/{area_id}` 404 or return empty device list when area has no devices? Proposal: 404 if area doesn't exist in `home.areas`, empty list if it exists with no devices.
- Should area `icon` use HA's MDI icon names (e.g. `mdi:sofa`)? Proposal: free string, no validation — caller decides format.
