## ADDED Requirements

### Requirement: Notify action
The system SHALL support a `notify` action type with a `message` field (non-empty string, max 512 characters). When a notify action executes, the system SHALL emit an SSE event of kind `automation` with the message as the event message. The notify action SHALL NOT modify any device state.

#### Scenario: Notify action emits SSE event
- **WHEN** an automation rule with `action: { type: "notify", message: "Low battery on sensor!" }` fires
- **THEN** an SSE event is broadcast to all connected clients with the notification message

#### Scenario: Empty message rejected
- **WHEN** a rule is created with `action: { type: "notify", message: "" }`
- **THEN** the system returns `400 Bad Request`

#### Scenario: Notify appears in run_automation response
- **WHEN** `POST /api/automation/run` executes a notify action
- **THEN** the response includes the notify action in the `actions` array with type `"notify"` and the message
