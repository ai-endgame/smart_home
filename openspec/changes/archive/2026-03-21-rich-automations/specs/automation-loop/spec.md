## ADDED Requirements

### Requirement: Background auto-evaluation loop
The system SHALL start a background tokio task on server startup that evaluates all enabled automation rules every 60 seconds. On each tick the system SHALL obtain the current local time, call `evaluate_rules` with that time, execute any triggered actions, and emit SSE events for notify actions. Errors in individual rule execution SHALL be logged and skipped; they SHALL NOT stop the loop.

#### Scenario: Loop fires time trigger at matching minute
- **WHEN** a rule with `trigger: { type: "time", time: "07:00" }` is registered and the loop ticks at 07:00
- **THEN** the rule's action executes and an SSE `automation` event is emitted

#### Scenario: Loop does not fire webhook rules
- **WHEN** a rule with a `webhook` trigger is registered and the loop ticks
- **THEN** the rule's action does NOT execute

#### Scenario: Loop continues after a rule error
- **WHEN** a rule's action fails (e.g., device not found)
- **THEN** the loop logs the failure and continues to the next tick without crashing

### Requirement: Sun time configuration
The system SHALL read `SUNRISE_TIME` and `SUNSET_TIME` environment variables at startup. Values SHALL be "HH:MM" strings. If a variable is missing or invalid, the system SHALL use defaults: `SUNRISE_TIME=06:00`, `SUNSET_TIME=18:00`. The resolved times SHALL be used by all sun trigger evaluations.

#### Scenario: Custom sunrise time used
- **WHEN** `SUNRISE_TIME=05:45` is set and a sun trigger with `event: "sunrise"` is evaluated at 05:45
- **THEN** the trigger fires

#### Scenario: Default used when env var absent
- **WHEN** `SUNRISE_TIME` is not set and a sun trigger with `event: "sunrise"` is evaluated at 06:00
- **THEN** the trigger fires using the default 06:00
