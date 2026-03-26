## ADDED Requirements

### Requirement: Time-range condition on rules
The system SHALL support an optional `time_range` field on any automation rule. When present, `time_range` SHALL contain `from` and `to` values in "HH:MM" (24-hour) format. After a trigger fires, the system SHALL check whether the current time falls within the window before executing the action. If the current time is outside the window, the action SHALL be skipped. Rules without a `time_range` SHALL execute unconditionally (existing behavior preserved).

#### Scenario: Action executes within time range
- **WHEN** a rule with `time_range: { from: "08:00", to: "22:00" }` triggers and current time is 14:30
- **THEN** the condition passes and the action executes

#### Scenario: Action skipped outside time range
- **WHEN** the same rule triggers and current time is 23:00
- **THEN** the condition fails and the action does not execute

#### Scenario: Midnight-spanning range works correctly
- **WHEN** a rule has `time_range: { from: "22:00", to: "06:00" }` and current time is 23:30
- **THEN** the condition passes (from > to means overnight window)

#### Scenario: No time_range — action always executes
- **WHEN** a rule has no `time_range` field and its trigger fires
- **THEN** the action executes regardless of current time
