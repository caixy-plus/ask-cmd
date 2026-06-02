#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}"

echo "=== cargo test ==="
cargo test --all

echo "=== build release ==="
cargo build --release

BIN="./target/release/ask"
echo "=== dry-run help ==="
$BIN --help >/dev/null

echo "=== unit pipeline (no network) ==="
# extract/validate only via embedded tests above

if command -v claude >/dev/null 2>&1; then
  echo "=== live claude dry-run ==="
  OUT=$($BIN -n "create empty file /tmp/ask-cmd-smoke-$$.txt" 2>/dev/null | head -1)
  echo "got: $OUT"
  [[ "$OUT" == touch* ]] || [[ "$OUT" == *New-Item* ]] || { echo "unexpected: $OUT"; exit 1; }
  echo "PASS live smoke"
else
  echo "SKIP live claude (not installed)"
fi

echo "=== all passed ==="
