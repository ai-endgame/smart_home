## ADDED Requirements

### Requirement: Device control modal
The frontend SHALL show a `DeviceControlModal` slide-over when a device card is clicked, replacing the current static card interaction. The modal SHALL display the current device state and provide live controls.

#### Scenario: Open control modal
- **WHEN** user clicks a device card
- **THEN** a modal slides in showing device name, type, current state, brightness (if applicable), and temperature (if applicable)

#### Scenario: Toggle device state from modal
- **WHEN** user clicks the toggle button in the modal
- **THEN** an optimistic update flips the displayed state immediately and `PATCH /api/devices/{name}/state` is called; on error the state reverts and an error message is shown

#### Scenario: Adjust brightness from modal
- **WHEN** user drags the brightness slider to a new value and releases
- **THEN** `PATCH /api/devices/{name}/brightness` is called with the new value; the slider reflects the confirmed value after the response

#### Scenario: Adjust color temperature from modal
- **WHEN** the device type is `light` and the user drags the color temperature slider
- **THEN** `PATCH /api/devices/{name}/temperature` is called with the new mired value

#### Scenario: Modal shows connection status
- **WHEN** `device.connected` is `false`
- **THEN** modal shows a "Disconnected" warning badge and controls are disabled

### Requirement: Device card shows live state indicator
The `DeviceCard` component SHALL show a colored dot reflecting `device.state` (green = on, grey = off) and `device.connected` (amber ring = disconnected).

#### Scenario: Connected and on
- **WHEN** device state is `on` and connected is `true`
- **THEN** card shows a green indicator dot

#### Scenario: Disconnected
- **WHEN** device connected is `false`
- **THEN** card shows an amber ring around the indicator dot

### Requirement: Entity panel in control modal
The control modal SHALL include an "Attributes" section showing all non-empty key-value pairs from `device.attributes` (populated from MQTT or Matter reads).

#### Scenario: Show linkquality attribute
- **WHEN** `device.attributes` contains `{"linkquality": 85}`
- **THEN** modal shows "linkquality: 85" in the attributes panel

#### Scenario: Hide attributes panel when empty
- **WHEN** `device.attributes` is empty or `{}`
- **THEN** the attributes section is not rendered
