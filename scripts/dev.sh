#!/usr/bin/env bash
# Start backend and frontend concurrently for local development.
# Usage: ./scripts/dev.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Load .env if present
if [[ -f "$ROOT/.env" ]]; then
  set -a; source "$ROOT/.env"; set +a
fi

cleanup() {
  echo "Stopping..."
  kill "$BACKEND_PID" "$FRONTEND_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "▶ Starting backend..."
cd "$ROOT/backend"
cargo run --bin smart_home_server &
BACKEND_PID=$!

echo "▶ Starting frontend..."
cd "$ROOT/frontend"
npm run dev &
FRONTEND_PID=$!

echo ""
echo "  Backend : http://localhost:8080"
echo "  Frontend: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop."
wait
