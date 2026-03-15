# Smart Home

A fullstack smart home management system with a Rust/Axum backend and a Next.js dashboard.

## Stack

- **Backend** — Rust, Axum 0.8, sqlx (PostgreSQL), mdns-sd, Tokio
- **Frontend** — Next.js 16 (App Router), TypeScript, Tailwind CSS, SWR
- **Contract** — OpenAPI spec in `contracts/openapi.yaml`

## Project layout

```
smart_home/
├── backend/          Rust API server + interactive CLI
├── frontend/         Next.js dashboard
├── contracts/        openapi.yaml
├── migrations/       001_initial_schema.sql
├── scripts/          dev.sh, gen-types.sh
├── docker-compose.yml
├── Dockerfile.backend
└── Dockerfile.frontend
```

## Getting started

### Local dev (no Docker)

**Prerequisites:** Rust (stable), Node.js 20+, PostgreSQL (optional)

```bash
# 1. Start backend + frontend together
./scripts/dev.sh

# Or separately:
cargo run --bin smart_home_server     # API on http://localhost:8080
cd frontend && npm run dev            # UI  on http://localhost:3000
```

The frontend proxies `/api/*` to the backend, so no CORS configuration is needed during development.

### Docker (full stack)

```bash
docker compose up
```

Starts PostgreSQL, the Rust API server, and the Next.js frontend. Services:

| Service  | URL                   |
|----------|-----------------------|
| Frontend | http://localhost:3000 |
| Backend  | http://localhost:8080 |
| Postgres | localhost:5432        |

## Backend

### Commands

```bash
cargo build
cargo test                            # all tests
cargo test <name>                     # single test by name substring
cargo clippy -- -D warnings
cargo bench --bench smart_home_bench
```

### Binaries

| Binary | Command | Description |
|--------|---------|-------------|
| `smart_home` | `cargo run` | Interactive CLI |
| `smart_home_server` | `cargo run --bin smart_home_server` | HTTP API server |

### Configuration

| Environment variable | Default | Description |
|----------------------|---------|-------------|
| `SMART_HOME_BIND_ADDR` | `127.0.0.1:8080` | Server bind address |
| `DATABASE_URL` | _(none)_ | PostgreSQL URL — persistence is optional |
| `CORS_ORIGINS` | _(allow all)_ | Comma-separated allowed origins |
| `RUST_LOG` | `info` | Log verbosity |

Place these in a `.env` file at the repo root — the server loads it automatically via `dotenvy`.

### PostgreSQL setup

```bash
createdb smart_home
# The server creates the `devices` table on first connection — no migration tool needed.

# Without TLS:
DATABASE_URL=postgres://user:pass@localhost/smart_home?sslmode=disable
```

### Architecture

The backend is a Cargo workspace library (`backend/`) consumed by two binaries. It is split into four layers:

```
domain/          Pure business logic (no framework deps)
infrastructure/  I/O adapters — PostgreSQL (sqlx) and mDNS discovery (mdns-sd)
http/            Axum handlers, router, DTOs, error types, middleware
state.rs         Shared AppState (Arc-wrapped)
config.rs        Typed config loaded from environment
```

**Notable design decisions:**

- **Persistence is optional** — the CLI never persists; the server persists only when `DATABASE_URL` is provided.
- **mDNS runs in a `std::thread`** (not Tokio) because the polling loop is synchronous. A `std::sync::RwLock` is used instead of `tokio::sync::RwLock` — handlers hold the read lock for microseconds so there is no blocking hazard for the async runtime.
- **Automation evaluate/execute split** — `evaluate_rules()` returns a `Vec<Action>` with no side effects; `execute_actions()` applies them. Both the CLI and the HTTP handler use this same pattern.
- **Devices are keyed by lowercased name** — the UUID reverse index (`devices_by_id`) is secondary; the canonical key is always the lowercased name string.

## Frontend

### Commands

```bash
cd frontend
npm install
npm run dev      # dev server on :3000
npm run build    # production build + TypeScript check
```

### Pages

| Route | Description |
|-------|-------------|
| `/` | Dashboard — device counts and status overview |
| `/devices` | Device grid — add, toggle, delete devices |
| `/discovery` | mDNS-discovered devices — add to home |
| `/automation` | Rules list — toggle, delete, run all rules |

### Structure

```
lib/api/      Fetch wrappers per backend domain (devices, discovery, automation)
lib/hooks/    SWR hooks — useDevices, useDiscovery, useAutomation
components/   UI primitives (Badge, Button, Modal), layout, device cards
app/          Next.js App Router pages
```

## API

The full API reference is in [`contracts/openapi.yaml`](contracts/openapi.yaml).

Quick reference:

```
GET    /api/devices
POST   /api/devices
GET    /api/devices/:name
PATCH  /api/devices/:name
DELETE /api/devices/:name
PATCH  /api/devices/:name/state
PATCH  /api/devices/:name/brightness
PATCH  /api/devices/:name/temperature
POST   /api/devices/:name/connect
POST   /api/devices/:name/disconnect
POST   /api/devices/:name/error
POST   /api/devices/:name/error/clear

GET    /api/automation/rules
POST   /api/automation/rules
DELETE /api/automation/rules/:name
POST   /api/automation/rules/:name/toggle
POST   /api/automation/run

GET    /api/discovery/devices
POST   /api/discovery/devices/add

GET    /health
GET    /status
POST   /server/stop
```

## Generating TypeScript types from OpenAPI

```bash
./scripts/gen-types.sh
# Writes to frontend/lib/api/types.generated.ts
```
