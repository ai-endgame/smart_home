## ADDED Requirements

### Requirement: Starter template rules in UI
The frontend automation page SHALL display a "Starter Templates" section containing at least 3 pre-defined automation rule templates. Each template SHALL be renderable as a card showing its name, trigger summary, and action summary. Clicking a template SHALL pre-fill the AddRuleModal form with that template's values, allowing the user to customise before saving. The templates SHALL be: "Motion Light" (state_change on motion sensor → turn light on), "Sunrise Blinds" (sun sunrise trigger → set cover state to on), and "Low Battery Alert" (numeric_state_below brightness 20 → notify).

#### Scenario: Template pre-fills the modal
- **WHEN** the user clicks the "Motion Light" template card
- **THEN** the AddRuleModal opens with trigger type `state_change`, target_state `on`, and action type `state` with state `on` pre-filled

#### Scenario: Templates shown when rule list is empty
- **WHEN** the automation page loads and no rules exist
- **THEN** the Starter Templates section is visible above the empty state message

#### Scenario: Templates shown alongside existing rules
- **WHEN** the automation page loads and rules already exist
- **THEN** the Starter Templates section is still visible (collapsible or always shown)
