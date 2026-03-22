## MODIFIED Requirements

### Requirement: State mutations dispatch protocol-appropriate commands
After any device state mutation (toggle, brightness, temperature), the server SHALL dispatch the appropriate protocol command based on `device.control_protocol`:
- `zigbee` / `mqtt` → publish to `zigbee2mqtt/{name}/set` (existing behavior)
- `matter` → dispatch Matter cluster command via `chip-tool` (new behavior)
- `None` → no external command dispatched (existing behavior)

#### Scenario: Zigbee device state change still uses MQTT
- **WHEN** a device with `control_protocol: zigbee` has its state toggled
- **THEN** `publish_command()` is called with the MQTT topic (unchanged from Session 3 behavior)

#### Scenario: Matter device state change uses chip-tool
- **WHEN** a device with `control_protocol: matter` has its state toggled
- **THEN** `matter_control::dispatch_onoff()` is called instead of `publish_command()`; no MQTT publish occurs

#### Scenario: Device with no protocol has no side effects
- **WHEN** a device with `control_protocol: None` has its state toggled
- **THEN** neither MQTT publish nor chip-tool command is issued
