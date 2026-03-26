## Why

Session 4 of the mastery plan teaches Matter and Thread: the IP-native open standard backed by Apple, Google, Amazon, and Samsung. The codebase already models `Protocol::Matter` and discovers `_matter._tcp` devices via mDNS, but does nothing with them — there is no commissioning flow, no Matter device state, and no Thread topology awareness. Adding Matter support turns the theory into observable running code that works with Apple Home, HomePod, and any Matter controller.

## What Changes

- Add `MatterFabric` struct to domain: `fabric_id`, `vendor_id`, `node_id`, `commissioner` (which ecosystem commissioned it — Apple, Google, HA, etc.)
- Add `thread_role: Option<ThreadRole>` to `Device` — `BorderRouter`, `Router`, `EndDevice`, `Sleepy` — models Thread mesh topology
- Add `matter_fabric: Option<MatterFabric>` to `Device` — which fabric the device joined
- Add `backend/src/infrastructure/matter.rs` — passive Matter scanner:
  - Subscribes to mDNS `_matter._tcp` and `_matterc._tcp` (commissioning) events
  - Parses TXT record fields: `D` (discriminator), `VP` (vendor/product), `CM` (commissioning mode), `RI` (rotating device ID)
  - Pushes newly-seen Matter devices into `DiscoveryStore` with `protocol: Protocol::Matter`
- Add `GET /api/matter/devices` — list all Matter-discovered devices with fabric and Thread role info
- Add `GET /api/matter/fabrics` — list unique fabrics seen across all devices
- Add `matter_status` to `AppState` as `Arc<RwLock<MatterStatus>>` — scan counts, last seen timestamp
- Add `GET /api/matter/status` — scanner health: devices seen, last scan timestamp
- Update `Makefile` with `matter-scan` target that runs a one-shot mDNS scan for Matter devices
- Frontend: add `ThreadRole`, `MatterFabric`, `MatterStatus` types; add Matter device list to Ecosystem page

## Capabilities

### New Capabilities

- `matter-scanner`: Passive mDNS scanner for `_matter._tcp` / `_matterc._tcp`, parses TXT records, pushes to DiscoveryStore with `Protocol::Matter`
- `matter-status-endpoint`: `GET /api/matter/status` — scanner health and device counts
- `matter-devices-endpoint`: `GET /api/matter/devices` — filtered view of discovered devices with Matter protocol; `GET /api/matter/fabrics` — unique fabric list
- `thread-topology`: `ThreadRole` enum on Device, fabric association via `MatterFabric`

### Modified Capabilities

- None — Matter is purely additive; no existing routes change

## Impact

- `backend/Cargo.toml` — no new deps (uses existing `mdns-sd`)
- `backend/src/domain/device.rs` — add `ThreadRole` enum, `MatterFabric` struct, fields on `Device`
- `backend/src/infrastructure/matter.rs` — new module (Matter mDNS scanner)
- `backend/src/infrastructure/mod.rs` — add `pub mod matter`
- `backend/src/state.rs` — add `matter_status: Arc<RwLock<MatterStatus>>`
- `backend/src/http/mod.rs` — start Matter scanner on boot
- `backend/src/http/handlers/matter.rs` — new handlers for status, devices, fabrics endpoints
- `backend/src/http/router.rs` — register new routes
- `backend/src/http/types.rs` — add `MatterStatusResponse`, `MatterDeviceResponse`, `FabricResponse`
- `contracts/openapi.yaml` — add `ThreadRole`, `MatterFabric`, `MatterStatus` schemas and new paths
- `frontend/lib/api/types.ts` — add `ThreadRole`, `MatterFabric`, `MatterStatus` types; add fields to `Device`
- `frontend/lib/api/matter.ts` — new fetch wrappers
- `frontend/app/ecosystem/page.tsx` — add Matter device count and fabric badges
