## Why

The smart home server can discover Matter devices via mDNS but has no ability to control them — state changes are stored in the DB but never sent to real hardware. This session bridges that gap with actual Matter SDK integration and upgrades the frontend from a basic dashboard into a genuinely useful control surface.

## What Changes

- Integrate a Matter control layer via a local `chip-tool` sidecar or `matter-rs` FFI bridge that can commission devices and send cluster commands
- Add a commissioning flow: HTTP endpoint to trigger pairing, track fabric membership
- Add cluster command dispatch: on/off, level control, color temperature — mapped from existing device state mutations
- Add a real-time device state sync loop: poll or subscribe to Matter attribute reports and reflect them in `SmartHome`
- Frontend: replace static device cards with live-control modals (toggle, slider, color temp)
- Frontend: add a commissioning wizard UI (enter pairing code → commission → device appears)
- Frontend: upgrade the Discovery page to show commissioning status and a one-click "Commission" button for Matter devices
- Frontend: add per-device entity panel showing real attributes (linkquality, temperature, occupancy) pulled from `Device.attributes`

## Capabilities

### New Capabilities

- `matter-commissioning`: HTTP endpoint + frontend wizard to commission a Matter device into the server's fabric using a setup code
- `matter-cluster-control`: Dispatch OnOff / LevelControl / ColorControl cluster commands to real Matter devices on state change
- `matter-state-sync`: Background loop that reads Matter attribute reports and updates `SmartHome` + pushes SSE events to connected clients
- `device-control-ui`: Frontend device control modal with live toggle, brightness slider, and temperature control; replaces static device cards
- `commissioning-wizard-ui`: Frontend step-by-step commissioning flow: enter pairing code → confirm → device added to home

### Modified Capabilities

- `mqtt-bridge`: State mutations that previously only published to MQTT now also dispatch Matter cluster commands when the device's `control_protocol` is `matter`

## Impact

- **Backend**: new `infrastructure/matter_control.rs` module; `chip-tool` binary invoked as subprocess (or `matter-rs` crate if stable enough); new HTTP handlers `POST /api/matter/commission`, `POST /api/matter/devices/{id}/command`
- **Frontend**: `DeviceCard` component upgraded; new `CommissionModal` component; Discovery page gains "Commission" CTA for Matter devices
- **Dependencies**: `chip-tool` (CHIP SDK) via Docker sidecar or system install; optionally `matter-rs` crate
- **API contract**: 2 new endpoints, `Device` schema gains `commissioned: bool` field
- **DB**: no new columns needed — `matter_fabric` already stores fabric membership
