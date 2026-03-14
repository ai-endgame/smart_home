# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run CLI (interactive)
cargo run

# Run HTTP server (default: 127.0.0.1:8080)
cargo run --bin smart_home_server
cargo run --bin smart_home_server -- --addr 0.0.0.0:9090

# Test
cargo test
cargo test <test_name>          # run a single test by name
cargo test -p smart_home        # run all tests in the package

# Lint
cargo clippy -- -D warnings

# Benchmarks (generates HTML reports in target/criterion/)
cargo bench
cargo bench --bench smart_home_bench <benchmark_name>
```

Control log verbosity with `RUST_LOG` (e.g. `RUST_LOG=debug cargo run`).

## Architecture

The crate exposes a library (`src/lib.rs`) used by two binaries:

- **`src/main.rs`** — interactive CLI (`cargo run`)
- **`src/bin/smart_home_server.rs`** — Axum HTTP server (`cargo run --bin smart_home_server`)

### Core modules

| Module | Responsibility |
|--------|---------------|
| `models` | `Device`, `Room`, `DeviceType`, `DeviceState` — plain data types with `Display` impls |
| `manager` | `SmartHome` — in-memory store of devices and rooms; all lookups are case-insensitive (keys stored lowercased). Maintains a `devices_by_id` reverse index for O(1) room-device lookups |
| `automation` | `AutomationEngine` + `AutomationRule` — trigger/action rules evaluated against a `SmartHome` snapshot; rules stored in a `Vec` (O(n) duplicate check) |
| `server` | Axum REST API wrapping `SmartHome` and `AutomationEngine` behind `Arc<RwLock<_>>`; also manages client sessions and a server-side event log |
| `cli` | Interactive REPL that owns its own `SmartHome` and `AutomationEngine` instances |
| `logger` | Thin wrapper around `env_logger` |

### Key design points

- **State is not persisted** — both the CLI and server start with empty state on each run.
- **Devices are keyed by lowercased name** in `SmartHome::devices`. The UUID-based `devices_by_id` map is a secondary index only; the canonical key is always the lowercased name.
- **Automation evaluation is decoupled from execution**: `evaluate_rules` returns a `Vec<Action>` without side effects; `execute_actions` applies them. This separation is used in both the CLI (`run-rules` command) and the server.
- **Server state** is all in `AppState` (shared via `Arc`) and is in-memory; the shutdown channel (`oneshot::Sender`) lives inside a `Mutex` so it can be consumed exactly once.
- Device-type-specific fields (`brightness` for lights, `temperature` for thermostats) are always present on `Device` but only meaningful for the relevant type — the manager enforces this at the method level.
