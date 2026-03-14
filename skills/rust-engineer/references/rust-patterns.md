# Rust Patterns

## Compiler Error Triage

Use this order to fix build failures quickly:

1. Fix type errors first (`E0308`, `E0282`) to reduce cascading diagnostics.
2. Fix ownership and borrow errors next (`E0382`, `E0502`, `E0505`, `E0597`).
3. Resolve trait bound and lifetime issues (`E0277`, `E0310`, `E0521`).
4. Re-run with focused commands (`cargo check -p <crate>`).

Apply these rules:

- Add explicit type annotations at API boundaries before refactoring internals.
- Narrow variable scope to shorten borrows.
- Prefer borrowing (`&T`, `&mut T`) over cloning unless profiles justify copies.
- Refactor long functions into helpers when lifetimes become hard to reason about.

## Clippy Cleanup

Run strict clippy only after code compiles:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Use targeted suppressions only when justified:

- Prefer code changes over `#[allow(...)]`.
- If an allow is necessary, scope it narrowly and document why.
- Avoid module-wide lint disables unless the user explicitly requests it.

## Async and Concurrency Checklist

For `tokio`/async code:

- Do not hold `Mutex`/`RwLock` guards across `.await`.
- Move blocking work to `tokio::task::spawn_blocking`.
- Use bounded channels where backpressure matters.
- Propagate cancellation by returning early on cancelled tasks.
- Add timeout handling for external I/O.

For shared state:

- Prefer ownership transfer via channels over large `Arc<Mutex<_>>` designs.
- Use atomics for simple counters/flags, not full mutable objects.
- Keep critical sections small and deterministic.

## Error Handling Patterns

- Use domain error enums for library and service boundaries.
- Attach context when mapping lower-level errors.
- Avoid collapsing to `String` unless crossing FFI or logging-only boundaries.
- Keep panic usage limited to impossible states and tests.

## Test Strategy

Choose the cheapest test that proves behavior:

- Unit tests for pure transforms, validation, and edge cases.
- Integration tests for I/O boundaries, adapters, and command handlers.
- Regression tests for reported bugs (name tests after issue/symptom).

Useful command sequence:

```bash
cargo test -p <crate>
cargo test --workspace --all-features
```

## Performance Sanity Checks

- Avoid unnecessary allocations in hot loops (`collect`/`clone` chains).
- Prefer iterators and slices where ownership is not required.
- Benchmark only when the change is performance-sensitive:

```bash
cargo bench -p <crate>
```

If no benchmark harness exists, compare representative timings before/after with the same input.
