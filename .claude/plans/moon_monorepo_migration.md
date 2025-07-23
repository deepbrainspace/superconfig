# SuperConfig Moon Monorepo Migration Plan

## Project Overview
Migrate SuperFigment to SuperConfig using Moon-based monorepo architecture. Start simple with a single Rust crate containing all functionality, add multi-language WASM wrappers later.

## Simplified Architecture Goals
- **Single Rust crate**: `superconfig` with feature flags for CLI/MCP/API
- **WASM wrappers**: Separate packages only for other languages
- **Unified build system**: Moon for task orchestration
- **Incremental expansion**: Add language bindings as needed

## Target Structure (Simplified)
```
superconfig/
├── .moon/
│   ├── workspace.yml          # Moon workspace configuration
│   └── tasks.yml              # Global task inheritance
├── .prototools               # Language version pinning (optional)
├── Cargo.toml                # Rust workspace root
├── 
├── crates/
│   └── superconfig/          # Main crate (current code goes here)
│       ├── Cargo.toml        # Features: cli, mcp, api, default = ["core"]
│       ├── src/
│       │   ├── lib.rs        # Core library
│       │   ├── cli/          # CLI implementation (feature = "cli")
│       │   ├── mcp/          # MCP server (feature = "mcp")  
│       │   ├── api/          # HTTP API server (feature = "api")
│       │   └── bin/          # Binary targets
│       └── moon.yml
│
├── wasm/                     # (Future) WASM language bindings
│   ├── js/                   # NPM package wrapper
│   ├── python/               # PyPI package wrapper
│   └── web/                  # Web frontend
│
├── scripts/                  # Build automation
├── docs/                     # Documentation  
└── examples/                 # Usage examples
```

## Phase 1: Foundation Setup

### 1.1 Install Moon and Proto
- [ ] Install Moon CLI: `npm install -g @moonrepo/cli`
- [ ] Install Proto (optional): `curl -fsSL https://moonrepo.dev/install/proto.sh | bash`
- [ ] Verify installations work

### 1.2 Create Git Branch
- [ ] Create feature branch: `git checkout -b feat/moon-monorepo-migration`

### 1.3 Initialize Moon Workspace
- [ ] Run `moon init` to create basic workspace
- [ ] Configure `.moon/workspace.yml` for Rust projects
- [ ] Create `.moon/tasks.yml` for global Rust tasks
- [ ] Set up `.prototools` with Rust version (optional)

## Phase 2: Migrate Current Code to Unified Crate

### 2.1 Create Crate Structure
- [ ] Create `crates/superconfig/` directory
- [ ] Move existing `src/` to `crates/superconfig/src/`
- [ ] Update `Cargo.toml` to support feature flags:
  ```toml
  [package]
  name = "superconfig"
  
  [features]
  default = ["core"]
  core = []
  cli = ["core", "clap"]
  mcp = ["core", "serde", "tokio"] 
  api = ["core", "axum", "tokio"]
  all = ["cli", "mcp", "api"]
  ```

### 2.2 Organize Code by Features
- [ ] Keep current core functionality in `lib.rs`
- [ ] Create `src/cli/mod.rs` for CLI functionality (behind `cli` feature)
- [ ] Create `src/mcp/mod.rs` for MCP server (behind `mcp` feature)
- [ ] Create `src/api/mod.rs` for HTTP API (behind `api` feature)
- [ ] Add binary targets in `src/bin/`:
  ```toml
  [[bin]]
  name = "superconfig"
  required-features = ["cli"]
  
  [[bin]]  
  name = "superconfig-mcp"
  required-features = ["mcp"]
  
  [[bin]]
  name = "superconfig-api" 
  required-features = ["api"]
  ```

### 2.3 Update Root Workspace
- [ ] Convert root `Cargo.toml` to workspace:
  ```toml
  [workspace]
  members = ["crates/superconfig"]
  resolver = "2"
  
  [workspace.dependencies]
  figment = "0.10.19"
  serde = { version = "1.0", features = ["derive"] }
  # ... shared dependencies
  ```

### 2.4 Fix Dependencies
- [ ] Update workspace dependencies to specific versions
- [ ] Test that `cargo build` works from root
- [ ] Test feature flag combinations work

## Phase 3: Moon Configuration

### 3.1 Configure Moon Tasks
- [ ] Create `crates/superconfig/moon.yml`:
  ```yaml
  type: 'lib'
  language: 'rust'
  
  tasks:
    build:
      command: 'cargo build --all-features'
      inputs: ['src/**/*', 'Cargo.toml']
      outputs: ['target/**/*']
    
    test:
      command: 'cargo test --all-features'
      deps: ['build']
    
    clippy:
      command: 'cargo clippy --all-features'
      
    publish:
      command: 'cargo publish'
      deps: ['build', 'test', 'clippy']
  ```

### 3.2 Global Task Configuration
- [ ] Configure `.moon/tasks.yml` for Rust defaults
- [ ] Set up task inheritance
- [ ] Configure caching and inputs/outputs

### 3.3 Test Moon Integration
- [ ] Run `moon run superconfig:build`
- [ ] Run `moon run superconfig:test`
- [ ] Verify caching works on subsequent runs

## Phase 4: Feature Implementation

### 4.1 CLI Feature
- [ ] Implement basic CLI using `clap` crate
- [ ] Commands: `init`, `get`, `set`, `validate`, `server`
- [ ] Test binary builds: `cargo build --features=cli`

### 4.2 MCP Server Feature  
- [ ] Implement MCP protocol handlers
- [ ] Configuration management via MCP
- [ ] Test binary builds: `cargo build --features=mcp`

### 4.3 API Server Feature
- [ ] Implement REST API using `axum`
- [ ] Configuration CRUD operations
- [ ] Test binary builds: `cargo build --features=api`

## Phase 5: Testing and Documentation

### 5.1 Comprehensive Testing
- [ ] Unit tests for core functionality
- [ ] Integration tests for each feature
- [ ] Binary functionality tests
- [ ] Feature flag combination tests

### 5.2 Documentation
- [ ] Update README with new structure
- [ ] Document feature flags usage
- [ ] Create usage examples for each feature
- [ ] Document migration from superfigment

### 5.3 Release Preparation
- [ ] Configure `cargo publish` settings
- [ ] Test publishing to crates.io (dry run)
- [ ] Set up GitHub releases for binaries

## Future Phases (Post-MVP)

### Phase 6: WASM Language Bindings (Later)
- [ ] Create `wasm/js/` for NPM package
- [ ] Create `wasm/python/` for PyPI package  
- [ ] Create `wasm/web/` for web frontend
- [ ] Configure Moon tasks for WASM builds

## Success Criteria
- [ ] Single `superconfig` crate builds with all features
- [ ] `cargo install superconfig --features=all` gives complete toolset
- [ ] CLI, MCP server, and API server binaries work
- [ ] Moon build system manages everything efficiently
- [ ] Ready for crates.io publishing
- [ ] Clean migration path from superfigment

## Key Benefits of This Approach
- **Simple Distribution**: `cargo install superconfig --features=cli,mcp,api`
- **Rust-Native**: Pure Rust ecosystem, no cross-language complexity initially
- **Incremental**: Add WASM bindings later when needed
- **Maintainable**: Single codebase, shared core logic
- **Feature Flags**: Users pick what they need

## Installation Examples
```bash
# Just the library
cargo add superconfig

# CLI tool
cargo install superconfig --features=cli

# Everything  
cargo install superconfig --features=all
cargo add superconfig --features=all

# In Cargo.toml
superconfig = { version = "1.0", features = ["cli", "mcp"] }
```

## Timeline Estimate
- **Phase 1**: 1 day (setup)
- **Phase 2**: 2-3 days (code migration)  
- **Phase 3**: 1 day (Moon config)
- **Phase 4**: 3-4 days (feature implementation)
- **Phase 5**: 2-3 days (testing + docs)

**Total: 9-12 days**

## Next Steps
1. Create the git branch
2. Install Moon and set up workspace
3. Move current code to `crates/superconfig/`
4. Set up feature flags and test builds
5. Implement CLI/MCP/API features incrementally