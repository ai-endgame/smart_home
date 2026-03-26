## ADDED Requirements

### Requirement: ThreadRole enum on Device
The `Device` struct SHALL have a `thread_role: Option<ThreadRole>` field with variants `BorderRouter`, `Router`, `EndDevice`, `Sleepy`. The `devices` table SHALL store this as nullable `thread_role TEXT`.

#### Scenario: Thread Border Router identified
- **WHEN** a Matter device resolves with TXT `_T=32` (Border Router bit set)
- **THEN** the device entry has `thread_role: "border_router"`

#### Scenario: Non-Thread device has null role
- **WHEN** a Wi-Fi Matter device has no `_T` TXT field
- **THEN** `thread_role` is `null` in the API response

### Requirement: MatterFabric on Device
The `Device` struct SHALL have a `matter_fabric: Option<MatterFabric>` field containing `fabric_id: String`, `vendor_id: u16`, `commissioner: String`. The `devices` table SHALL store this as nullable `matter_fabric TEXT` (JSON-encoded).

#### Scenario: Fabric set when device is commissioned
- **WHEN** a device is updated with fabric information (via future commissioning or manual assignment)
- **THEN** `GET /api/devices/{name}` includes `matter_fabric` with fabric details

#### Scenario: Device without fabric has null
- **WHEN** a device has no `matter_fabric` set
- **THEN** `matter_fabric` is `null` in the API response

### Requirement: thread_role and matter_fabric persisted to DB
Both fields SHALL be persisted via `upsert_device` and loaded via `load_all_devices`.

#### Scenario: Thread role round-trips through DB
- **WHEN** a device with `thread_role: "router"` is persisted and reloaded
- **THEN** `thread_role` is `Router` after reload
