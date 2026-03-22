## 1. Domain — New Trigger Variants

- [x] 1.1 Add `SunEvent` enum (`Sunrise`, `Sunset`) and `NumericAttr` enum (`Brightness`, `Temperature`) in `automation.rs`; derive `Debug`, `Clone`
- [x] 1.2 Add `Time { time: String }` variant to `Trigger` enum
- [x] 1.3 Add `Sun { event: SunEvent, offset_minutes: i32 }` variant to `Trigger` enum
- [x] 1.4 Add `NumericStateAbove { device_name: String, attribute: NumericAttr, threshold: f64 }` and `NumericStateBelow { .. }` variants to `Trigger` enum
- [x] 1.5 Add `Webhook { id: String }` variant to `Trigger` enum
- [x] 1.6 Update `Trigger::fmt` Display to cover all 5 new variants
- [x] 1.7 Add `time_range: Option<(String, String)>` field to `AutomationRule` struct (from, to as "HH:MM")
- [x] 1.8 Update `AutomationEngine::add_rule` to accept `time_range: Option<(String, String)>` as an extra parameter and store it on the rule
- [x] 1.9 Update `evaluate_rules` signature to `(&self, home: &SmartHome, now: chrono::NaiveTime) -> Vec<Action>` and add match arms for new trigger types; `Webhook` always returns `false`
- [x] 1.10 Add `time_in_range(now: NaiveTime, from: &str, to: &str) -> bool` helper in `automation.rs` supporting overnight ranges (from > to)
- [x] 1.11 Apply `time_range` check inside `evaluate_rules`: after trigger fires, check `time_in_range` if `time_range` is `Some`

## 2. Domain — Notify Action

- [x] 2.1 Add `Notify { message: String }` variant to `Action` enum with `Display`
- [x] 2.2 Change `execute_actions` return type to `Vec<String>` (notification messages); `Notify` appends to result, device actions return nothing
- [x] 2.3 Update `execute_actions` callers in tests to handle new return type (ignore or assert)
- [x] 2.4 Update `run_automation` handler in `handlers/automation.rs` to call `evaluate_rules` with `chrono::Local::now().time()` and collect notification messages; emit one SSE `automation` event per message

## 3. Infrastructure — Sun Times

- [x] 3.1 Create `backend/src/infrastructure/sun.rs` with `sunrise_time() -> chrono::NaiveTime` and `sunset_time() -> chrono::NaiveTime` reading `SUNRISE_TIME` / `SUNSET_TIME` env vars; default to `"06:00"` / `"18:00"` on parse failure
- [x] 3.2 Add `pub mod sun;` to `backend/src/infrastructure/mod.rs`

## 4. Infrastructure — Auto-Eval Loop

- [x] 4.1 Create `backend/src/infrastructure/automation_loop.rs` with `start_automation_loop(state: AppState)` that spawns a `tokio::task` with a `tokio::time::interval(Duration::from_secs(60))`
- [x] 4.2 Loop body: lock home + automation read locks, call `evaluate_rules(home, now)`, unlock, call `execute_actions`, emit SSE for each notification message via `record_event`
- [x] 4.3 Add `pub mod automation_loop;` to `backend/src/infrastructure/mod.rs`
- [x] 4.4 Call `start_automation_loop(state.clone())` in `run_server_full()` in `http/mod.rs`

## 5. HTTP — Webhook Endpoint + Types

- [x] 5.1 Add `WebhookFireResponse { rule_name: String, action_executed: bool, message: String }` to `types.rs`
- [x] 5.2 Add `time_range: Option<TimeRange>` and `TimeRange { from: String, to: String }` to `types.rs` and to `RuleResponse` / `AddRuleRequest`
- [x] 5.3 Update `TriggerInput` in `types.rs` to add fields for new trigger types: `time`, `event` (SunEvent), `offset_minutes`, `attribute` (NumericAttr), `id` (webhook); update `to_domain()` conversion
- [x] 5.4 Update `ActionInput` in `types.rs` to add `message: Option<String>` field; update `to_domain()` and `action_to_response()` helper in `helpers.rs`
- [x] 5.5 Update `rule_to_response()` helper in `helpers.rs` to map `time_range` field from domain rule
- [x] 5.6 Add `webhook_trigger` handler in `handlers/automation.rs`: `POST /api/automations/webhook/{rule_name}` — looks up rule, validates it has `Webhook` trigger, executes action, returns `WebhookFireResponse`
- [x] 5.7 Register `POST /api/automations/webhook/:rule_name` (and legacy alias) in `router.rs`

## 6. OpenAPI Contract

- [x] 6.1 Add `time`, `sun`, `numeric_state_above`, `numeric_state_below`, `webhook` to the `TriggerType` enum in `contracts/openapi.yaml`
- [x] 6.2 Add `SunEvent` enum schema with values `sunrise`, `sunset`
- [x] 6.3 Add `NumericAttr` enum schema with values `brightness`, `temperature`
- [x] 6.4 Add `TimeRange` schema with `from` and `to` string fields
- [x] 6.5 Update `Trigger` schema to include all new optional fields: `time`, `event`, `offset_minutes`, `attribute`, `threshold`, `id`
- [x] 6.6 Add `notify` to the `ActionType` enum; add `message` field to `Action` schema
- [x] 6.7 Add `time_range` (nullable `TimeRange`) to `AutomationRule` schema
- [x] 6.8 Add `POST /api/automations/webhook/{rule_name}` path with `WebhookFireResponse` schema

## 7. Frontend — Extended AddRuleModal

- [x] 7.1 Add `time`, `sun`, `numeric_state_above`, `numeric_state_below`, `webhook` to `TRIGGER_TYPES` in `add-rule-modal.tsx`
- [x] 7.2 Update `handleTriggerType` reset logic to initialise correct default fields for each new trigger type
- [x] 7.3 Render time input (HH:MM) for `time` trigger type
- [x] 7.4 Render sun event dropdown (`sunrise`/`sunset`) + offset_minutes number input for `sun` trigger type
- [x] 7.5 Render attribute dropdown (`brightness`/`temperature`) + threshold number input for `numeric_state_above` / `numeric_state_below`
- [x] 7.6 Render webhook id text input for `webhook` trigger type
- [x] 7.7 Add `notify` to `ACTION_TYPES`; render message textarea for `notify` action type
- [x] 7.8 Add optional time-range section (collapsible toggle) with `from`/`to` HH:MM inputs at the bottom of the form
- [x] 7.9 Update `Trigger` and `Action` TypeScript types in `frontend/lib/api/types.ts` to include all new variants/fields

## 8. Frontend — Trigger/Action Summary Display

- [x] 8.1 Update `triggerSummary` in `automation/page.tsx` to handle `time`, `sun`, `numeric_state_above`, `numeric_state_below`, `webhook`
- [x] 8.2 Update `actionSummary` to handle `notify`
- [x] 8.3 Show a small time-range badge `🕐 08:00–22:00` on the rule card when `time_range` is set

## 9. Frontend — Starter Templates

- [x] 9.1 Add `STARTER_TEMPLATES` array in `automation/page.tsx` with 3 entries: Motion Light, Sunrise Blinds, Low Battery Alert
- [x] 9.2 Add `templateToRequest(t: TemplateRule): CreateRuleRequest` helper that maps template to a form-ready object
- [x] 9.3 Render a "Starter Templates" section on the automation page (always visible, collapsible with a toggle) showing template cards
- [x] 9.4 Each template card shows name, trigger chip, action chip, and a "Use Template" button
- [x] 9.5 Clicking "Use Template" calls `setOpen(true)` and pre-fills the modal form via `setForm(templateToRequest(t))`

## 10. Tests

- [x] 10.1 Unit test: `Time` trigger matches when `now` matches the configured HH:MM; does not fire at any other minute
- [x] 10.2 Unit test: `NumericStateAbove` fires when brightness exceeds threshold; does not fire below
- [x] 10.3 Unit test: `time_in_range` correctly handles normal ranges, overnight ranges, and boundary values
- [x] 10.4 Unit test: `execute_actions` with `Notify` action returns the message in the Vec; device state unchanged
- [x] 10.5 Integration test: `POST /api/automations/webhook/{rule_name}` fires rule with webhook trigger and returns 200
- [x] 10.6 Integration test: `POST /api/automations/webhook/{rule_name}` returns 404 for unknown rule
- [x] 10.7 Integration test: `POST /api/automations/webhook/{rule_name}` returns 400 when rule has non-webhook trigger
