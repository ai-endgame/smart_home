.PHONY: build run run-server run-server-db run-server-mqtt mqtt-broker commission-sidecar test lint lint-fix clean fmt check all db db-stop db-logs

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

# Run the HTTP server with PostgreSQL persistence + MQTT bridge
run-server-db: db mqtt-broker
	DATABASE_URL=postgres://smart_home:smart_home@localhost:5432/smart_home \
	MQTT_URL=mqtt://localhost:1883 \
	cargo run --bin smart_home_server

# Start the chip-tool Docker sidecar (for Matter commissioning + control)
commission-sidecar:
	docker compose up chip-tool -d

# Start a local Mosquitto MQTT broker (for dev/testing)
mqtt-broker:
	docker run -d --name mosquitto -p 1883:1883 eclipse-mosquitto || docker start mosquitto

# Run the HTTP server with PostgreSQL + MQTT bridge
run-server-mqtt: db mqtt-broker
	DATABASE_URL=postgres://smart_home:smart_home@localhost:5432/smart_home \
	MQTT_URL=mqtt://localhost:1883 \
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

# Start only the PostgreSQL container (detached)
db:
	docker compose up postgres -d

# Stop the PostgreSQL container
db-stop:
	docker compose stop postgres

# Tail PostgreSQL logs
db-logs:
	docker compose logs -f postgres

# Clean build artifacts
clean:
	cargo clean
