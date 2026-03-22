## ADDED Requirements

### Requirement: Automation conditions array
An automation rule SHALL support an optional `conditions` array. Each condition SHALL be one of: `StateEquals { device_name, state }`, `BrightnessAbove { device_name, value }`, `BrightnessBelow { device_name, value }`, `TemplateEval { expr }`. All conditions in the array SHALL be evaluated after the trigger fires; if ANY condition fails, the action SHALL NOT execute (logical AND semantics).

#### Scenario: All conditions pass — action executes
- **WHEN** an automation fires and all conditions in the array evaluate to true
- **THEN** the action is executed normally

#### Scenario: One condition fails — action skipped
- **WHEN** an automation fires and at least one condition evaluates to false
- **THEN** the action is not executed; no error is raised

#### Scenario: Empty conditions array — action always executes
- **WHEN** an automation fires and the `conditions` array is empty or absent
- **THEN** the action executes unconditionally (backwards-compatible behavior)

### Requirement: script_call action type
An automation rule action SHALL support a new type `script_call` with fields `script_name: String` and `args: HashMap<String, Value>`. When this action type fires, the system SHALL look up the named script in the registry and execute it with the provided args, using the same async executor as `POST /api/scripts/{id}/run`.

#### Scenario: script_call triggers script execution
- **WHEN** an automation fires with action `{ type: "script_call", script_name: "dim_all", args: { "brightness": 30 } }`
- **THEN** the `dim_all` script executes with `brightness = 30` in its argument context

#### Scenario: script_call with missing script
- **WHEN** an automation fires with a `script_call` action referencing a script that does not exist
- **THEN** the action is skipped and the automation engine logs a warning; no error is propagated to the HTTP layer
