---
name: rust-engineer
description: Rust implementation, debugging, refactoring, and code review workflow for production-grade Rust projects. Use when Codex must work on `*.rs` files, `Cargo.toml`, or Rust workspaces, including async/concurrency changes, ownership and lifetime issues, trait/API design, performance tuning, test authoring, or fixing `cargo`/CI failures.
---

# Rust Engineer

## Overview

Deliver safe, idiomatic Rust changes with minimal churn. Keep behavior stable unless the user requests a functional change, and prove modifications with targeted checks.

## Core Workflow

1. Scope the change.
- Identify crate/package boundaries and affected public APIs.
- Detect project conventions from existing code before introducing new patterns.
- Check toolchain constraints (`rust-toolchain*`, workspace settings, lint config).

2. Implement the smallest correct change first.
- Preserve module structure, naming style, and existing error model.
- Prefer explicit ownership and borrowing over cloning by default.
- Avoid `unwrap`/`expect` in non-test code unless failure is unrecoverable and documented.
- Keep new dependencies minimal; reuse existing crates where practical.

3. Make correctness obvious.
- Replace complex branches with typed enums/structs where it clarifies invariants.
- Return structured errors (`Result<T, E>`) with context instead of stringly-typed failures.
- For async code, avoid holding locks across `.await` and avoid blocking calls in async paths.

4. Verify in escalating cost order.
- Run `cargo fmt --all`.
- Run targeted checks first (`cargo test -p <crate>` or focused test names).
- Run workspace checks before finishing:
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  `cargo test --workspace --all-features`

5. Report outcomes precisely.
- List commands executed and what passed/failed.
- Flag residual risks when full validation cannot run.
- Call out API or behavior changes explicitly.

## Code Review Mode

Prioritize findings over summaries. Report:
- Correctness bugs and panic paths.
- Concurrency hazards (deadlocks, lock scope, race-prone shared state).
- Error handling gaps and lossy context propagation.
- Performance regressions from extra allocations, copies, or sync bottlenecks.
- Missing tests for changed behavior.

Use file+line references and order findings by severity.

## Testing Guidance

- Add unit tests for pure logic and boundary cases.
- Add integration tests for public behavior and end-to-end flows.
- For parser/state-machine logic, prefer table-driven tests.
- For bug fixes, add a regression test that fails before and passes after.

## Reference Files

- Use [references/rust-patterns.md](references/rust-patterns.md) for quick triage of compiler errors, clippy noise reduction, async safety checks, and test strategy templates.
