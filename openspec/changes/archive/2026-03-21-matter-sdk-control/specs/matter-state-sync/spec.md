## ADDED Requirements

### Requirement: Background Matter attribute polling loop
The server SHALL spawn a `tokio` interval task (default interval: 30s, configurable via `MATTER_SYNC_INTERVAL_SECS`) that iterates all commissioned Matter devices and reads the `OnOff` and `LevelControl` attributes via `chip-tool read`.

#### Scenario: Sync loop updates device state
- **WHEN** the sync loop runs and `chip-tool read on-off <node_id> 1` returns `true`
- **THEN** `SmartHome` device state is updated to `On` if it was `Off`, and a `device_updated` SSE event is emitted

#### Scenario: Sync loop updates brightness
- **WHEN** `chip-tool read current-level <node_id> 1` returns `203`
- **THEN** device brightness is updated to `80` (203/254 * 100, rounded)

#### Scenario: Sync loop skips non-Matter devices
- **WHEN** the sync loop runs
- **THEN** only devices with `control_protocol == Some(Protocol::Matter)` are polled

#### Scenario: Sync loop handles offline device gracefully
- **WHEN** `chip-tool read` times out or returns an error for a device
- **THEN** `device.last_error` is updated, the loop continues to the next device without crashing

### Requirement: Sync loop is opt-in via environment variable
The Matter state sync loop SHALL only start when `MATTER_SYNC_ENABLED=true` is set, to avoid hammering offline devices in development environments.

#### Scenario: Sync disabled by default
- **WHEN** `MATTER_SYNC_ENABLED` is not set
- **THEN** no sync loop is started and no `chip-tool read` commands are issued

#### Scenario: Sync enabled explicitly
- **WHEN** `MATTER_SYNC_ENABLED=true` is set
- **THEN** sync loop starts on server startup and logs `"Matter state sync started (interval: 30s)"`

### Requirement: Matter sync status in /api/matter/status
The `MatterStatus` response SHALL include `sync_enabled: bool` and `last_sync_at: Option<String>` fields reflecting the current sync state.

#### Scenario: Status reflects sync state
- **WHEN** `GET /api/matter/status` is called after a sync cycle completes
- **THEN** response includes `"last_sync_at": "<ISO 8601 timestamp>"` and `"sync_enabled": true`
