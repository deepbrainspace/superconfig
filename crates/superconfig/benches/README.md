# SuperConfig Benchmarks

Git-based benchmark baseline management for tracking performance across commits and releases.

## Quick Start

```bash
# Save current performance as baseline
moon run superconfig:bench-save

# Compare against a baseline  
moon run superconfig:bench-compare pre-logging
```

## Commands

### Save Baseline

```bash
# Save with commit SHA (default)
./benchmarks/scripts/save-baseline.sh

# Save with custom name
./benchmarks/scripts/save-baseline.sh pre-logging
./benchmarks/scripts/save-baseline.sh v2.0.0-rc1
```

### Compare Against Baseline

```bash
# Compare against default baseline
./benchmarks/scripts/compare-baseline.sh

# Compare against specific baseline  
./benchmarks/scripts/compare-baseline.sh pre-logging
./benchmarks/scripts/compare-baseline.sh d8b7a46
```

## Baseline Structure

```
benchmarks/
├── baselines/
│   ├── v1.0.0/              # Release baselines
│   ├── d8b7a46/             # Commit-specific baselines  
│   ├── pre-logging/         # Feature baselines
│   └── main-latest/         # Latest main branch
└── scripts/
    ├── save-baseline.sh     # Save current performance
    └── compare-baseline.sh  # Compare against baseline
```

## Workflow Examples

### Feature Development

```bash
# Before making changes
./benchmarks/scripts/save-baseline.sh pre-my-feature

# After implementing feature
./benchmarks/scripts/compare-baseline.sh pre-my-feature
```

### Release Process

```bash
# Save release baseline
./benchmarks/scripts/save-baseline.sh v2.0.0

# Compare future changes against release
./benchmarks/scripts/compare-baseline.sh v2.0.0
```

### CI Integration

```bash
# In CI, compare against main branch
./benchmarks/scripts/compare-baseline.sh main-latest
```

## Advantages Over S3/Cloud Storage

✅ **Simple**: No external dependencies or credentials\
✅ **Fast**: Local comparisons, no network overhead\
✅ **Offline**: Works without internet connection\
✅ **Traceable**: Git history tracks when baselines were created\
✅ **Small**: Benchmark files are tiny (< 5KB total)\
✅ **Shareable**: Team members get same baselines via git
