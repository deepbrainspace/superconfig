#!/bin/bash

# Run all examples in the current crate
set -e

echo "🚀 Running all examples in $(pwd)"
echo ""

# Find all example files
for example_file in examples/*.rs; do
    if [ -f "$example_file" ]; then
        example_name=$(basename "$example_file" .rs)
        echo "=== Running $example_name ==="
        if cargo run --example "$example_name"; then
            echo "✅ $example_name completed successfully"
        else
            echo "❌ $example_name failed"
        fi
        echo ""
    fi
done

echo "🎯 All examples completed!"