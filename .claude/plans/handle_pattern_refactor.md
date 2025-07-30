# SuperConfig Handle Pattern Refactor Plan

## Overview

This document outlines the plan for refactoring SuperConfig from the current clone-based FFI approach to a high-performance handle-based architecture while maintaining full API compatibility.

## Current State (Clone-Based)

### Architecture

- **Core Struct**: `SuperConfig` contains Figment + metadata
- **FFI Methods**: Take `&self`, clone inner state for operations
- **Performance**: ~2x overhead for FFI calls due to cloning
- **Memory**: Higher memory usage due to frequent cloning

### Performance Characteristics

```
Native Rust: config.with_file("config.toml")           // Direct mutation
FFI Clone:    config.with_file("config.toml").clone()  // Clone overhead
```

## Target State (Handle-Based)

### Architecture

- **Handle System**: Each SuperConfig instance gets unique handle ID
- **Global Registry**: Thread-safe handle → config mapping
- **FFI Methods**: Operations via handle lookups, no cloning
- **Memory**: Single source of truth per configuration

### Performance Goals

```
Native Rust: config.with_file("config.toml")     // Direct mutation  
FFI Handle:  handle_registry.get(id).with_file() // Registry lookup only
```

## Implementation Phases

### Phase 1: Preparation (Current - Clone-based shipping)

- ✅ Ship clone-based FFI for immediate market feedback
- ✅ Establish API patterns and user expectations
- ✅ Validate FFI wrapper across Python, Node.js, WASM

### Phase 2: Handle Infrastructure

**Timeline**: 2-3 weeks after Phase 1 user feedback

#### 2.1 Handle Registry Design

```rust
// Global registry for handle management
pub struct ConfigRegistry {
    configs: DashMap<HandleId, SuperConfig>,
    next_id: AtomicU64,
}

// Handle wrapper for FFI
#[derive(Copy, Clone)]
pub struct ConfigHandle {
    id: HandleId,
}

// Registry operations
impl ConfigRegistry {
    fn insert(&self, config: SuperConfig) -> HandleId;
    fn get(&self, id: HandleId) -> Option<DashRef<HandleId, SuperConfig>>;
    fn remove(&self, id: HandleId) -> Option<SuperConfig>;
    fn update<F>(&self, id: HandleId, f: F) -> Result<(), Error>
        where F: FnOnce(&mut SuperConfig);
}
```

#### 2.2 Handle-Based SuperConfig API

```rust
impl SuperConfig {
    // Create new handle-managed instance
    pub fn new_handled() -> ConfigHandle;
    
    // Convert existing instance to handle
    pub fn into_handle(self) -> ConfigHandle;
    
    // Extract from handle (consumes handle)
    pub fn from_handle(handle: ConfigHandle) -> Result<Self, Error>;
}

impl ConfigHandle {
    // All existing SuperConfig methods via handle operations
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self;
    pub fn with_env<S: AsRef<str>>(self, prefix: S) -> Self;
    pub fn extract<T: for<'de> serde::Deserialize<'de>>(self) -> Result<T, Error>;
    
    // Handle-specific operations
    pub fn clone_config(&self) -> ConfigHandle; // Creates new handle with cloned config
    pub fn is_valid(&self) -> bool;             // Check if handle still exists
}
```

#### 2.3 Memory Management

```rust
// RAII pattern for automatic cleanup
impl Drop for ConfigHandle {
    fn drop(&mut self) {
        GLOBAL_REGISTRY.remove(self.id);
    }
}

// Reference counting for shared handles
pub struct SharedConfigHandle {
    handle: ConfigHandle,
    ref_count: Arc<AtomicUsize>,
}
```

### Phase 3: API Transition

**Timeline**: Concurrent with Phase 2

#### 3.1 Backwards Compatibility

```rust
// Legacy clone-based methods (deprecated but functional)
impl SuperConfig {
    #[deprecated(note = "Use handle-based methods for better performance")]
    pub fn with_file_clone(&self, path: &str) -> Self {
        // Fallback to clone behavior
    }
}

// New handle-based methods (primary API)
impl SuperConfig {
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
        // Direct mutation for owned instances
    }
    
    pub fn with_file_ref<P: AsRef<Path>>(&self, path: P) -> ConfigHandle {
        // Handle-based for borrowed instances
    }
}
```

#### 3.2 FFI Wrapper Migration

```rust
// Phase 2: Clone-based (current)
#[superffi]
impl SuperConfigFFI {
    pub fn with_file(&self, path: &str) -> Self {
        SuperConfigFFI { inner: self.inner.clone().with_file(path) }
    }
}

// Phase 3: Handle-based (target)
#[superffi]
impl SuperConfigFFI {
    pub fn with_file(&self, path: &str) -> Self {
        let handle = self.handle.with_file_ref(path);
        SuperConfigFFI { handle }
    }
}
```

### Phase 4: Performance Optimization

**Timeline**: 1-2 weeks after Phase 3

#### 4.1 Registry Optimizations

- **Lock-free Operations**: Use atomic operations where possible
- **Memory Pooling**: Reuse handle IDs and reduce allocations
- **Lazy Evaluation**: Defer expensive operations until extract()

#### 4.2 Figment Integration Analysis

Based on our analysis of Figment source code:

**Core Dependencies** (must handle in registry):

```rust
// These require special handling due to Figment's data model
pub struct SuperConfigCore {
    providers: Vec<Box<dyn Provider>>,  // Can't easily clone
    profile: Profile,                   // Simple to handle
    metadata: Vec<Metadata>,            // Contains Arc<dyn Display>
}
```

**Solutions**:

- **Provider Handling**: Registry stores provider configs, reconstructs Figments on demand
- **Metadata Tracking**: Handle metadata separately from core config data
- **Extraction Caching**: Cache common extraction patterns

#### 4.3 Benchmarking Framework

```rust
// Performance test suite
mod benchmarks {
    fn bench_clone_vs_handle_simple();
    fn bench_clone_vs_handle_complex();
    fn bench_provider_intensive_ops();
    fn bench_memory_usage_patterns();
}
```

## Migration Strategy

### For Rust Users

- **No Breaking Changes**: All existing APIs continue to work
- **Performance Opt-in**: New handle-based methods for performance-sensitive code
- **Gradual Migration**: Deprecation warnings guide users to better patterns

### For FFI Users

- **Transparent Upgrade**: FFI wrapper performance improves automatically
- **API Stability**: No changes to Python/Node.js/WASM interfaces
- **Backwards Compatibility**: Existing FFI code continues working

### For SuperConfig Core

- **Incremental Rollout**: Handle system deployed alongside existing clone system
- **Feature Flags**: Runtime switching between clone/handle modes for testing
- **Rollback Plan**: Ability to disable handle system if issues arise

## Risk Assessment

### Technical Risks

- **Thread Safety**: Handle registry must be completely thread-safe
- **Memory Leaks**: Proper handle cleanup under all failure scenarios
- **Performance Regression**: Registry overhead must not exceed clone savings

### Mitigation Strategies

- **Extensive Testing**: Multi-threaded stress tests before deployment
- **RAII Patterns**: Automatic resource cleanup via Drop traits
- **Benchmark Gates**: Performance tests must pass before merge

### Success Metrics

- **Performance**: ≥50% reduction in FFI operation overhead
- **Memory**: ≥30% reduction in peak memory usage for FFI workloads
- **Compatibility**: 100% existing test suite passes
- **Adoption**: ≥80% of FFI users see performance improvements

## Timeline Summary

| Phase             | Duration  | Key Deliverables         |
| ----------------- | --------- | ------------------------ |
| Phase 1 (Current) | Completed | Clone-based FFI shipping |
| Phase 2           | 2-3 weeks | Handle infrastructure    |
| Phase 3           | 2 weeks   | API transition layer     |
| Phase 4           | 1-2 weeks | Performance optimization |

**Total Timeline**: 5-7 weeks for complete handle pattern implementation

## Future Enhancements

### Advanced Handle Features

- **Handle Sharing**: Multiple FFI instances sharing same underlying config
- **Config Snapshots**: Point-in-time config captures for debugging
- **Transaction Support**: Batched config operations with rollback

### Platform-Specific Optimizations

- **WASM**: Memory-mapped handle tables for maximum efficiency
- **Python**: GIL-aware handle operations
- **Node.js**: V8 optimizer-friendly handle patterns

This refactor will position SuperConfig as the highest-performance configuration library across all target languages while maintaining the clean, familiar API that users expect.
