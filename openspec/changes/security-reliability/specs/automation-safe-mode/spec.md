## ADDED Requirements

### Requirement: Safe mode flag on AutomationRule
The system SHALL add a `safe_mode: bool` field to `AutomationRule` that defaults to `false`. When `safe_mode` is `true` for a rule, `execute_actions` SHALL skip any action of type `State`, `Brightness`, `Temperature`, or `Lock` (physical actuator actions) and log a warning instead. Non-actuator actions (`Notify`, `ScriptCall`) SHALL still execute.

#### Scenario: Safe mode rule skips actuator actions
- **GIVEN** a rule with `safe_mode: true` and action `type: "state", device_name: "lamp", state: "on"`
- **WHEN** the rule's trigger condition is met and `execute_actions` is called
- **THEN** the lamp state is NOT changed and a warning is logged

#### Scenario: Safe mode rule allows Notify action
- **GIVEN** a rule with `safe_mode: true` and action `type: "notify", message: "Alert"`
- **WHEN** the rule's trigger is satisfied
- **THEN** the notification action executes normally (not suppressed)

#### Scenario: Non-safe mode rule executes all actions
- **GIVEN** a rule with `safe_mode: false` (default) and a state-change action
- **WHEN** the trigger fires
- **THEN** the device state IS changed

### Requirement: Safe mode HTTP toggle endpoint
The system SHALL expose `POST /api/automation/rules/{name}/safe-mode` that toggles the `safe_mode` field of the named rule (same pattern as the existing enable/disable toggle). The response SHALL be `200 { "name": "...", "safe_mode": <new_value> }` or 404 if the rule does not exist.

#### Scenario: Toggle safe mode on an existing rule
- **WHEN** `POST /api/automation/rules/night/safe-mode` is called on a rule with `safe_mode: false`
- **THEN** 200 is returned with `{"safe_mode": true}`

#### Scenario: Toggle safe mode again reverts to false
- **GIVEN** a rule currently has `safe_mode: true`
- **WHEN** `POST /api/automation/rules/night/safe-mode` is called again
- **THEN** 200 is returned with `{"safe_mode": false}`

#### Scenario: Toggle safe mode on unknown rule returns 404
- **WHEN** `POST /api/automation/rules/nonexistent/safe-mode` is called
- **THEN** 404 is returned

### Requirement: safe_mode included in rule list response
The `AutomationRule` response objects returned by `GET /api/automation/rules` and `GET /api/automation/rules/{name}` SHALL include a `safe_mode` boolean field.

#### Scenario: List rules includes safe_mode field
- **WHEN** `GET /api/automation/rules` is called
- **THEN** each rule object includes `"safe_mode": false` (or `true` if toggled)
