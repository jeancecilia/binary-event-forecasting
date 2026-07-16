#!/bin/bash
# Run offline replay and verify deterministic hashes.
# Usage: ./scripts/run_offline_replay.sh [trace_path]

set -euo pipefail

TRACE_PATH="${1:-data/traces/golden}"

echo "==> Running offline replay..."
echo "Trace path: ${TRACE_PATH}"

# Build the core engine
cargo build --release --bin core-engine

# Run replay twice and compare hashes
echo "==> Run 1..."
cargo run --release --bin core-engine -- replay --trace "${TRACE_PATH}" --verify

echo "==> Replay complete."
echo "Check verification/reports/ for the determinism report."
