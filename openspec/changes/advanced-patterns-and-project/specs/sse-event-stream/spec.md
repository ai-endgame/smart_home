## ADDED Requirements

### Requirement: SSE broadcast channel in AppState
`AppState` SHALL contain a `tokio::sync::broadcast::Sender<ServerEvent>` with capacity 128. Every call to `record_event` in `http/helpers.rs` SHALL also send the event on this channel after writing to the ring buffer. The sender SHALL be cloned into each handler that needs to emit events.

#### Scenario: Event is broadcast to all connected SSE clients
- **WHEN** `record_event` is called with any `EventKind`
- **THEN** the `ServerEvent` is sent on the broadcast channel within the same async task that calls `record_event`

#### Scenario: Slow consumer is disconnected
- **WHEN** a connected SSE client lags more than 128 events behind
- **THEN** the SSE handler receives `RecvError::Lagged`, closes the stream, and the browser reconnects automatically

### Requirement: GET /api/events/stream SSE endpoint
The system SHALL expose `GET /api/events/stream` that returns an SSE response (`Content-Type: text/event-stream`). Each `ServerEvent` broadcast on the channel SHALL be serialized as `data: <json>\n\n`. The connection SHALL stay open indefinitely until the client disconnects or a lag error occurs. A `retry: 3000` directive SHALL be included in the initial response so browsers auto-reconnect within 3 seconds.

#### Scenario: Client receives live event
- **WHEN** a client holds an open SSE connection and a device state changes
- **THEN** the client receives a `data:` frame with the serialized `ServerEvent` within 100 ms

#### Scenario: Client reconnects after server restart
- **WHEN** the SSE connection is dropped (server restart, lag)
- **THEN** the browser reconnects within 3 seconds due to the `retry: 3000` directive

#### Scenario: Multiple simultaneous SSE clients
- **WHEN** two or more clients are connected to `/api/events/stream`
- **THEN** each client receives every event independently (broadcast fan-out)
