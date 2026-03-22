## Why

The backend currently has no authentication on write endpoints (only `/server/stop` is gated by `ADMIN_TOKEN`), no request-size protection, no way to back up or restore system state, and no safe-mode guard against automations accidentally actuating physical devices. As the system grows — and especially before any remote access scenario — these gaps make the server trivially writable by anyone on the network.

## What Changes

- Add optional `API_KEY` authentication middleware gating all write (non-GET) requests; health/status/events always bypass
- Add `RequestBodyLimitLayer` (64 KB default) and tighten name-length validation in handlers
- Add `GET /api/backup` endpoint that exports the full system state as a portable JSON snapshot
- Add `safe_mode: bool` field to `AutomationRule`; when true, actuator actions (State, Brightness, Temperature) are suppressed with a warning log — only Notify and ScriptCall proceed

## Capabilities

### New Capabilities
- `api-auth`: Opt-in API key authentication middleware for all write endpoints; controlled by `API_KEY` env var
- `request-hardening`: Body size limit layer + name/title input length validation (max 120 chars) in handlers
- `state-backup`: `GET /api/backup` exports all registries (devices, automations, scripts, scenes, persons, dashboards) as a single JSON document
- `automation-safe-mode`: `safe_mode` field on `AutomationRule`; safe rules skip actuator actions; HTTP endpoint to set/clear safe_mode per rule

### Modified Capabilities
*(none — no existing spec-level behavior changes)*

## Impact

- **Backend**: `config.rs` (API_KEY field), `http/middleware.rs` (auth middleware), `http/router.rs` (body limit layer + auth layer), `domain/automation.rs` (`safe_mode` field), `http/handlers/system.rs` (backup), `http/handlers/automation.rs` (safe mode toggle)
- **OpenAPI**: new `backup` response schema, new `safe_mode` field on Rule, new auth header description, new `/api/backup` path
- **Frontend**: backup download button on dashboard or system info; safe_mode toggle in automation UI
- **No breaking changes** — API key is opt-in (if `API_KEY` env var absent, all requests pass); safe_mode defaults to `false`
