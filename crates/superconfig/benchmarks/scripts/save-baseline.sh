#!/bin/bash
set -e

# Save benchmark baseline with commit SHA or custom name
# Usage: ./save-baseline.sh [baseline-name]

BASELINE_NAME="${1:-$(git rev-parse --short HEAD)}"
BASELINE_DIR="benchmarks/baselines/$BASELINE_NAME"

echo "📊 Saving benchmark baseline: $BASELINE_NAME"

# Create baseline directory
mkdir -p "$BASELINE_DIR"

# Run benchmarks and save results
echo "Running benchmarks..."
cargo bench --all-features -- --output-format json > "$BASELINE_DIR/results.json" 2>&1

# Copy criterion detailed results  
if [ -d "target/criterion" ]; then
    echo "Copying detailed results..."
    cp -r target/criterion "$BASELINE_DIR/"
fi

# Save metadata
cat > "$BASELINE_DIR/metadata.json" << EOF
{
  "baseline_name": "$BASELINE_NAME",
  "commit_sha": "$(git rev-parse HEAD)",
  "commit_short": "$(git rev-parse --short HEAD)",
  "branch": "$(git branch --show-current)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "rust_version": "$(rustc --version)",
  "cargo_version": "$(cargo --version)"
}
EOF

echo "✅ Baseline saved to: $BASELINE_DIR"
echo "📁 Files created:"
find "$BASELINE_DIR" -type f | head -10