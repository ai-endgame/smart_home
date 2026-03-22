## ADDED Requirements

### Requirement: GET /api/ecosystem returns live topology snapshot
The system SHALL expose `GET /api/ecosystem` that reads current home state and returns a structured topology map including: total device count, protocol distribution, layer breakdown (local vs cloud), and connected/disconnected counts.

#### Scenario: Ecosystem reflects current home state
- **WHEN** the home has 3 devices (2 Shelly, 1 Zigbee) and `GET /api/ecosystem` is called
- **THEN** the response `protocols` array SHALL contain two entries: one for `shelly` with `device_count: 2` and one for `zigbee` with `device_count: 1`

#### Scenario: Ecosystem map includes layer summary
- **WHEN** `GET /api/ecosystem` is called
- **THEN** the response SHALL include `layers.local_devices` (count of devices with local-only protocols) and `layers.cloud_devices` (count with non-local protocols)

#### Scenario: Empty home returns zero counts
- **WHEN** the home has no devices and `GET /api/ecosystem` is called
- **THEN** the response SHALL have `total_devices: 0`, empty `protocols` array, and all layer counts at `0`

#### Scenario: Devices without protocol are counted separately
- **WHEN** the home has devices where `control_protocol` is `null`
- **THEN** the response SHALL include `unprotocolled_devices` count reflecting how many devices have no protocol assigned

### Requirement: EcosystemResponse includes connectivity health
The ecosystem response SHALL include `connected_count` and `disconnected_count` across all devices, enabling a quick health check without calling `/status`.

#### Scenario: Connectivity counts are accurate
- **WHEN** the home has 5 devices, 3 connected and 2 disconnected
- **THEN** `GET /api/ecosystem` SHALL return `connected_count: 3` and `disconnected_count: 2`

### Requirement: Ecosystem protocol entries embed ProtocolInfo
Each protocol entry in the ecosystem response SHALL embed the full `ProtocolInfo` (transport, local_only, mesh, description) alongside the device count.

#### Scenario: Protocol entry is self-describing
- **WHEN** `GET /api/ecosystem` returns a Zigbee protocol entry
- **THEN** that entry SHALL include `transport: "802.15.4"`, `local_only: true`, `mesh: true`, and `device_count: <n>`
