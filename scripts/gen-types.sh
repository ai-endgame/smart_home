#!/usr/bin/env bash
# Generate TypeScript types from the OpenAPI spec.
# Requires: npx (Node.js)
# Usage: ./scripts/gen-types.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SPEC="$ROOT/contracts/openapi.yaml"
OUT="$ROOT/frontend/lib/api/types.generated.ts"

echo "Generating TypeScript types from $SPEC ..."
npx --yes openapi-typescript "$SPEC" -o "$OUT"
echo "Written to $OUT"
