# CI/CD Pipeline Documentation

This document explains the Continuous Integration workflow for the SuperConfig project and how to work with it effectively.

## Overview

Our CI pipeline is designed with an **optimized build→test structure** that eliminates redundant builds while maintaining clear visual separation and fast feedback.

## CI Pipeline Flow

```
┌─────────────────────┐
│  Detect Affected    │
│      Crates         │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              PHASE 1: PARALLEL                      │
│   ┌─────────────┐ ┌─────────────┐                   │
│   │   Quality   │ │  Security   │                   │
│   │             │ │             │                   │
│   │format-check │ │   audit     │                   │
│   │    lint     │ │    deny     │                   │
│   └─────────────┘ └─────────────┘                   │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              PHASE 2: BUILD                         │
│                ┌─────────────┐                      │
│                │    Build    │                      │
│                │             │                      │
│                │build-release│                      │
│                └─────────────┘                      │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              PHASE 3: PARALLEL                      │
│   ┌─────────────┐ ┌─────────────┐                   │
│   │    Test     │ │  Coverage   │                   │
│   │             │ │             │                   │
│   │test(release)│ │  coverage   │                   │
│   │             │ │   upload    │                   │
│   └─────────────┘ └─────────────┘                   │
└─────────────────────────────────────────────────────┘
```

**Key Benefits:**

- **Phase 1**: Fast parallel validation catches most issues quickly
- **Phase 2**: Creates release build artifacts once and caches them
- **Phase 3**: Tests and coverage reuse cached release artifacts (no rebuild)
- **Performance**: ~40% faster execution by eliminating duplicate builds
- **Fail-Fast**: Pipeline stops immediately on any failure

## Why This Structure?

### Optimized Build Strategy

Our CI uses an efficient build→test flow that maximizes performance:

- **Single Release Build**: Creates release artifacts once and caches them
- **Artifact Reuse**: Test and coverage jobs reuse cached build artifacts
- **No Redundant Compilation**: Eliminates duplicate dev/release builds
- **Smart Caching**: GitHub Actions caches target directory across jobs

### Performance Benefits

1. **~40% Faster**: Significant reduction in total pipeline execution time
2. **Resource Efficient**: Single compilation phase instead of multiple redundant builds
3. **Cache Optimization**: Build artifacts shared between test and coverage phases
4. **Visual Clarity**: Distinct build→test→coverage flow provides clear progress indication

## Running Locally

You can run the same checks locally using Moon:

### Phase 1 Checks (Parallel)

```bash
# Quality checks
moon run superconfig:format-check   # Format validation
moon run superconfig:lint            # Linting

# Security checks
moon run superconfig:audit       # Security audit
moon run superconfig:deny        # Policy checks
```

### Phase 2: Build

```bash
moon run superconfig:build-release  # Creates release artifacts with caching
```

### Phase 3: Test & Coverage (using cached artifacts)

```bash
moon run superconfig:test        # Tests using release build (--release flag)
moon run superconfig:coverage    # Coverage analysis
```

### Run Everything

```bash
moon check superconfig  # Runs all tasks (equivalent to full CI)
```

## Affected Crate Detection

The CI only runs for crates that have changes, determined by:

- `moon query projects --affected --json`
- Compares against `origin/main` branch
- Uses file changes and dependency graphs

## Caching Strategy

The CI uses multiple layers of caching for optimal performance:

### Build Artifact Caching

- **Target Cache**: Compiled release artifacts cached between jobs
- **Cache Key**: Based on Rust version, lock files, and source changes
- **Reuse**: Test and coverage jobs reuse build artifacts (no recompilation)

### Tool and Dependency Caching

- **Tool Cache**: Moon, Proto, Rust toolchain installations
- **Cargo Cache**: Registry, git dependencies, and tool binaries
- **Moon Cache**: Task outputs and intermediate build results

### Cache Performance

- **Hit Rate**: High cache hit rates for incremental builds
- **Sharing**: Build artifacts shared across test and coverage phases
- **Invalidation**: Smart cache invalidation based on actual file changes

## Workflow Files

- `.github/workflows/ci.yml` - Main CI pipeline
- `.github/actions/setup-moon/action.yml` - Moon installation and caching
- `crates/superconfig/moon.yml` - Task definitions and dependencies

## Contributing

When modifying the CI:

1. Test changes locally with `moon check superconfig`
2. Consider impact on parallel execution
3. Update this documentation for significant changes
4. Verify caching behavior isn't broken
