## Why

The current automation engine supports simple single-trigger → single-action rules, but real smart home workflows require **reusable logic** (scripts called from multiple rules), **state snapshots** (scenes that restore a set of device states at once), and **dynamic expressions** (template conditions and action values computed at runtime). Without these, users must duplicate rules and hard-code values everywhere.

## What Changes

- Add **Scripts** — named, reusable action sequences with optional input parameters; can be called from automations or triggered directly via the API
- Add **Scenes** — device state snapshots that can be captured from current state or defined manually, and applied in one API call
- Add **Template expressions** — a minimal Jinja2-inspired expression engine (`{{ expr }}`) evaluated in automation conditions and action field values (state, brightness, temperature)
- Extend automation rules with a `conditions` array (optional pre-checks before executing the action) and `script_call` action type
- Expose new CRUD endpoints for scripts and scenes in the HTTP layer and OpenAPI contract
- Add frontend pages for Scripts and Scenes management with apply/run controls

## Capabilities

### New Capabilities
- `script-registry`: Named scripts with parameter definitions, step lists (state/brightness/temperature/delay actions), and execution via API
- `scene-registry`: Named scenes that snapshot or define device states and can be applied atomically
- `template-engine`: Inline expression evaluation (`{{ device.state }}`, arithmetic, comparisons) used in automation conditions and action field values

### Modified Capabilities
- `automation-rules`: Extend existing rules with optional `conditions` array and new `script_call` action type that invokes a registered script with arguments

## Impact

- **Backend**: New domain structs (`Script`, `Scene`, `TemplateExpr`), new infrastructure module `template.rs`, new HTTP handlers (`scripts.rs`, `scenes.rs`), domain extensions in `automation.rs` and `manager.rs`
- **Database**: Two new tables `scripts` and `scenes` (JSON columns for steps/states); `automation_rules` table gains `conditions` JSONB column
- **OpenAPI**: New tags `scripts` and `scenes`, new schemas, new CRUD paths
- **Frontend**: New `/scripts` and `/scenes` pages; automation editor gains conditions UI; DeviceControlModal gains "Save as Scene" button
