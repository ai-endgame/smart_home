# Smart Home Server Implementation Plan

## Scope and Assumptions
- Build a new `smart_home_server` binary in the existing Rust crate.
- Keep existing CLI functionality intact.
- Implement an HTTP+JSON server with in-memory state for devices, automations, events, and client/device connection lifecycle.
- Focus on the spec's required capabilities: start/stop, requests, errors, automations, device states/commands/events/connections/disconnections/errors/updates/queries, and multi-client handling.

## Current Codebase Baseline
- Existing modules provide reusable domain logic:
  - `src/models.rs`: device and room models.
  - `src/manager.rs`: device/room state operations.
  - `src/automation.rs`: automation rules and execution.
  - `src/logger.rs`: env logger setup.
- Current binary entrypoint is CLI-only (`src/main.rs`).
- No HTTP server stack or multi-binary setup exists yet.

## Implementation Steps
1. Convert crate to shared library + binaries
- Add `src/lib.rs` exporting domain modules and new server module.
- Keep CLI in `src/main.rs` but make it use library exports.
- Add second binary `src/bin/smart_home_server.rs`.

2. Add server dependencies
- Add `tokio`, `axum`, `serde`, `serde_json`, and `thiserror`.

3. Extend domain model for connection/error state
- Add connection and last-error fields to `Device`.
- Add manager methods for connect/disconnect and setting/clearing device errors.

4. Implement server core module
- Create `src/server.rs` with:
  - Shared app state (`SmartHome`, `AutomationEngine`, event log, client sessions).
  - Router setup and graceful shutdown.
  - Health/status endpoints.
  - Unified error handling and JSON error responses.

5. Implement device APIs
- Add endpoints for:
  - Device CRUD and query.
  - State/brightness/temperature updates.
  - Generic command endpoint.
  - Connect/disconnect.
  - Error report/clear.

6. Implement automation APIs
- Add endpoints for:
  - Create/list/remove/toggle rules.
  - Evaluate+execute rules.

7. Implement events and client lifecycle APIs
- Add endpoints for:
  - Connect/disconnect/list clients.
  - Query all events and per-device events.
- Emit events for key actions and failures.

8. Add tests
- Add async handler/integration-style tests for:
  - Health endpoint.
  - Device create/query flow.
  - Automation execution flow.

9. Validation and docs
- Run formatting and tests.
- Update `Makefile` with server run target.
- Document server usage in output notes.

## Deliverables
- New plan file (this document).
- Working `smart_home_server` binary.
- New HTTP API covering spec capabilities.
- Passing tests for existing and server features.
