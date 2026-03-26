## ADDED Requirements

### Requirement: notify_url field on AutomationRule
`AutomationRule` in `backend/src/domain/automation.rs` SHALL gain an optional field `notify_url: Option<String>` with `#[serde(default)]`. The field SHALL be initialized to `None` in `add_rule`. `AddRuleRequest` in `http/types.rs` SHALL also gain `notify_url: Option<String>` so callers can set it at rule creation time. `RuleResponse` SHALL include `notify_url: Option<String>`.

#### Scenario: Rule without notify_url has None
- **WHEN** a rule is created without the `notify_url` field
- **THEN** `notify_url` is `None` in the domain object and absent (or null) in the API response

#### Scenario: Rule created with notify_url stores it
- **WHEN** a rule is created with `"notify_url": "https://example.com/hook"`
- **THEN** `GET /api/automation/rules` returns that rule with `notify_url` set to the provided URL

### Requirement: Outbound HTTP POST on Notify action
When `execute_actions` processes an `Action::Notify` and the corresponding rule has a non-empty `notify_url`, the system SHALL asynchronously POST a JSON body `{"rule": "<name>", "message": "<message>", "timestamp": "<iso8601>"}` to the URL. The POST SHALL use a 5-second timeout. Errors SHALL be logged at `WARN` level and not propagated to callers.

#### Scenario: Notify action without URL is silent
- **WHEN** a `Notify` action fires and the rule has no `notify_url`
- **THEN** no HTTP request is made; the notification message is returned normally via the existing notifications vec

#### Scenario: Notify action with URL triggers outbound POST
- **WHEN** a `Notify` action fires and the rule has `notify_url` set
- **THEN** a POST request is dispatched asynchronously to the URL with the notification payload

#### Scenario: Outbound POST failure is non-fatal
- **WHEN** the `notify_url` host is unreachable or returns an error
- **THEN** the automation execution continues normally and a WARN log is emitted

### Requirement: Webhook dispatch in automation loop and HTTP handlers
The outbound webhook dispatch SHALL occur in all execution paths: the HTTP `run_automation` handler, the `fire_rule` function in `automation_loop.rs`, and the time-based automation loop. Each path that calls `execute_actions` SHALL also have access to rule metadata (specifically `notify_url` and `name`) to dispatch the outbound webhook.

#### Scenario: Webhook fires from automation loop
- **WHEN** the time-based automation loop triggers a `Notify` action on a rule with `notify_url`
- **THEN** the outbound POST is dispatched asynchronously from within the loop
