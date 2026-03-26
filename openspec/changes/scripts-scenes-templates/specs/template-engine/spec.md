## ADDED Requirements

### Requirement: Template expression syntax
The system SHALL recognize template expressions as strings containing one or more `{{ expr }}` tokens. The expression inside the braces SHALL support: device state lookups (`state("device_name")`), brightness lookups (`brightness("device_name")`), current hour (`now_hour()`), integer/float literals, arithmetic operators (`+`, `-`, `*`, `/`), and comparison operators (`==`, `!=`, `>`, `<`, `>=`, `<=`). Expressions SHALL be evaluated at the time the containing automation condition or script step is executed.

#### Scenario: Simple state lookup resolves
- **WHEN** the expression `{{ state("living_room_light") }}` is evaluated and the device state is `"on"`
- **THEN** the result is the string `"on"`

#### Scenario: Arithmetic expression resolves
- **WHEN** the expression `{{ brightness("desk_lamp") + 20 }}` is evaluated and the device brightness is `60`
- **THEN** the result is the number `80`

#### Scenario: Time-based expression resolves
- **WHEN** the expression `{{ now_hour() >= 22 }}` is evaluated at 23:00
- **THEN** the result is `true`

### Requirement: Template evaluation in automation conditions
The system SHALL evaluate template expressions in `TemplateEval` conditions during automation rule checking. A `TemplateEval` condition SHALL pass when the expression evaluates to a truthy value (`true`, non-zero number, non-empty string).

#### Scenario: Condition passes on truthy template result
- **WHEN** an automation with condition `{ type: "template_eval", expr: "{{ brightness(\"lamp\") > 50 }}" }` fires and lamp brightness is 80
- **THEN** the condition passes and the automation action executes

#### Scenario: Condition fails on falsy template result
- **WHEN** the same condition fires and lamp brightness is 30
- **THEN** the condition fails and the automation action is skipped

### Requirement: Template evaluation in script steps
Template expressions in `set_state`, `set_brightness`, and `set_temperature` step fields SHALL be resolved before the step is executed. The resolved value SHALL be used as the effective field value.

#### Scenario: Brightness step uses template value
- **WHEN** a script step `{ type: "set_brightness", device_name: "lamp", brightness: "{{ brightness(\"ref_lamp\") - 10 }}" }` executes and `ref_lamp` brightness is 70
- **THEN** `lamp` brightness is set to `60`

### Requirement: Template error handling
If a template expression references an unknown device or contains a syntax error, the evaluation SHALL return an error. In automation conditions this causes the condition to fail (skips the action). In script steps this causes the step to be skipped and sets `last_error` on the target device.

#### Scenario: Unknown device in template
- **WHEN** a template expression references a device name that does not exist in SmartHome
- **THEN** evaluation returns an error and the containing rule/step is treated as failed
