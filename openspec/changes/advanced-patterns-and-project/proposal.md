## Why

The smart home backend is feature-complete for core use cases but lacks the real-time responsiveness, operational tooling, and extensibility that production IoT systems demand. Session 10 elevates the platform from a working prototype to a production-ready system by adding SSE-based live event streaming, device state history, backup restore, and outbound webhook dispatch — patterns that round out every major capability introduced across Sessions 1–9.

## What Changes

- **Real-time event stream** — `GET /api/events/stream` returns a Server-Sent Events (SSE) connection; any `ServerEvent` written to the ring buffer is immediately flushed to all connected SSE clients via a Tokio broadcast channel
- **Device state history** — an in-memory ring buffer (max 500 entries per device) records every state transition with a timestamp; exposed via `GET /api/devices/{name}/history`
- **Backup restore** — `POST /api/restore` accepts a `BackupDocument` JSON body, clears all in-memory registries, re-imports devices/rules/scripts/scenes/persons/dashboards, and persists to the database if configured
- **Outbound webhook dispatch** — each automation rule may carry an optional `notify_url: Option<String>`; when a `Notify` action fires, the server POSTs the message as JSON to that URL asynchronously (fire-and-forget with a 5-second timeout)
- **Frontend live updates** — the frontend connects to `/api/events/stream` and uses the stream to invalidate SWR caches without polling, replacing the 10-second polling intervals on the devices and automation hooks

## Capabilities

### New Capabilities

- `sse-event-stream`: Server-Sent Events endpoint that broadcasts `ServerEvent` to all connected clients in real time via a shared broadcast channel in `AppState`
- `device-state-history`: Per-device in-memory ring buffer of state-change history; new `HistoryEntry` domain type and `GET /api/devices/{name}/history` endpoint
- `backup-restore`: `POST /api/restore` handler that validates and re-imports a `BackupDocument`, clearing existing state first; idempotent with conflict detection options
- `outbound-webhook`: Optional `notify_url` field on `AutomationRule`; async fire-and-forget HTTP POST when a `Notify` action executes

### Modified Capabilities

- `automation-engine`: `AutomationRule` gains `notify_url: Option<String>` field; `execute_actions` dispatches outbound webhooks for `Notify` actions when a URL is present

## Impact

- **Backend**: New `AppState.history` registry; new broadcast channel `AppState.events_tx: broadcast::Sender<ServerEvent>`; `execute_actions` becomes async for outbound HTTP; `automation.rs`, `state.rs`, `http/handlers/system.rs`, `http/handlers/devices.rs`, `http/router.rs`, `http/types.rs` all touched
- **Frontend**: `lib/hooks/use-devices.ts` and `lib/hooks/use-automation.ts` gain an SSE subscriber that calls `mutate()` on relevant events; new `lib/api/events.ts` for the SSE client helper
- **Dependencies**: No new Rust crates needed (tokio broadcast + reqwest already present); `axum::response::sse` is part of axum 0.8 core
