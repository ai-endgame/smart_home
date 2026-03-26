## Why

The current automation engine supports only three trigger types (device state change, temperature above/below) and three action types (set state, brightness, temperature). Real smart home automations require **time-based triggers** (turn off all lights at midnight), **sun-event triggers** (open blinds at sunrise), **webhook triggers** (fire a rule from an external system), and richer actions like **notifications** emitted as SSE events. Without these, users can only react to device state — they cannot automate based on time or receive feedback when a rule fires. This is Session 5 of the mastery plan: *Automations: The Core Value*.

## What Changes

- Add **4 new trigger types**: `time` (fires at a specific HH:MM daily), `sun` (fires at sunrise or sunset with an optional ±offset in minutes), `numeric_state_above` / `numeric_state_below` (device brightness crosses a threshold), `webhook` (fires via `POST /api/automations/webhook/{rule_name}`)
- Add **`notify` action type** — emits an SSE `automation` event with a user-defined message instead of mutating device state
- Add **optional `time_range` condition** on each rule — when set, the trigger action only executes if the current time falls within the window (HH:MM–HH:MM)
- Add **background auto-evaluation loop** — a `tokio::interval` task running every 60 seconds that evaluates time-based and sun-based triggers automatically
- Add **webhook endpoint** `POST /api/automations/webhook/{rule_name}` that fires a specific rule's action on demand
- Add **3 starter template rules** to the frontend automation page for quick-add (Motion Light, Sunrise Blinds, Low Battery Alert)
- Extend the frontend add-rule modal with all new trigger/action types and the time-range condition field

## Capabilities

### New Capabilities
- `automation-triggers`: Extended trigger types covering time, sun, numeric-state, and webhook inputs
- `automation-conditions`: Optional time-range window that gates whether a triggered rule's action executes
- `automation-actions`: New `notify` action type that emits an SSE event with a custom message
- `automation-loop`: Background evaluation loop that polls time/sun triggers every minute automatically
- `automation-templates`: Curated starter templates shown in the UI for one-click rule creation

### Modified Capabilities
- `automation-rules`: AutomationRule extended with `time_range` optional field; `evaluate_rules` accepts current time parameter; `execute_actions` returns notification messages

## Impact

- **Backend domain**: `automation.rs` — new trigger/action variants, `time_range` on `AutomationRule`, updated `evaluate_rules` signature
- **Backend infrastructure**: New `sun.rs` (sunrise/sunset time from env vars), new `automation_loop.rs` (background eval task)
- **Backend HTTP**: New webhook handler in `automation.rs`, new types in `types.rs`, router update
- **OpenAPI**: New trigger type enum values, new action type, new webhook path, time_range field on rule schema
- **Frontend**: Extended `AddRuleModal`, new starter templates section on automation page, updated summary display functions
