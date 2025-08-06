# SuperConfig Development Commands

## Essential Moon Commands

### Build Commands

```bash
# Build all affected crates
moon run --affected :build

# Build specific crate
moon run superconfig:build

# Release build for all affected
moon run --affected :build-release

# Build specific crate in release mode
moon run logffi:build-release
```

### Testing Commands

```bash
# Run tests for all affected crates
moon run --affected :test

# Test specific crate
moon run superconfig:test

# Run tests without default features
moon run superconfig:test-no-default
```

### Code Quality Commands

```bash
# Format all affected code
moon run --affected :format

# Check formatting without changes
moon run --affected :format-check

# Lint all affected crates
moon run --affected :lint

# Run all quality checks
moon run --affected :format-check
moon run --affected :lint
```

### Coverage Commands

```bash
# Generate coverage summary
moon run superconfig:coverage

# Generate HTML coverage report
moon run superconfig:coverage-html

# Open coverage report in browser (WSL compatible)
moon run superconfig:coverage-open

# Generate CI-friendly coverage
moon run superconfig:coverage-ci
```

### Security & Compliance

```bash
# Run security audit
moon run --affected :security-audit

# Run cargo-deny checks
moon run --affected :deny

# Check for outdated dependencies
moon run --affected :outdated
```

### Documentation

```bash
# Build documentation
moon run superconfig:doc

# Build and open documentation
moon run superconfig:doc-open
```

### Publishing (when ready)

```bash
# Dry-run publish
moon run superconfig:publish-dry

# Publish crate (requires approval)
moon run superconfig:publish

# Auto-publish without confirmation
moon run superconfig:publish-auto
```

## Direct Cargo Commands (if needed)

Note: Always prefer Moon commands, but these work from within crate directories:

```bash
# Navigate to crate first
cd crates/superconfig

# Standard cargo commands
cargo build
cargo test
cargo fmt
cargo clippy
cargo doc --open
```

## Git Commands

```bash
# Check repository status
git status

# Create conventional commit (enforced by hooks)
git commit -m "feat(superconfig): add new configuration provider"

# The hooks will automatically:
# - Format code (rust, python, js, markdown)
# - Run lints
# - Validate commit messages
# - Check for clean repo state
```

## Utility Commands (Linux/WSL)

```bash
# File operations
ls -la
find . -name "*.rs"
grep -r "pattern" crates/

# Use ripgrep for faster search
rg "pattern" crates/

# Process management
ps aux | grep cargo
htop

# System info
uname -a
lscpu
df -h
```

## Moon Query Commands

```bash
# Show all projects
moon query projects

# Show affected projects
moon query projects --affected

# Show project dependency graph
moon query graph

# Show task details
moon query tasks
```

## Environment Setup

```bash
# Check Rust toolchain
rustup show

# Check Moon version
moon --version

# Check git hooks
lefthook version

# Check all proto tools
proto list
```
