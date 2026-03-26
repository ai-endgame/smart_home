## Context

The codebase already discovers `_matter._tcp` devices via mDNS (in `mdns.rs`) and models `Protocol::Matter`, but treats Matter devices identically to any other mDNS device — no TXT record parsing, no fabric tracking, no Thread topology. Session 4 makes Matter first-class: a dedicated scanner extracts the rich metadata that Matter's mDNS advertisements carry (vendor/product IDs, commissioning mode, discriminator) and surfaces it through new endpoints.

No new Rust dependencies are needed — `mdns-sd` already handles multicast DNS browsing and is already in the dependency tree.

## Goals / Non-Goals

**Goals:**
- Parse Matter mDNS TXT fields and store structured metadata per device
- Model Thread mesh roles (`BorderRouter`, `Router`, `EndDevice`, `Sleepy`) on `Device`
- Model `MatterFabric` (which ecosystem commissioned the device) on `Device`
- Expose `GET /api/matter/status`, `GET /api/matter/devices`, `GET /api/matter/fabrics`
- Show Matter device count and fabric badges on the Ecosystem page

**Non-Goals:**
- Full Matter commissioning flow (requires CHIP SDK — out of scope)
- Sending commands to Matter devices (no Matter controller implemented)
- Thread Border Router setup or Thread network management
- Matter over BLE (only mDNS/IP discovery)

## Decisions

### Dedicated `matter.rs` module, not extending `mdns.rs`

**Decision**: Create `infrastructure/matter.rs` as a separate module that runs its own browse loop alongside the existing `mdns.rs` loop.

**Why**: `mdns.rs` is a generic scanner that browses many service types. Matter discovery has specific logic (TXT field parsing, commissioning mode detection, fabric tracking) that would clutter the generic scanner. Separation keeps each module focused and testable independently.

**Alternative**: Add Matter-specific branches inside `mdns.rs`. Rejected — `mdns.rs` already handles 14 service types and adding Matter TXT parsing would make it significantly harder to read.

### MatterFabric stored on Device, not in a separate table

**Decision**: `matter_fabric: Option<MatterFabric>` is a field on `Device`, persisted as a JSON column (`matter_fabric TEXT`).

**Why**: Fabrics are per-device metadata, not a separate registry in this system. Storing as JSON TEXT avoids a new table while keeping the data queryable. `GET /api/matter/fabrics` derives the unique fabric list by scanning all devices at query time.

**Alternative**: Separate `fabrics` table with foreign key. Over-engineered for a learning project — adds migration complexity with no query performance benefit at this scale.

### ThreadRole on Device, not a separate mesh model

**Decision**: `thread_role: Option<ThreadRole>` field directly on `Device`, analogous to `zigbee_role`.

**Why**: Consistent with the `ZigbeeRole` pattern from Session 3. Thread topology is a property of the device, not a separate entity. A Border Router is just a device with `thread_role = BorderRouter`.

### MatterStatus as Arc<RwLock<>> in AppState

**Decision**: Same pattern as `MqttStatus` — `Arc<RwLock<MatterStatus>>` written by the scanner, read by the status handler.

**Why**: Zero new patterns to learn — identical to the MQTT status approach. Consistent AppState shape.

## Risks / Trade-offs

- **Matter mDNS records vary by implementation** — Apple Home, Google, HA all advertise slightly differently. TXT field parsing uses `get()` with `None` fallbacks, never panics on missing fields. → Mitigation: lenient parsing; log unknown fields as debug.
- **No real Matter devices in dev** → Use `dns-sd` or `avahi-browse` to manually register a fake Matter service for testing. Integration tests use the existing fake-AppState pattern (no mDNS required).
- **Thread topology not directly observable** — Thread role comes from Matter TXT records (`_T` field), which not all devices advertise. → Mitigation: `thread_role` is always `Option`; no UI hard-dependency on it.
- **`matter_fabric` JSON column** — if `MatterFabric` struct changes, old JSON rows won't deserialize. → Mitigation: use `serde(default)` on all fields; add `ADD COLUMN IF NOT EXISTS` migration.

## Migration Plan

1. Add `matter_fabric TEXT` and `thread_role TEXT` columns via idempotent `ADD COLUMN IF NOT EXISTS`
2. Existing devices get `NULL` for both — no data loss
3. Matter scanner starts automatically on boot (no config required — always on)
4. Rollback: remove `matter_status` from AppState and the scanner call — all other code unaffected

## Open Questions

- Should `GET /api/matter/fabrics` include device counts per fabric? (Proposed: yes — useful for the UI)
- Should `MatterFabric.commissioner` be a typed enum or free string? (Proposed: free string — commissioner names aren't standardised)
