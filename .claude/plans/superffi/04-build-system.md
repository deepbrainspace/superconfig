# SuperFFI Build System & CI/CD

This document details the Moon-based build orchestration and GitHub Actions CI/CD pipeline for multi-language package distribution.

## CI/CD Orchestration Flow

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Git Source    │    │  Moon Tasks     │    │   Published     │
│                 │    │                 │    │   Packages      │
│ Rust Code       │───▶│ Build Pipeline  │───▶│ PyPI/npm        │
│ Config Files    │    │ Test & Package  │    │ User Installs   │
│ Package Configs │    │ Publish         │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Project Structure & Task Organization

### Rust Crate Tasks (`crates/superconfig-ffi/moon.yml`)

```yaml
# crates/superconfig-ffi/moon.yml - Rust development tasks
language: 'rust'
type: 'library'

tasks:
  test:
    command: 'cargo test'
    inputs: ['src/**/*', 'Cargo.toml']
    
  lint:
    command: 'cargo clippy -- -D warnings'
    inputs: ['src/**/*', 'Cargo.toml']
    
  format:
    command: 'cargo fmt --check'
    inputs: ['src/**/*', 'Cargo.toml']
```

### Python Binding Tasks (`bindings/python/moon.yml`)

```yaml
# bindings/python/moon.yml - Project: superconfig/python
language: 'python'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features python'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'maturin build --manifest-path ../../Cargo.toml --features python'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['dist/*.whl']
    deps: ['check']
    
  test:
    command: 'python -m pytest tests/'
    inputs: ['tests/**/*', 'dist/*.whl']
    deps: ['build']
    
  publish:
    command: 'twine upload dist/*'
    deps: ['test']
    env:
      TWINE_PASSWORD: '$PYPI_TOKEN'
```

### Node.js Binding Tasks (`bindings/nodejs/moon.yml`)

```yaml
# bindings/nodejs/moon.yml - Project: superconfig/nodejs
language: 'javascript'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features nodejs'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'napi build --manifest-path ../../Cargo.toml --features nodejs --release'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['lib/']
    deps: ['check']
    
  test:
    command: 'npm test'
    inputs: ['tests/**/*', 'lib/**/*']
    deps: ['build']
    
  publish:
    command: 'npm publish'
    deps: ['test']
    env:
      NPM_TOKEN: '$NPM_TOKEN'
```

### WebAssembly Binding Tasks (`bindings/wasm/moon.yml`)

```yaml
# bindings/wasm/moon.yml - Project: superconfig/wasm
language: 'javascript'
type: 'library'

tasks:
  check:
    command: 'cargo check --manifest-path ../../Cargo.toml --features wasm'
    inputs: ['../../src/**/*', '../../Cargo.toml']
    
  build:
    command: 'wasm-pack build --manifest-path ../../Cargo.toml --features wasm --target web'
    inputs: ['../../src/**/*', '../../Cargo.toml', '**/*']
    outputs: ['pkg/']
    deps: ['check']
    
  test:
    command: 'npm test'
    inputs: ['tests/**/*', 'pkg/**/*']
    deps: ['build']
    
  publish:
    command: 'npm publish'
    deps: ['test']
    env:
      NPM_TOKEN: '$NPM_TOKEN'
```

### Workspace-Level Tasks (`moon.yml`)

```yaml
# Root workspace moon.yml - Top-level orchestration
tasks:
  test-all:
    deps: ['superconfig-ffi:test', 'superconfig/python:test', 'superconfig/nodejs:test', 'superconfig/wasm:test']
    
  build-all:
    deps: ['superconfig/python:build', 'superconfig/nodejs:build', 'superconfig/wasm:build']
    
  publish-all:
    deps: ['superconfig/python:publish', 'superconfig/nodejs:publish', 'superconfig/wasm:publish']
```

## Development Commands

### Rust Development (Crate-Level)

```bash
# Rust crate development and testing
moon run superconfig-ffi:test        # Unit tests for Rust FFI wrapper
moon run superconfig-ffi:lint        # Clippy linting
moon run superconfig-ffi:format      # Format checking
```

### Language-Specific Development

```bash
# Check Rust compilation with specific features
moon run superconfig/python:check   # cargo check --features python
moon run superconfig/nodejs:check   # cargo check --features nodejs
moon run superconfig/wasm:check     # cargo check --features wasm

# Build language-specific packages
moon run superconfig/python:build   # maturin build → .whl
moon run superconfig/nodejs:build   # napi build → .node + .tgz
moon run superconfig/wasm:build     # wasm-pack build → .wasm + JS

# Test language bindings (integration tests)
moon run superconfig/python:test    # pytest on Python bindings
moon run superconfig/nodejs:test    # npm test on Node.js bindings
moon run superconfig/wasm:test      # npm test on WASM bindings
```

### Publishing

```bash
# Publish individual packages
moon run superconfig/python:publish  # twine upload → PyPI
moon run superconfig/nodejs:publish  # npm publish → npm
moon run superconfig/wasm:publish    # npm publish → npm (as superconfig-wasm)
```

### Workspace Orchestration

```bash
# High-level commands that coordinate multiple projects
moon run test-all      # Test Rust crate + all language bindings
moon run build-all     # Build all language packages
moon run publish-all   # Publish all packages to registries
```

## Testing Strategy

### Two-Level Testing Approach

**Level 1: Rust Unit Tests** (`superconfig-ffi:test`)

- Tests the Rust FFI wrapper logic
- Validates JSON parameter conversions
- Tests feature flag compilation
- Runs in `crates/superconfig-ffi/`

**Level 2: Language Integration Tests** (`superconfig/*/test`)

- Tests actual generated bindings work correctly
- Validates APIs from user perspective
- Tests cross-language data marshaling
- Runs in each `bindings/*/` directory

### Test Dependencies

```bash
# Tests run in dependency order automatically
moon run superconfig/python:test
# → Depends on: superconfig/python:build
# → → Depends on: superconfig/python:check
# → → → Depends on: superconfig-ffi:lint (implicitly)
```

## GitHub Actions Integration

### Multi-Language Release Workflow (`.github/workflows/release.yml`)

```yaml
name: Release Multi-Language Packages

on:
  push:
    tags: ['v*']

jobs:
  release-all:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Moon
        run: |
          curl -fsSL https://moonrepo.dev/install/moon.sh | bash
          echo "$HOME/.moon/bin" >> $GITHUB_PATH
      
      - name: Setup build tools
        run: |
          pip install maturin twine pytest
          npm install -g @napi-rs/cli
          cargo install wasm-pack
      
      - name: Run all tests
        run: moon run test-all
      
      - name: Build and publish all packages
        run: moon run publish-all
        env:
          PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
          NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Pipeline Execution Flow

When `moon run publish-all` executes:

1. **Rust Development Tasks** (if needed):
   ```bash
   moon run superconfig-ffi:test     # Rust unit tests
   moon run superconfig-ffi:lint     # Rust code quality
   ```

2. **Language Compilation Checks**:
   ```bash
   moon run superconfig/python:check   # cargo check --features python
   moon run superconfig/nodejs:check   # cargo check --features nodejs  
   moon run superconfig/wasm:check     # cargo check --features wasm
   ```

3. **Build Native Packages**:
   ```bash
   moon run superconfig/python:build   # maturin → .whl
   moon run superconfig/nodejs:build   # napi → .node + .tgz
   moon run superconfig/wasm:build     # wasm-pack → .wasm + JS
   ```

4. **Integration Testing**:
   ```bash
   moon run superconfig/python:test    # pytest integration tests
   moon run superconfig/nodejs:test    # npm test integration tests
   moon run superconfig/wasm:test      # npm test WASM integration
   ```

5. **Registry Publishing**:
   ```bash
   moon run superconfig/python:publish  # twine → PyPI
   moon run superconfig/nodejs:publish  # npm → npm registry
   moon run superconfig/wasm:publish    # npm → npm registry
   ```

## Key Architecture Benefits

- **Separation of Concerns**: Rust testing vs language integration testing
- **Dependency Management**: Moon ensures correct execution order
- **Parallel Execution**: Independent language builds run concurrently
- **Fail Fast**: If any test fails, publishing stops
- **Monorepo Scalability**: Clean naming prevents conflicts with other FFI crates

---

_See [`project-structure.md`](./project-structure.md) for directory organization and [`architecture.md`](./architecture.md) for technical architecture_
