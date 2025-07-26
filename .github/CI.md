# CI/CD Pipeline Documentation

This document explains the Continuous Integration workflow for the SuperConfig project and how to work with it effectively.

## Overview

Our CI pipeline is designed with a **3-phase sequential structure** to provide fast feedback while avoiding resource conflicts that can occur when multiple jobs run identical tasks simultaneously.

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
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│   │    Test     │ │   Quality   │ │  Security   │   │
│   │             │ │             │ │             │   │
│   │    test     │ │ fmt-check   │ │   audit     │   │
│   │             │ │   clippy    │ │    deny     │   │
│   └─────────────┘ └─────────────┘ └─────────────┘   │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              PHASE 2: SEQUENTIAL                    │
│                ┌─────────────┐                      │
│                │    Build    │                      │
│                │             │                      │
│                │    build    │                      │
│                │ build-rel   │                      │
│                └─────────────┘                      │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              PHASE 3: SEQUENTIAL                    │
│                ┌─────────────┐                      │
│                │  Coverage   │                      │
│                │             │                      │
│                │  coverage   │                      │
│                │   upload    │                      │
│                └─────────────┘                      │
└─────────────────────────────────────────────────────┘
```

**Key Benefits:**
- **Phase 1**: Fast parallel validation catches most issues quickly
- **Phase 2**: Only builds if validation passes (saves time on failures)  
- **Phase 3**: Expensive coverage analysis only runs if everything else works
- **Fail-Fast**: Pipeline stops immediately on any failure

## Why This Structure?

### Problem We Solved
Previously, the CI ran `moon check` which executed ALL tasks (test, clippy, build, coverage, etc.) while other specialized jobs were running the same tasks in parallel. This caused:
- File lock conflicts
- Resource contention  
- Inconsistent failures
- Slower overall execution

### Solution Benefits
1. **Fast Feedback**: Critical validation (tests, linting, security) runs immediately in parallel
2. **Fail Fast**: If basic validation fails, expensive build/coverage steps are skipped
3. **No Conflicts**: Each task runs in only one job, eliminating race conditions
4. **Clear Failures**: Easy to identify which specific check failed

## Running Locally

You can run the same checks locally using Moon:

### Phase 1 Checks (Parallel)
```bash
# Run all Phase 1 checks
moon run superconfig:test superconfig:fmt-check superconfig:clippy superconfig:audit superconfig:deny

# Or individually:
moon run superconfig:test        # All tests (unit + integration + doc)
moon run superconfig:fmt-check   # Format validation
moon run superconfig:clippy      # Linting
moon run superconfig:audit       # Security audit
moon run superconfig:deny        # Policy checks
```

### Phase 2 Checks
```bash
moon run superconfig:build superconfig:build-release
```

### Phase 3 Checks  
```bash
moon run superconfig:coverage
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

The CI uses multiple layers of caching for performance:
- **Tool Cache**: Moon, Proto, Rust toolchain
- **Cargo Cache**: Registry, git dependencies  
- **Target Cache**: Compiled artifacts
- **Moon Cache**: Task outputs and intermediate results


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