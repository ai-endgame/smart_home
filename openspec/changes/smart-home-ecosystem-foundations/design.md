## Context

The project models `control_protocol` as `Option<String>` on `Device`. The only protocols referenced in code comments are "shelly", "wled", "tasmota", "esphome" — but they are never validated, parsed, or enriched. No endpoint describes the home's protocol landscape.

Session 1 of the learning plan requires hands-on understanding of how smart-home protocols stack: physical transport → mesh/routing → application protocol → hub integration → cloud gateway. This design adds that layer to the codebase without breaking existing functionality.

The pattern mirrors what `DeviceType` already does: a typed enum with a `from_str_loose` parser and a `Display` impl for serialization. The DB column stays `TEXT` — no schema migration needed.

## Goals / Non-Goals

**Goals:**
- Add a typed `Protocol` enum with `ProtocolInfo` metadata (transport, local_only, mesh capability)
- Replace `Device.control_protocol: Option<String>` with `Option<Protocol>`
- Expose `GET /api/protocols` — static registry of all supported protocols
- Expose `GET /api/ecosystem` — live topology snapshot of the home
- Keep DB column as `TEXT` (serialize/deserialize at the boundary using the same pattern as `DeviceType`)

**Non-Goals:**
- Actually communicating over Zigbee/Z-Wave/Matter (hardware integration is out of scope)
- Adding protocol-level credentials or pairing state
- Changing `DeviceType` in any way

## Decisions

### 1. `Protocol` as a domain enum, not a newtype string

**Decision:** `Protocol` is a proper enum in `domain/device.rs`, adjacent to `DeviceType`.

**Rationale:** Free-form strings are already causing drift — "shelly" and "Shelly" would be treated as different. A typed enum gives exhaustive match in handlers, prevents invalid states, and matches the `DeviceType` pattern already established. `from_str_loose` + `Display` keeps DB/API compatibility.

**Alternative considered:** Newtype `struct Protocol(String)` — rejected because it provides no exhaustiveness and no compile-time knowledge of valid variants.

### 2. `ProtocolInfo` is a static/pure function, not a stored struct

**Decision:** `Protocol::info(&self) -> ProtocolInfo` returns a value built from a `match` arm — no HashMap, no lazy_static, no DB table.

**Rationale:** Protocol metadata (transport, local_only, mesh) is a compile-time constant of the enum variant. Storing it anywhere else would be a synchronization hazard. A pure function is zero-cost and trivially testable.

### 3. DB column stays `TEXT`

**Decision:** `control_protocol` column remains `TEXT NOT NULL DEFAULT NULL`. The Rust layer parses on read (logging an error and returning `None` for unknown strings) and serializes via `Display` on write.

**Rationale:** Same approach `DeviceType` already uses — proven to work with SQLx's untyped row access. Avoids a Postgres `ALTER COLUMN` migration and keeps the DB portable (SQLite fallback stays viable).

**Risk:** Unknown protocol strings in existing DB rows are silently dropped on load. Mitigation: log a `WARN` with the raw string, same as `parse_device_type` does today.

### 4. `DeviceResponse` gains `control_protocol: Option<String>` (serialized enum)

**Decision:** The JSON response field stays as a nullable string for backward compatibility, but is now driven by `Protocol::to_string()` rather than whatever raw string was stored.

**Rationale:** Clients (frontend, curl users) don't need to change. The type tightening happens at the Rust boundary. If we add a stricter typed field later we can do it additively.

### 5. Ecosystem endpoint aggregates from in-memory home state

**Decision:** `GET /api/ecosystem` reads from `AppState.home` (the `RwLock<SmartHome>`), not the DB.

**Rationale:** `AppState.home` is the authoritative runtime state (DB is just a persistence layer). Protocol distribution reflects what's actually loaded, not what's in storage. Consistent with how `/status` works today.

## Risks / Trade-offs

- [Risk] `from_str_loose` for `Protocol` needs to cover all free-form strings already in DB rows → Mitigation: generous alias matching ("shelly1", "shelly_plus" → `Shelly`; log unknowns as WARN not ERROR)
- [Risk] Adding `control_protocol` to `DeviceResponse` is additive but existing clients may not expect it → Mitigation: field is `Option`, defaults to `null` — no breaking change to existing response consumers
- [Risk] `ProtocolInfo` struct adds a dependency from HTTP types on domain — acceptable since the handler calls `protocol.info()` and serializes it directly

## Migration Plan

1. Add `Protocol` enum + `ProtocolInfo` + `Protocol::info()` to `domain/device.rs`
2. Update `Device.control_protocol` field type; update `db.rs` load/upsert to go through `Protocol::from_str_loose` / `Display`
3. Update `DeviceResponse` in `http/types.rs` to serialize the typed field
4. Add `EcosystemResponse` and `ProtocolInfoResponse` DTOs to `http/types.rs`
5. Add `ecosystem.rs` handler
6. Register `/api/ecosystem` and `/api/protocols` routes in `router.rs`
7. Update `contracts/openapi.yaml`
8. Add frontend `EcosystemMap` page

Rollback: because the DB column stays TEXT, reverting the Rust code restores the previous string behavior without a DB migration.

## Open Questions

- Should unknown protocol strings in the DB be rejected (return error device) or silently become `None`? Current proposal: `None` + WARN log, consistent with `DeviceType` behavior.
- Should `GET /api/protocols` return only protocols *present* in the home, or all supported protocols? Current proposal: all supported (it's a static registry); `GET /api/ecosystem` returns only those present.
