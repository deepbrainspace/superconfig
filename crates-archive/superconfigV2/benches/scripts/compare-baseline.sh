#!/bin/bash
set -e

# Compare current performance against a saved baseline
# Usage: ./compare-baseline.sh [baseline-name]

BASELINE_NAME="${1:-pre-logging}"
BASELINE_DIR="benches/baselines/$BASELINE_NAME"

if [ ! -d "$BASELINE_DIR" ]; then
    echo "❌ Baseline '$BASELINE_NAME' not found in $BASELINE_DIR"
    echo "Available baselines:"
    ls -1 benches/baselines/ 2>/dev/null || echo "  (none)"
    exit 1
fi

echo "📊 Comparing against baseline: $BASELINE_NAME"

# Run current benchmarks
echo "Running current benchmarks..."
cargo bench --all-features -- --baseline "$BASELINE_NAME"

echo "✅ Comparison complete!"
echo "📁 Baseline info:"
cat "$BASELINE_DIR/metadata.json" | jq -r '"  Commit: \(.commit_short) (\(.branch))", "  Created: \(.timestamp)"'