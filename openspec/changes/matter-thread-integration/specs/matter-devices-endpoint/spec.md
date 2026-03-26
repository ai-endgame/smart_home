## ADDED Requirements

### Requirement: GET /api/matter/devices returns Matter-discovered devices
The server SHALL expose `GET /api/matter/devices` returning all devices from `DiscoveryStore` where `protocol == Matter`, including parsed TXT metadata.

#### Scenario: Empty result when no Matter devices
- **WHEN** `GET /api/matter/devices` is called with no Matter devices in DiscoveryStore
- **THEN** response is `200` with an empty array `[]`

#### Scenario: Matter device appears after discovery
- **WHEN** a Matter device is resolved via mDNS
- **THEN** `GET /api/matter/devices` includes that device with `vendor_id`, `product_id`, `discriminator` fields

#### Scenario: Non-Matter devices excluded
- **WHEN** DiscoveryStore contains both a Shelly (Wi-Fi) and a Matter device
- **THEN** `GET /api/matter/devices` returns only the Matter device

### Requirement: GET /api/matter/fabrics returns unique fabric list
The server SHALL expose `GET /api/matter/fabrics` returning the list of unique `MatterFabric` entries across all devices in `AppState.home`, with a `device_count` per fabric.

#### Scenario: No fabrics when no commissioned devices
- **WHEN** no devices have `matter_fabric` set
- **THEN** `GET /api/matter/fabrics` returns `[]`

#### Scenario: Fabrics grouped with device counts
- **WHEN** 3 devices share `fabric_id="abc"` (Apple Home) and 1 device has `fabric_id="xyz"` (Google)
- **THEN** `GET /api/matter/fabrics` returns 2 entries with `device_count: 3` and `device_count: 1`
