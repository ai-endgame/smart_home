# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository layout

```
smart_home/
├── backend/        Rust/Axum API server + CLI (Cargo workspace member)
├── frontend/       Next.js 16 dashboard (TypeScript, Tailwind, SWR)
├── contracts/      openapi.yaml — source of truth for the API contract
├── migrations/     001_initial_schema.sql (also applied inline at startup)
├── scripts/        dev.sh, gen-types.sh
├── docker-compose.yml
├── Dockerfile.backend / Dockerfile.frontend
└── Cargo.toml      Workspace root (members = ["backend"])
```

## Backend commands

All `cargo` commands run from the repo root or `backend/`:

```bash
cargo build
cargo run --bin smart_home           # interactive CLI
cargo run --bin smart_home_server    # HTTP server on 127.0.0.1:8080
cargo run --bin smart_home_server -- --addr 0.0.0.0:9090
cargo run --bin smart_home_server -- --database-url postgres://...

cargo test                           # all tests (unit + integration + binary)
cargo test <name>                    # single test by name substring
cargo clippy -- -D warnings
cargo bench --bench smart_home_bench
```

`RUST_LOG=debug` controls log verbosity. Server also loads `.env` via `dotenvy` at startup.

### PostgreSQL (optional)

```bash
createdb smart_home
DATABASE_URL=postgres://user:pass@localhost/smart_home?sslmode=disable cargo run --bin smart_home_server
```

Schema is a single `devices` table applied inline on first connection — no migration tool needed.

## Frontend commands

```bash
cd frontend
npm run dev      # dev server on :3000
npm run build    # production build + TypeScript check
```

`frontend/.env.local` sets `NEXT_PUBLIC_API_URL=http://localhost:8080`.

Next.js rewrites `/api/*` → backend in `next.config.ts`, so components call `/api/devices` and the proxy forwards to the Rust server.

## Full-stack dev

```bash
./scripts/dev.sh          # starts backend (:8080) and frontend (:3000) concurrently
docker compose up         # Postgres + backend + frontend in containers
```

## Backend architecture

The `backend/` crate is a library + two binaries. The library is split into four layers:

```
domain/        Pure business logic — no framework deps
  device.rs      Device, Room, DeviceType, DeviceState
  manager.rs     SmartHome — in-memory store, case-insensitive name keys, UUID reverse index
  automation.rs  AutomationEngine — Vec<AutomationRule>, evaluate/execute split

infrastructure/  I/O adapters
  db.rs          sqlx PgPool — upsert_device / delete_device / load_all_devices
  mdns.rs        mDNS discovery in a std::thread (not tokio); DiscoveryStore = Arc<std::sync::RwLock<HashMap>>

http/            Axum transport layer
  handlers/      devices, automation, discovery, system — one file per domain
  router.rs      build() — all routes (both /api/* prefixed and legacy unprefixed aliases)
  types.rs       Request/response DTOs + to_domain() converters
  errors.rs      ApiError enum → IntoResponse
  helpers.rs     persist_device, record_event, device_to_response, etc.
  middleware.rs  cors_layer()
  mod.rs         run_server_full(), run_server(), ServerStartError, all HTTP integration tests

state.rs         AppState (Arc-wrapped, Clone) — home, automation, events, clients, db, discovery, shutdown_tx
config.rs        Config::from_env() — reads SMART_HOME_BIND_ADDR / SMART_HOME_SERVER_ADDR / DATABASE_URL / CORS_ORIGINS
```

### Key design decisions

- **Persistence is optional** — CLI never persists. Server only persists when `DATABASE_URL` is set. `SmartHome::insert_device` rehydrates DB rows with original UUIDs (bypasses `add_device` UUID generation) while still populating `devices_by_id`.
- **Discovery uses `std::sync::RwLock`** — the mDNS polling loop is a `std::thread` that cannot call async. Handlers hold the read lock for microseconds; no async blocking hazard.
- **Devices keyed by lowercased name** — `devices_by_id` is a secondary index only. The canonical key is always the lowercased name string.
- **Automation evaluate/execute split** — `evaluate_rules(&home) -> Vec<Action>` has no side effects; `execute_actions(&actions, &mut home)` applies them. Used the same way in CLI and HTTP handler.
- **Shutdown channel** — `oneshot::Sender<()>` lives in `AppState.shutdown_tx: Arc<Mutex<Option<...>>>` so it can be consumed exactly once via the `/server/stop` endpoint.
- **Route aliasing** — `router::build()` registers every route twice: once under `/api/*` and once as a legacy unprefixed alias (e.g. `/devices`). Both serve identical handlers. Tests use the legacy paths.
- **`Config.bind_addr` is a raw `String`** — parsing to `SocketAddr` happens inside `run_server_full`, so an invalid address surfaces as `ServerStartError::InvalidBindAddress` rather than a panic at config load time.

## Frontend architecture

Next.js App Router (`src/app/`). All data-fetching is client-side via SWR.

```
lib/api/        Thin fetch wrappers — one file per backend domain
  client.ts       apiFetch<T>() — base fetcher with error handling
  types.ts        Shared TypeScript interfaces mirroring the OpenAPI schema
lib/hooks/      SWR hooks (use-devices, use-discovery, use-automation)
components/
  ui/             Badge, Button, Modal — design-system primitives
  layout/nav.tsx  Sticky nav with active-route highlight
  devices/        DeviceCard, AddDeviceModal
app/
  page.tsx        Dashboard (metrics overview)
  devices/        Device grid + add modal
  discovery/      mDNS discovered devices + "Add to Home"
  automation/     Rules list + toggle/delete/run
```

SWR keys match the `/api/*` route paths (e.g. `'/api/devices'`) so cache invalidation is predictable.
