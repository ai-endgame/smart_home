## ADDED Requirements

### Requirement: Commissioning wizard modal
The frontend SHALL provide a `CommissionModal` component accessible from the Discovery page for any detected Matter device. The modal guides the user through a 3-step commissioning flow.

#### Scenario: Open commissioning wizard from discovery
- **WHEN** user clicks "Commission" on a Matter device in the Discovery page
- **THEN** `CommissionModal` opens pre-filled with the device name; step 1 (enter setup code) is shown

#### Scenario: Step 1 — enter pairing code
- **WHEN** user types an 11-digit setup code and clicks "Start Commissioning"
- **THEN** `POST /api/matter/commission` is called; the modal advances to step 2 (progress)

#### Scenario: Step 2 — commissioning in progress
- **WHEN** commissioning job is in `in_progress` state
- **THEN** modal shows a spinner, progress message from `job.message`, and a 60-second countdown timer

#### Scenario: Step 3 — success
- **WHEN** job status becomes `"done"`
- **THEN** modal advances to step 3 showing "Device added!" with the new device name; a "Go to Devices" button navigates to `/devices`

#### Scenario: Commissioning failed
- **WHEN** job status becomes `"failed"`
- **THEN** modal shows the error message with a "Try Again" button that resets to step 1

#### Scenario: Invalid pairing code format
- **WHEN** user enters fewer than 11 digits
- **THEN** the "Start Commissioning" button is disabled and an inline validation message is shown

### Requirement: Discovery page Commission button
The Discovery page SHALL show a "Commission" CTA button on cards for devices with `protocol == "matter"` (from `DiscoveredDevice.protocol`).

#### Scenario: Matter device shows Commission button
- **WHEN** a discovered device has `protocol: "matter"`
- **THEN** a "Commission" button appears on the device card

#### Scenario: Non-Matter device has no Commission button
- **WHEN** a discovered device has `protocol: null` or a non-Matter protocol
- **THEN** no "Commission" button is shown; only "Add to Home" is available
