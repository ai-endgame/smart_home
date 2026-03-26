## Context

Sessions 3 and 4 added Zigbee/MQTT integration and a passive Matter mDNS scanner. The server can now discover Matter devices and track them in `DiscoveryStore`, but every state mutation (toggle, brightness, temperature) is fire-and-forget into the in-memory `SmartHome` and DB — nothing reaches real hardware.

The Matter protocol is controlled via the CHIP SDK (`chip-tool` CLI or `matter-rs` Rust bindings). Commissioning requires a one-time pairing using a setup code (QR or manual entry) to create a fabric relationship. Post-commissioning, cluster commands are sent over the local network using the commissioned fabric credentials stored on disk by the CHIP SDK.

The frontend currently shows static device cards with no real-time feedback and no commissioning UI.

## Goals / Non-Goals

**Goals:**
- Commission Matter devices from the server using a setup pairing code
- Dispatch OnOff, LevelControl, and ColorTemperatureMired cluster commands when device state changes
- Sync Matter attribute reports back into `SmartHome` and push SSE events
- Frontend device control modal with live toggle, brightness slider, color temp
- Frontend commissioning wizard (pairing code → commission → device appears in home)
- Discovery page "Commission" button for detected Matter devices

**Non-Goals:**
- Full Matter fabric administration (multi-admin, ACL management)
- Thread network setup (Border Router configuration out of scope)
- Matter bridge devices (e.g. Hue Bridge Matter mode) — treated as individual devices
- iOS/Android HomeKit parity — the goal is local HTTP control, not HomeKit replacement
- `matter-rs` native integration — too unstable; `chip-tool` sidecar is the pragmatic choice

## Decisions

### D1: chip-tool sidecar over matter-rs

**Choice:** Run `chip-tool` as a subprocess, invoked per-command via `tokio::process::Command`.

**Rationale:** `matter-rs` is incomplete and has no stable API for commissioning or cluster control as of early 2026. `chip-tool` is the official CHIP SDK reference CLI, widely tested, and supports all required cluster commands. The subprocess overhead (~100ms per call) is acceptable for home automation use cases.

**Alternative considered:** `matter-rs` FFI — rejected due to instability and missing commissioning support.

### D2: chip-tool via Docker sidecar

**Choice:** Run `chip-tool` in a Docker container (`connectedhomeip/chip-tool`) mounted with a shared volume for fabric credentials (`/tmp/chip_tool_config`).

**Rationale:** Avoids native CHIP SDK build complexity on macOS/Linux. The server writes commands via `docker exec` or a thin HTTP wrapper. Credentials persist in the mounted volume across restarts.

**Alternative considered:** System-installed `chip-tool` — rejected because it requires a full CHIP SDK build (30+ min) and is not reproducible across environments.

### D3: Command dispatch on state mutation

**Choice:** After every device state mutation in HTTP handlers (toggle, brightness, temperature), check `device.control_protocol == Some(Protocol::Matter)` and dispatch the corresponding cluster command asynchronously (fire-and-forget with error logging).

**Rationale:** Keeps handlers fast — Matter command is spawned as a `tokio::task`, handler returns immediately. If the device is offline the error is logged and reflected in `device.last_error`.

**Alternative considered:** Synchronous dispatch — rejected because `chip-tool` can take 1–5 seconds.

### D4: State sync via polling

**Choice:** Background `tokio::interval` loop (every 30s) reads `OnOff` and `LevelControl` attributes from commissioned Matter devices via `chip-tool read` and updates `SmartHome`.

**Rationale:** Matter attribute subscriptions require a persistent CHIP SDK session, which adds significant complexity. Polling at 30s is sufficient for home automation and keeps the architecture simple.

**Alternative considered:** Matter attribute subscriptions — deferred to a future session.

### D5: Frontend control modal

**Choice:** Replace `DeviceCard` click behavior with a slide-over modal (`DeviceControlModal`) that shows real-time state and provides toggle/slider controls. Uses SWR `mutate` for optimistic updates.

**Rationale:** The current static card is not actionable. A modal keeps the device grid clean while providing a rich control surface.

### D6: Commissioning wizard as a multi-step modal

**Choice:** A 3-step modal: (1) enter setup code / discriminator, (2) commission in progress (polling `GET /api/matter/commission/{job_id}`), (3) success — device added.

**Rationale:** Commissioning takes 10–30 seconds; a job-based polling approach avoids HTTP timeout issues and gives the user visible progress.

## Risks / Trade-offs

- **chip-tool Docker latency**: First command after container cold-start can take 3–5s. → Mitigation: keep the container running, use `docker exec` rather than `docker run`.
- **Fabric credential loss**: If `/tmp/chip_tool_config` volume is deleted, all commissioned devices must be re-commissioned. → Mitigation: document the volume path, optionally back up to DB as JSON blob.
- **Matter device offline**: `chip-tool` hangs for up to 30s on unreachable devices. → Mitigation: set `--timeout 10` flag on all `chip-tool` invocations; reflect failure in `device.last_error`.
- **Commissioning window**: Matter devices are only commissionable for ~15 minutes after factory reset. → Mitigation: commissioning wizard shows a countdown timer.
- **chip-tool one-shot model**: `chip-tool` does not maintain a persistent session; each command incurs TCP/UDP setup overhead. → Acceptable for MVP; persistent sessions are a future optimization.

## Migration Plan

1. Add `chip-tool` Docker service to `docker-compose.yml` with shared credential volume
2. Add `make commission-sidecar` Makefile target
3. Deploy backend changes — new endpoints are additive, no breaking changes
4. Frontend changes are component-level, no routing changes

## Open Questions

- Should fabric credentials be backed up to PostgreSQL? (Currently stored only in Docker volume)
- Should `matter-state-sync` be opt-in via `MATTER_SYNC_ENABLED=true` env var to avoid hammering offline devices?
- Is `chip-tool` available as a pre-built Docker image that works on Apple Silicon (arm64)?
