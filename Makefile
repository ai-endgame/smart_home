.PHONY: build run run-server test lint lint-fix clean fmt check all

# Build the project
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run the application
run:
	cargo run

# Run the HTTP server binary
run-server:
	cargo run --bin smart_home_server

# Run all tests
test:
	cargo test

# Run tests with verbose output
test-verbose:
	cargo test -- --nocapture

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Run clippy and automatically fix issues where possible
lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged

# Format code
fmt:
	cargo fmt

# Check formatting without modifying files
fmt-check:
	cargo fmt -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# Full CI pipeline
all: fmt-check lint test build

# Clean build artifacts
clean:
	cargo clean
