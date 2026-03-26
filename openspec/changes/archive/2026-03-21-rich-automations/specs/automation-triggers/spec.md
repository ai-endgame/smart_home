## ADDED Requirements

### Requirement: Time trigger
The system SHALL support a `time` trigger type with a `time` field in "HH:MM" (24-hour) format. The trigger SHALL evaluate to true when the current time's hour and minute match the configured value. Seconds SHALL be ignored. An invalid time string SHALL cause the trigger to always evaluate to false.

#### Scenario: Time trigger matches current minute
- **WHEN** a rule with `trigger: { type: "time", time: "22:00" }` is evaluated at 22:00:45
- **THEN** the trigger evaluates to true and the action executes

#### Scenario: Time trigger does not match
- **WHEN** the same rule is evaluated at 21:59:59
- **THEN** the trigger evaluates to false and the action is skipped

### Requirement: Sun trigger
The system SHALL support a `sun` trigger type with a `event` field (`"sunrise"` or `"sunset"`) and an optional `offset_minutes` field (integer, positive = after, negative = before). The system SHALL resolve sunrise and sunset times from `SUNRISE_TIME` and `SUNSET_TIME` environment variables (default `"06:00"` and `"18:00"`). The trigger SHALL evaluate to true when current time matches base_time + offset_minutes.

#### Scenario: Sunrise trigger fires at configured time
- **WHEN** `SUNRISE_TIME=06:30` and a sun trigger with `event: "sunrise", offset_minutes: 0` is evaluated at 06:30
- **THEN** the trigger evaluates to true

#### Scenario: Sunset with offset
- **WHEN** `SUNSET_TIME=18:00` and a sun trigger with `event: "sunset", offset_minutes: -30` is evaluated at 17:30
- **THEN** the trigger evaluates to true

### Requirement: Numeric state triggers
The system SHALL support `numeric_state_above` and `numeric_state_below` trigger types with fields `device_name` (string) and `threshold` (float). The `attribute` field SHALL specify which numeric value to compare: `"brightness"` or `"temperature"`. These trigger types SHALL evaluate against the device's current attribute value.

#### Scenario: Brightness above threshold triggers
- **WHEN** a rule with `trigger: { type: "numeric_state_above", device_name: "lamp", attribute: "brightness", threshold: 80 }` is evaluated and lamp's brightness is 90
- **THEN** the trigger evaluates to true

#### Scenario: Brightness below threshold fires
- **WHEN** a rule with `trigger: { type: "numeric_state_below", device_name: "lamp", attribute: "brightness", threshold: 20 }` is evaluated and lamp's brightness is 10
- **THEN** the trigger evaluates to true

#### Scenario: Device not found — trigger false
- **WHEN** a numeric_state trigger references a device that does not exist in SmartHome
- **THEN** the trigger evaluates to false

### Requirement: Webhook trigger
The system SHALL support a `webhook` trigger type with an `id` field (string). A webhook trigger SHALL only fire when an explicit `POST /api/automations/webhook/{rule_name}` request is received. The webhook trigger SHALL always evaluate to false inside the automatic evaluation loop.

#### Scenario: Webhook endpoint fires rule action
- **WHEN** `POST /api/automations/webhook/my_rule` is called and the rule has a webhook trigger
- **THEN** the rule's action executes and `200` is returned with the action result

#### Scenario: Webhook with wrong trigger type returns 400
- **WHEN** `POST /api/automations/webhook/my_rule` is called and the rule exists but has a non-webhook trigger
- **THEN** the system returns `400 Bad Request` with a message explaining the trigger type mismatch

#### Scenario: Webhook for unknown rule returns 404
- **WHEN** `POST /api/automations/webhook/nonexistent` is called
- **THEN** the system returns `404 Not Found`
