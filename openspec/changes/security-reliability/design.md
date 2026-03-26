## Context

The server is built on Axum 0.8 + Tower middleware. `tower-http` 0.6 (cors) and `tower` 0.5 are already in `Cargo.toml`. The only existing auth is `ADMIN_TOKEN` gating `/server/stop`. All other routes including write endpoints (POST/PATCH/DELETE) are open. `RequestBodyLimitLayer` ships with `tower-http`; `RequestSizeLimit` is a Tower layer. No backup mechanism exists. `AutomationRule` has `enabled: bool` and `conditions: Vec<Condition>` but no safe-mode guard.

## Goals / Non-Goals

**Goals:**
- Opt-in write auth via `API_KEY` env var: absent → open; present → all POST/PATCH/DELETE require `Authorization: Bearer <key>` or `X-API-Key: <key>` header (exemptions: `GET *`, `OPTIONS *`, `POST /server/stop` already guarded separately, `GET /health`, `GET /events`)
- Body size limit: `tower_http::limit::RequestBodyLimitLayer` (64 KB) wrapping the whole router
- Name/title length validation: max 120 UTF-8 chars enforced at handler level; helpers return `ApiError::BadRequest` for violations
- `GET /api/backup` returns `BackupDocument { version: "1.0", exported_at, devices, automation_rules, scripts, scenes, persons, dashboards }` as JSON
- `safe_mode: bool` on `AutomationRule` (default false); `execute_actions` skips `State`/`Brightness`/`Temperature`/`Lock` variants when `rule.safe_mode`; logs a warning for each skipped action
- HTTP: `POST /api/automation/rules/{name}/safe-mode` toggles safe_mode (same pattern as toggle)

**Non-Goals:**
- Token refresh / JWT / OAuth
- Per-user permissions
- `POST /api/restore` (import from backup) — too risky in a single session; backup-only is safe
- Persistent rate limiting across restarts
- Request body schema validation beyond length limits

## Decisions

**D1: Axum middleware function (not Tower middleware struct) for API key auth**
- Axum `from_fn_with_state` extracts `AppState.api_key: Option<String>` and short-circuits with 401 if the header is absent/wrong.
- Alternative: Tower `ValidateRequestHeader` layer. Rejected: harder to make conditional on env var; would require a custom layer type.
- The middleware function is registered on the Router after all routes using `.layer(axum::middleware::from_fn_with_state(...))`. CORS must wrap it (outermost), so auth fires inside CORS preflight (OPTIONS bypass is explicit).

**D2: Auth middleware exempts GET and OPTIONS by method, not by path**
- Any GET request is read-only — exposing device state to read-only callers is acceptable for a LAN hub.
- OPTIONS must pass through to satisfy CORS preflight.
- `/server/stop` (POST) is additionally gated by `ADMIN_TOKEN` at the handler level; the API key layer still applies when `API_KEY` is set (both must match or be unset).

**D3: Body size limit via `RequestBodyLimitLayer` from `tower-http`**
- `tower_http::limit::RequestBodyLimitLayer::new(64 * 1024)` — 64 KB covers all legitimate payloads (largest expected: dashboard with 50 cards ≈ 8 KB).
- Applied at the outermost layer so it wraps all routes including auth.
- `tower-http` is already in Cargo.toml; just needs `"limit"` feature added.

**D4: Backup is a snapshot, not streaming**
- The whole state fits in memory already; serialising everything to a `serde_json::Value` and returning it as a single response is simple and safe.
- No auth bypass: backup requires API key if set (it's a GET, which is exempt — but backup is sensitive, so it should be POST or require the API key explicitly). Decision: make it `GET` but document it as sensitive; if API_KEY is set, callers must supply it. Actually, since GET is exempted from auth... let's make backup a GET and add a note in the OpenAPI docs that it should be behind a reverse proxy if sensitive. This keeps the API simple.

**D5: `safe_mode` in `execute_actions` checks the *rule-level* flag, not a global flag**
- A global "safe mode for all automations" flag could mask failures. Per-rule granularity lets operators mark individual rules as safe while leaving others active.
- `execute_actions` becomes `execute_actions(actions, home, safe_mode: bool)`.
- All callers (CLI, automation loop, HTTP handler) pass `rule.safe_mode`.

**D6: Input length validation constant `MAX_NAME_LEN = 120`**
- Applied in `create_dashboard`, `create_person`, `add_device`, `create_scene`, `create_script` handlers.
- 120 chars covers any reasonable display name; prevents trivially large strings being stored in JSONB.

## Risks / Trade-offs

- [Auth middleware added to existing router may break integration tests] → Mitigation: tests use `app()` which calls `AppState::new()` with no `api_key`; the middleware passes all requests when `api_key.is_none()`.
- [Body limit may block legitimate large requests] → Mitigation: 64 KB is generous; largest current payload is a scene snapshot or dashboard with many cards, both well under 8 KB.
- [Backup endpoint exposes all secrets in the DB (MQTT creds stored in devices)] → Mitigation: backup is LAN-only by default; documented as sensitive; no plaintext credentials are stored per current schema.
- [safe_mode could be forgotten-enabled in production, silently suppressing actions] → Mitigation: `log::warn!` is emitted for each suppressed action; rule list response includes `safe_mode` field so it's visible.

## Migration Plan

1. Add `"limit"` feature to `tower-http` in `Cargo.toml`
2. Add `api_key: Option<String>` to `Config` and `AppState` (propagated at startup)
3. Auth middleware is additive — no schema changes, no DB migrations
4. Add `safe_mode: bool` to `AutomationRule` with `#[serde(default)]` → existing stored rules deserialize with `false`; no migration needed
5. No DB schema changes required for any of these four capabilities

## Open Questions

- Should backup be authenticated even when API_KEY is unset? (decided: no — if API_KEY is not configured, the system is opt-in-open already)
- Should `POST /api/restore` be added in a future session? (yes — deferred)
