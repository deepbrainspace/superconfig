# ErrorRegistry Architecture Design & Implementation Plan

## Overview

Design and implement a standalone `error-registry` crate that provides generic, high-performance error collection and querying capabilities. This will serve as the foundation for SuperConfig's unified error handling system.

## Architecture Goals

### Core Principles

- **Generic & Reusable**: Works for any error types, not SuperConfig-specific
- **High Performance**: Lock-free operations using DashMap, optimized for concurrent access
- **Type Safety**: Full Rust generics support with compile-time type checking
- **FFI Compatible**: String-based API for Python/Node.js while maintaining type safety
- **Builder Pattern**: Flexible, chainable query interface
- **Zero Copy**: Arc-based sharing for efficient memory usage

### Performance Targets

- Error storage: <100ns per error
- Type-filtered queries: <1μs for <1000 errors
- Concurrent access: Lock-free scaling across threads
- Memory overhead: <50 bytes per error entry

## Detailed Design

### 1. Core ErrorRegistry Structure

```rust
pub struct ErrorRegistry {
    /// Storage for errors by ID (lock-free concurrent access)
    errors: DashMap<u64, ErrorEntry>,
    /// Next error ID (atomic counter)
    next_id: AtomicU64,
    /// Ordered list of error IDs for FIFO/LIFO access
    error_order: Arc<RwLock<Vec<u64>>>,
    /// Type name to TypeId mapping for FFI (concurrent reads)
    #[cfg(feature = "ffi")]
    type_registry: DashMap<String, std::any::TypeId>,
}
```

**Key Decision: DashMap for type_registry**

- Reason: Optimizes for concurrent reads (multiple FFI threads)
- Performance: No lock contention on type lookups
- Use case: Write-once at startup, read-many during runtime

### 2. ErrorEntry Structure

```rust
struct ErrorEntry {
    /// The actual error data (any type)
    data: Box<dyn std::any::Any + Send + Sync>,
    /// Type name for runtime type checking
    type_name: &'static str,
    /// When this error occurred
    timestamp: u64,
    /// Human-readable message
    message: String,
    /// Error code for programmatic handling
    code: u32,
}
```

### 3. Builder Pattern API

```rust
// Rust usage (type-safe)
registry.errors
    .query()
    .of_type::<FlagError>()
    .newest_first()
    .limit(5)
    .show(); // or .extract()

// FFI usage (string-based)
registry.errors
    .query()
    .of_type_named("FlagError")
    .newest_first()
    .limit(5)
    .show_json(); // or .extract_json()
```

### 4. SuperConfig Integration

```rust
pub struct ConfigRegistry {
    pub errors: ErrorRegistry,  // Direct field access
    // ... other fields
}

// Usage
registry.errors.store(FlagError { ... }, "message", 1001);
registry.errors.has::<FlagError>();
registry.errors.extract::<ValidationError>();
```

## Complete Function List

### **Core ErrorRegistry Methods**

1. `ErrorRegistry::new() -> Self` - Create new registry instance
2. `store<T>(&self, error_data: T, message: String, code: u32)` - Store typed error
3. `register_type<T>(&self, name: &str)` - Register type for FFI (feature-gated)
4. `query(&self) -> ErrorQueryBuilder` - Start building query (main entry point)

### **Convenience Shortcuts**

5. `show<T>(&self) -> Vec<(Arc<T>, String, u32, u64)>` - Quick peek at errors
6. `extract<T>(&self) -> Vec<(Arc<T>, String, u32, u64)>` - Quick removal
7. `has<T>(&self) -> bool` - Check if errors exist
8. `count<T>(&self) -> usize` - Count errors of type

### **ErrorQueryBuilder Methods**

9. `of_type<T>(self) -> Self` - Filter by Rust type (type-safe)
10. `of_type_named(self, name: &str) -> Self` - Filter by string (FFI)
11. `newest_first(self) -> Self` - LIFO ordering
12. `oldest_first(self) -> Self` - FIFO ordering (default)
13. `limit(self, count: usize) -> Self` - Limit results
14. `count(&self) -> usize` - Count without retrieving
15. `show<T>(&self) -> Vec<(Arc<T>, String, u32, u64)>` - Peek at results
16. `extract<T>(&self) -> Vec<(Arc<T>, String, u32, u64)>` - Get and remove
17. `show_json(&self) -> String` - FFI peek (feature-gated)
18. `extract_json(&self) -> String` - FFI extraction (feature-gated)

**Total: 18 functions across 3 structs**

## Implementation Plan

### Phase 1: Core ErrorRegistry (2-3 hours)

1. **Create error-registry crate structure**
   - Set up Cargo.toml with features: `default = []`, `ffi = ["serde", "serde_json"]`
   - Project structure: `lib.rs`, `builder.rs`, `entry.rs`, `ffi.rs`

2. **Implement ErrorEntry and core storage**
   - ErrorEntry struct with `Box<dyn Any + Send + Sync>` storage
   - DashMap-based error storage with atomic ID generation
   - Basic `store()` method with type validation

3. **Add type registration system**
   - DashMap for type name → TypeId mapping (concurrent read optimization)
   - `register_type()` method for FFI support
   - Type safety validation helpers

### Phase 2: Query Builder (2-3 hours)

1. **Design ErrorQueryBuilder structure**
   - Builder pattern with method chaining
   - Type filters (generic and string-based)
   - Ordering options (FIFO/LIFO)
   - Limit support

2. **Implement core query methods**
   - count() - non-destructive counting
   - show() - peek without removing
   - extract() - remove and return
   - Type filtering logic

3. **Add FFI query methods**
   - show_json() and extract_json()
   - JSON serialization of results
   - String-based type filtering

### Phase 3: Performance & Testing (1-2 hours)

1. **Performance optimizations**
   - Benchmark core operations
   - Optimize hot paths
   - Memory usage validation

2. **Comprehensive testing**
   - Unit tests for all functionality
   - Concurrent access stress tests
   - FFI compatibility tests
   - Memory leak detection

### Phase 4: SuperConfig Integration (1-2 hours)

1. **Define SuperConfig error types**
   - FlagError, ValidationError, HandleError
   - Serde serialization support
   - Error code constants

2. **Integrate into ConfigRegistry**
   - Add ErrorRegistry field
   - Register error types
   - Update enable/disable methods to use new system

3. **Update FFI layers**
   - Python bindings using new error system
   - Node.js bindings using new error system
   - Test cross-language compatibility

## Error Types for SuperConfig

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct FlagError {
    pub invalid_flags: u64,
    pub operation: String,
    pub flag_type: String, // "runtime" or "startup"
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub field: String,
    pub value: String,
    pub constraint: String,
    pub valid_range: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HandleError {
    pub handle_id: u64,
    pub expected_type: String,
    pub found_type: String,
}
```

## API Examples

### Rust Client (Type-Safe)

```rust
let registry = ConfigRegistry::arc_new()
    .enable(invalid_flags)
    .set_verbosity(99);

// Builder pattern queries
let recent_flag_errors = registry.errors
    .query()
    .of_type::<FlagError>()
    .newest_first()
    .limit(3)
    .show();

// Convenience methods
let has_validation_errors = registry.errors.has::<ValidationError>();
let all_handle_errors = registry.errors.extract::<HandleError>();

// Count by type
let error_count = registry.errors.query().of_type::<FlagError>().count();
```

### Python Client (String-Based)

```python
registry = superconfig.ConfigRegistry()
registry.enable(invalid_flags)
registry.set_verbosity(99)

# Same builder pattern
recent_errors = registry.errors.query().of_type_named("FlagError").newest_first().limit(3).show_json()
error_count = registry.errors.query().of_type_named("ValidationError").count()

# Parse JSON results
import json
errors = json.loads(recent_errors)
for error in errors:
    print(f"Flag error: {error['message']} - flags: {error['data']['invalid_flags']}")
```

### Node.js Client (String-Based)

```javascript
const registry = new superconfig.ConfigRegistry();
registry.enable(invalidFlags);
registry.setVerbosity(99);

// Same builder pattern
const recentErrors = registry.errors.query().ofTypeNamed("FlagError").newestFirst().limit(3).showJson();
const errorCount = registry.errors.query().ofTypeNamed("ValidationError").count();

// Parse JSON results
const errors = JSON.parse(recentErrors);
errors.forEach(error => {
    console.log(`Flag error: ${error.message} - flags: 0x${error.data.invalid_flags.toString(16)}`);
});
```

## Dependencies

### error-registry/Cargo.toml

```toml
[package]
name = "error-registry"
version = "0.1.0"
edition = "2021"

[features]
default = []
ffi = ["serde", "serde_json"]

[dependencies]
dashmap = "7.0.0-rc2"
parking_lot = "0.12.4"
serde = { version = "1.0.219", features = ["derive"], optional = true }
serde_json = { version = "1.0.141", optional = true }

[dev-dependencies]
criterion = "0.7.0"
tempfile = "3.15.0"
```

## Success Criteria

1. **Performance Benchmarks**
   - Error storage: <100ns per operation
   - Type queries: <1μs for reasonable error counts
   - Memory usage: <50 bytes overhead per error

2. **Functionality Tests**
   - All query builder combinations work correctly
   - Type safety maintained in Rust
   - FFI compatibility across Python/Node.js
   - Concurrent access without data races

3. **Integration Success**
   - SuperConfig uses new error system
   - All existing functionality preserved
   - FFI bindings updated and working
   - No breaking changes to public API

## Next Steps After Completion

1. **Update SuperConfig methods** to use new error collection
2. **Implement remaining fluent API patterns** with error handling
3. **Performance benchmark** against current system
4. **Documentation and examples** for new error handling patterns

## Timeline

- **Total Estimate**: 6-10 hours
- **Phase 1**: 2-3 hours (Core ErrorRegistry)
- **Phase 2**: 2-3 hours (Query Builder)
- **Phase 3**: 1-2 hours (Performance & Testing)
- **Phase 4**: 1-2 hours (SuperConfig Integration)

This architecture provides a solid foundation for high-performance, type-safe error handling that works seamlessly across Rust and FFI boundaries while maintaining excellent performance characteristics.

---

## Session Restart Prompt

**Use this prompt when restarting the session:**

```
I need you to implement the ErrorRegistry crate based on the architecture document at `/home/nsm/code/deepbrain/superconfig/.claude/plans/superconfig-v2-rewrite/20-error-registry-architecture.md`. 

Key requirements:
- Standalone `error-registry` crate with 18 functions across 3 structs
- Use DashMap for both error storage AND type registry (concurrent optimization)
- Builder pattern API: `registry.errors.query().of_type::<T>().show()` / `.extract()`
- FFI support with string-based types and JSON serialization
- Performance targets: <100ns error storage, <1μs queries

Start with Phase 1: Core ErrorRegistry implementation. Create the crate structure and implement the basic storage system with DashMap.

Read the full document first, then begin implementation.
```

**Context Files to Reference:**

- Primary: `/home/nsm/code/deepbrain/superconfig/.claude/plans/superconfig-v2-rewrite/20-error-registry-architecture.md`
- Existing codebase: `/home/nsm/code/deepbrain/superconfig/crates/superconfig/`
- Work log: `/home/nsm/code/deepbrain/superconfig/.claude/plans/superconfig-v2-rewrite/work-log.md`
