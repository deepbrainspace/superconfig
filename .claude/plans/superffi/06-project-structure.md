# SuperFFI Project Structure

This document outlines the complete directory structure for the SuperFFI multi-language FFI implementation within the SuperConfig Moon monorepo.

## Complete Directory Layout

```
superconfig/                    # Moon workspace root
├── .github/workflows/          # GitHub Actions (Moon-based CI/CD)
│   └── release.yml            # Automated package publishing
├── moon.yml                   # Workspace-level tasks and config
├── .moon/                     # Moon metadata (generated, gitignored)
├── crates/                    # Rust crates (all go in Git)
│   ├── superconfig/           # Core Rust API project (published to crates.io)
│   │   ├── moon.yml          # Project-specific tasks
│   │   ├── Cargo.toml
│   │   └── src/lib.rs        # Native Rust API (unchanged)
│   ├── superffi/             # ✅ COMPLETED - Reusable FFI macro generator
│   │   ├── moon.yml          # Macro development tasks
│   │   ├── Cargo.toml        # proc-macro = true
│   │   ├── README.md         # Comprehensive documentation
│   │   └── src/lib.rs        # Generates PyO3/NAPI/wasm-bindgen annotations
│   └── superconfig-ffi/       # FFI wrapper (Rust code only)
│       ├── moon.yml          # Build tasks for all targets
│       ├── Cargo.toml        # Uses superffi macro, feature flags
│       └── src/lib.rs        # JSON wrapper using #[superffi] annotations
├── bindings/                  # Language-specific packaging (all go in Git)
│   ├── python/               # Python packaging config → PyPI as "superconfig"
│   │   ├── moon.yml          # Python build/publish tasks
│   │   ├── setup.py          # Maturin configuration
│   │   ├── pyproject.toml    # Modern Python packaging
│   │   └── superconfig/      # Python module structure
│   │       └── __init__.py   # Python entry point
│   ├── nodejs/               # Node.js packaging config → npm as "superconfig"
│   │   ├── moon.yml          # Node.js build/publish tasks
│   │   ├── package.json      # npm package configuration
│   │   ├── index.js          # JavaScript entry point
│   │   └── src/              # JS wrapper code
│   └── wasm/                 # WASM packaging config → npm as "superconfig-wasm"
│       ├── moon.yml          # WASM build/publish tasks
│       ├── package.json      # WASM package configuration
│       ├── webpack.config.js # Bundling configuration
│       └── src/              # TypeScript wrappers
├── target/                   # ❌ Gitignored - Rust build artifacts
├── dist/                     # ❌ Gitignored - Final distribution packages
└── examples/                 # Usage examples (go in Git)
    ├── rust/                 # Using core superconfig
    ├── python/               # Using published Python package
    └── nodejs/               # Using published Node.js package
```

## Source Code vs Build Artifacts

### ✅ **Source Code (Git Tracked)**

These files are authored by developers and stored in version control:

**Rust Source Code**:

- `crates/superconfig/src/` - Core SuperConfig implementation (unchanged)
- `crates/superffi/src/` - SuperFFI macro generator (✅ COMPLETED)
- `crates/superconfig-ffi/src/` - FFI wrapper implementation

**Packaging Configuration**:

- `bindings/python/setup.py` - Python wheel configuration
- `bindings/nodejs/package.json` - npm package configuration
- `bindings/wasm/package.json` - WebAssembly package configuration

**Build System**:

- `moon.yml` files - Moon task definitions and dependencies
- `.github/workflows/` - CI/CD pipeline definitions
- `Cargo.toml` files - Rust crate configurations

**Documentation**:

- `README.md` files - Usage instructions and examples
- `examples/` - Sample code for each language

### ❌ **Build Artifacts (Gitignored)**

These files are generated during the build process and should not be committed:

**Rust Compilation Outputs**:

- `target/` - Rust compilation cache and binaries
- `target/release/libsuperconfig_ffi.so` - Python extension binary
- `target/release/libsuperconfig_ffi.node` - Node.js addon binary

**Language-Specific Build Outputs**:

- `bindings/python/dist/` - Built Python wheels (.whl files)
- `bindings/nodejs/lib/` - Compiled Node.js packages
- `bindings/wasm/pkg/` - wasm-pack generated outputs
- `crates/superconfig-ffi/pkg/` - WebAssembly build artifacts

**Distribution Packages**:

- `dist/` - Final packaged distributions ready for upload
- `*.whl`, `*.tgz` files - Language-specific package formats

**Moon Metadata**:

- `.moon/` - Moon workspace cache and metadata

## Directory Purposes

### `/crates/superffi/` ✅ COMPLETED

**Purpose**: Reusable procedural macro for multi-language FFI generation\
**Status**: Phase 1 complete - fully implemented\
**Contents**:

- Macro that generates PyO3/NAPI/wasm-bindgen bindings
- Feature flag system (python, nodejs, wasm, all)
- Comprehensive documentation and examples

### `/crates/superconfig-ffi/` 🔄 PHASE 2

**Purpose**: FFI-compatible wrapper around core SuperConfig\
**Status**: Ready for implementation\
**Contents**:

- JSON-based parameter interface for complex types
- SuperFFI macro annotations for binding generation
- Feature flags matching target languages

### `/bindings/python/` ⏳ PHASE 4

**Purpose**: Python package configuration and distribution\
**Distribution**: PyPI as "superconfig"\
**Build Tool**: maturin (Rust → Python wheel)\
**Contents**:

- `setup.py` / `pyproject.toml` - Modern Python packaging
- `superconfig/__init__.py` - Python module entry point

### `/bindings/nodejs/` ⏳ PHASE 4

**Purpose**: Node.js package configuration and distribution\
**Distribution**: npm as "superconfig"\
**Build Tool**: @napi-rs/cli (Rust → Node.js addon)\
**Contents**:

- `package.json` - npm package configuration
- `index.js` - JavaScript entry point and wrapper

### `/bindings/wasm/` ⏳ PHASE 4

**Purpose**: WebAssembly package configuration and distribution\
**Distribution**: npm as "superconfig-wasm"\
**Build Tool**: wasm-pack (Rust → WebAssembly + JS)\
**Contents**:

- `package.json` - WebAssembly package configuration
- `webpack.config.js` - Bundling for browser/Node.js targets
- TypeScript type definitions

## File Creation Order

### Phase 2: SuperConfig FFI Wrapper

1. `crates/superconfig-ffi/Cargo.toml` - Dependencies and feature flags
2. `crates/superconfig-ffi/src/lib.rs` - Core wrapper implementation
3. `crates/superconfig-ffi/moon.yml` - Build task configuration

### Phase 3: Complex Type Handling

1. Extend `crates/superconfig-ffi/src/lib.rs` - JSON parameter conversion
2. Add JSON schema validation utilities
3. Implement Figment method exposure

### Phase 4: Build & Publishing Integration

1. `bindings/python/setup.py` - Python package configuration
2. `bindings/nodejs/package.json` - Node.js package configuration
3. `bindings/wasm/package.json` - WebAssembly package configuration
4. `moon.yml` files for each bindings directory
5. `.github/workflows/release.yml` - CI/CD pipeline

## Development Workflow

### Local Development

```bash
# Build Rust components
moon run superconfig-ffi:build-python
moon run superconfig-ffi:build-nodejs
moon run superconfig-ffi:build-wasm

# Package for distribution  
moon run python:package
moon run nodejs:package
moon run wasm:package

# Test locally
moon run python:test
moon run nodejs:test
```

### CI/CD Pipeline

1. **Trigger**: Git tag push (e.g., `git tag v1.0.0 && git push --tags`)
2. **Build**: Moon builds all targets in dependency order
3. **Package**: Create distribution packages (.whl, .tgz)
4. **Publish**: Upload to PyPI and npm registries
5. **Users Install**: `pip install superconfig` / `npm install superconfig`

## Security Considerations

### File Permissions

- All source files should be readable (644)
- No executable permissions needed except for scripts
- Build outputs should not be committed to version control

### Secrets Management

- PyPI and npm tokens stored as GitHub repository secrets
- No hardcoded credentials in any configuration files
- CI/CD environment variables for sensitive data

---

_See [`build-system.md`](./build-system.md) for Moon task configuration details_
