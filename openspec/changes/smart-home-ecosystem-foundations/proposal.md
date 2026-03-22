## Why

The project stores `control_protocol` as a free-form string on each device, with no typed model, no protocol metadata, and no way to query the system's protocol landscape. Session 1 of the smart-home learning plan requires understanding how protocols stack — Zigbee, Z-Wave, Matter, Wi-Fi/IP — and how local vs cloud control differs. Implementing this as structured domain logic turns the learning into working code that reflects production smart-home architecture.

## What Changes

- Replace the untyped `control_protocol: Option<String>` field on `Device` with a typed `Protocol` enum covering Zigbee, Z-Wave, Matter, Thread, WiFi (IP-native), and the IP-based device protocols already in use (Shelly, Tasmota, ESPHome, WLED)
- Add a `ProtocolInfo` struct carrying metadata per protocol: transport layer, local-only flag, mesh capability, and typical use cases
- Expose a `GET /api/ecosystem` endpoint that returns an ecosystem map: all protocols present in the home, device count per protocol, connectivity stack layers (cloud / hub / local / device), and a summary of local-only vs cloud-dependent devices
- Add a `GET /api/protocols` endpoint listing all supported protocols with their metadata — acts as a reference/registry
- Update `Device` serialization and the OpenAPI contract to reflect the typed protocol field
- **BREAKING**: `control_protocol` changes from `string | null` to an enum value; existing DB rows with free-form strings are migrated via a `from_str_loose` converter (same pattern as `DeviceType`)

## Capabilities

### New Capabilities

- `protocol-registry`: Typed `Protocol` enum + `ProtocolInfo` metadata struct; `GET /api/protocols` endpoint listing all supported protocols with transport, local_only, mesh, and description fields
- `ecosystem-map`: `GET /api/ecosystem` endpoint returning a live topology snapshot — protocol distribution, device counts per layer (cloud/hub/local/device), and local-vs-cloud ratio across the home

### Modified Capabilities

- None — no existing spec files; this is the first capability layer

## Impact

- `backend/src/domain/device.rs` — add `Protocol` enum, `ProtocolInfo`, update `Device` struct
- `backend/src/http/types.rs` — update request/response DTOs for the typed protocol field
- `backend/src/http/handlers/` — new `ecosystem.rs` handler file
- `backend/src/http/mod.rs` / `router.rs` — register new routes
- `contracts/openapi.yaml` — add `Protocol` schema, new endpoint specs
- `backend/src/infrastructure/db.rs` — protocol column migration (string → enum-compatible)
- `frontend/lib/api/types.ts` — update `Device` type, add `Protocol` and `EcosystemMap` types
- `frontend/app/` — new ecosystem page showing protocol distribution
