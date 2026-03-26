## ADDED Requirements

### Requirement: PresenceChange trigger variant
The system SHALL add a `Trigger::PresenceChange { person_name: String, target_state: PresenceState }` variant to the `Trigger` enum in `domain/automation.rs`.

#### Scenario: Trigger fires when person arrives
- **WHEN** a rule has trigger `PresenceChange { person_name: "alice", target_state: Home }`
- **AND** the person's `effective_state` equals `Home`
- **THEN** `evaluate_rules` SHALL include the rule's action in the result

#### Scenario: Trigger does not fire for wrong state
- **WHEN** a rule has trigger `PresenceChange { person_name: "alice", target_state: Away }`
- **AND** the person's `effective_state` is `Home`
- **THEN** `evaluate_rules` SHALL NOT include the rule's action

#### Scenario: Trigger does not fire for unknown person
- **WHEN** a rule references a person name that does not exist in the registry
- **THEN** `evaluate_rules` SHALL NOT include the rule's action (returns false)

### Requirement: evaluate_rules accepts PresenceRegistry
The system SHALL update `evaluate_rules(home, presence, now)` to accept an additional `presence: &PresenceRegistry` parameter. All callers (HTTP handler, CLI, automation loop) SHALL be updated accordingly.

#### Scenario: CLI invocation with empty presence registry
- **WHEN** the CLI calls `evaluate_rules` with a fresh empty `PresenceRegistry`
- **THEN** all non-presence triggers SHALL continue to evaluate as before

### Requirement: PresenceChange in HTTP API and OpenAPI contract
The system SHALL accept `{ "type": "presence_change", "person_name": "…", "target_state": "home" | "away" }` in `CreateRuleRequest.trigger` and serialize it back in `RuleResponse.trigger`.

#### Scenario: Rule created with presence_change trigger
- **WHEN** `POST /automation/rules` is called with a `presence_change` trigger
- **THEN** the rule SHALL be stored and returned with `trigger.type = "presence_change"`
