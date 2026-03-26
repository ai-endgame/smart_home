## ADDED Requirements

### Requirement: Matter mDNS scanner runs on boot
The server SHALL start a passive mDNS scanner that browses `_matter._tcp.local.` and `_matterc._tcp.local.` on startup without requiring any configuration.

#### Scenario: Scanner starts automatically
- **WHEN** the server starts
- **THEN** mDNS browsing begins for `_matter._tcp.local.` and `_matterc._tcp.local.`

### Requirement: Matter TXT record fields are parsed
When a Matter service is resolved, the scanner SHALL parse the following TXT record fields: `D` (discriminator), `VP` (vendor/product IDs), `CM` (commissioning mode), `RI` (rotating device ID), `_T` (Thread role bitmask).

#### Scenario: Full TXT record parsed
- **WHEN** a Matter device resolves with TXT `D=3840 VP=65521+32768 CM=0 _T=2`
- **THEN** discriminator=3840, vendor_id=65521, product_id=32768, commissioning_mode=0, thread_role hint is captured

#### Scenario: Missing TXT fields tolerated
- **WHEN** a Matter device resolves with only partial TXT records
- **THEN** missing fields default to `None` and the device is still pushed to DiscoveryStore

### Requirement: Matter devices pushed to DiscoveryStore
Resolved Matter devices SHALL be inserted into `DiscoveryStore` with `protocol: Protocol::Matter` and any parsed metadata stored in `properties`.

#### Scenario: New Matter device added to discovery
- **WHEN** `_matter._tcp` service resolves for device `matter-bulb-1`
- **THEN** `GET /api/discovery/devices` includes an entry with `protocol: "matter"`

#### Scenario: Already-discovered device not duplicated
- **WHEN** the same Matter device resolves twice (e.g. address change)
- **THEN** the existing DiscoveryStore entry is updated, not duplicated
