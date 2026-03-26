## Context

The platform now has nine completed sessions worth of features: devices, automation, MQTT/Matter, entities/areas, presence, scripts/scenes, dashboards, and security hardening. All data-fetching in the frontend uses SWR with fixed polling intervals (5–10 s). Events are written to a `VecDeque` ring buffer in `AppState` and only exposed via a polling `GET /api/events` endpoint. There is no real-time push path, no state history for devices, and no restore counterpart to the backup endpoint.

## Goals / Non-Goals

**Goals:**
- Add SSE push so the frontend receives events within milliseconds without polling
- Store per-device state-change history (last 500 entries) and expose it via REST
- Complete the backup/restore loop with `POST /api/restore`
- Allow automation rules to fire outbound webhook POSTs on `Notify` actions

**Non-Goals:**
- WebSocket bidirectional channels (SSE is read-only push; sufficient for this use case)
- Persistent event storage (events stay in-memory; history ring buffer is not written to DB)
- Full CQRS or event sourcing (existing architecture is retained)
- Authentication for outbound webhooks (out-of-band; caller manages secrets in the URL)

## Decisions

### D1 — SSE via `tokio::sync::broadcast` channel

**Decision**: Add `events_tx: broadcast::Sender<ServerEvent>` to `AppState`. Every call to `record_event` also sends on `events_tx`. The SSE handler holds a `broadcast::Receiver` and yields events as SSE data frames.

**Alternative considered**: A `Vec<mpsc::Sender>` subscriber list requires write-locking on every subscriber add/remove and cleanup of dead senders. `broadcast` gives lock-free fan-out with automatic backpressure and `RecvError::Lagged` for slow consumers.

**Constraint**: `ServerEvent` must derive `Clone` for broadcast. It already derives `Clone` (required by `VecDeque::push_back` callers).

### D2 — Device history as `Arc<RwLock<HashMap<String, VecDeque<HistoryEntry>>>>`

**Decision**: A `history` map in `AppState` keyed by lowercase device name, capped at 500 entries per device with `pop_front` on overflow. Appended directly inside `set_device_state`, `set_device_brightness`, `set_device_temperature`.

**Alternative considered**: Recording history in `SmartHome::set_state`. That would keep domain logic pure but couples domain to `chrono::Utc::now()`. Instead, history is recorded at the HTTP handler layer (same pattern as `record_event`) which keeps domain free of timestamps.

**Why not DB**: History is diagnostic; losing it on restart is acceptable. DB writes on every state change would add latency and require a new schema migration.

### D3 — `POST /api/restore` clears then re-imports

**Decision**: Acquire write locks on all registries in order (home → automation → scripts → scenes → presence → dashboard), clear each, then replay the backup fields. DB records are deleted and re-inserted if a pool is present. Return `200 {restored: {...counts}}`.

**Alternative considered**: Merge strategy (only add missing items). Merge creates ambiguity when names conflict — a clean replace is simpler and matches user expectation of "restore from backup means start fresh".

**Risk**: A partial failure mid-import leaves state inconsistent. Mitigation: hold all write locks for the full duration so no request can observe the partial state.

### D4 — `execute_actions` stays synchronous; webhook dispatched via `tokio::spawn`

**Decision**: `execute_actions` signature does not become `async`. Instead, when a `Notify` action fires with a non-empty `notify_url`, a `tokio::spawn` is issued for the outbound HTTP POST (using the existing `reqwest::Client` or a new one-shot client). Fire-and-forget with a 5-second timeout.

**Alternative considered**: Making `execute_actions` async. This cascades async through the CLI path which uses a sync context. The spawn approach keeps the CLI unchanged and avoids touching every caller.

### D5 — Frontend SSE via `EventSource` in a custom hook

**Decision**: `lib/hooks/use-sse-events.ts` opens an `EventSource` to `/api/events/stream`, parses each `data:` frame as a `ServerEvent`, and calls `mutate()` on SWR keys matching the event's `entity`. Devices and automation hooks change `refreshInterval` to `0` (disable polling) once the SSE connection is established.

**Alternative considered**: Global SWR `broadcast` via `useSWRConfig`. Requires knowing all affected SWR keys upfront. The event-driven approach is more precise and degrades gracefully if SSE is not supported (falls back to polling).

## Risks / Trade-offs

- **Broadcast channel overflow** → `broadcast::Sender` capacity set to 128. Slow SSE clients that lag by >128 events get `RecvError::Lagged` and the handler closes the stream, letting the browser reconnect.
- **Lock ordering in restore** → Deadlock risk if any other handler tries to acquire locks in different order during restore. Mitigation: document the canonical lock order (home → automation → scripts → scenes → presence → dashboard) and acquire them all before clearing.
- **`execute_actions` sync/async split** → Webhook dispatch is truly fire-and-forget; errors are logged but not surfaced to callers. This is acceptable for notification side-effects but means no retry or delivery guarantee.
- **History memory** → 500 entries × (approx 80 bytes per `HistoryEntry`) × 1000 devices = ~40 MB worst case. Acceptable for a home system; can be tuned via a `MAX_HISTORY_PER_DEVICE` constant.

## Migration Plan

All changes are additive. No breaking API changes. The SSE endpoint is new; existing polling clients continue to work. The `notify_url` field uses `#[serde(default)]` so existing rules deserialize cleanly. Restore clears state — document this prominently in the OpenAPI description. No DB schema changes required.
