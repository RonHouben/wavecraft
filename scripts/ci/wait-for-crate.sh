#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/ci/wait-for-crate.sh <crate> <version> [max_attempts] [delay_seconds]
# Waits until crates.io reports the given crate version as available.

CRATE="${1:-}"
VERSION="${2:-}"
MAX_ATTEMPTS="${3:-30}"
DELAY="${4:-10}"

if [ -z "$CRATE" ] || [ -z "$VERSION" ]; then
  echo "⏩ Skipping wait (missing crate or version)"
  exit 0
fi

echo "Waiting for $CRATE@$VERSION..."
for i in $(seq 1 "$MAX_ATTEMPTS"); do
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "User-Agent: wavecraft-ci (https://github.com/RonHouben/wavecraft)" \
    "https://crates.io/api/v1/crates/$CRATE/$VERSION")

  if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ $CRATE@$VERSION available on crates.io (attempt $i)"
    exit 0
  fi

  echo "⏳ $CRATE@$VERSION not yet available (HTTP $HTTP_CODE, attempt $i/$MAX_ATTEMPTS, waiting ${DELAY}s...)"
  sleep "$DELAY"
done

echo "❌ Timed out waiting for $CRATE@$VERSION after $((MAX_ATTEMPTS * DELAY))s"
exit 1
