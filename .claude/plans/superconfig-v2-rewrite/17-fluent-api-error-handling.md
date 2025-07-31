# Fluent API Error Handling System

**Status**: 🔄 DESIGN FINALIZED - READY FOR IMPLEMENTATION\
**Priority**: HIGH\
**Dependencies**: Core Registry Refactoring Complete ✅\
**Estimated Duration**: 6-8 hours

## Overview

Design and implement a comprehensive error handling system for SuperConfig's fluent API that supports three distinct patterns: fail-fast, permissive error collection, and error inspection. The system must work seamlessly across Rust native usage and FFI boundaries (Python, Node.js, WASM).

## 🎯 Final Design

### **Three Error Handling Patterns**

```rust
// Pattern 1: Fail-fast (traditional Result pattern)
let registry = ConfigRegistry::new()
    .enable(flags)?
    .disable(other_flags)?;

// Pattern 2: Permissive with error collection
let registry = ConfigRegistry::new()
    .try_enable(flags)
    .try_disable(other_flags);
let errors = registry.flush_errors(); // Get and clear errors

// Pattern 3: Error inspection without consumption
let errors = registry.errors(); // Just peek, don't clear
```

### **Error Structure (Rust + FFI Compatible)**

```rust
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluentError {
    /// Operation that failed (e.g., "enable", "disable", "set_verbosity")
    pub operation: String,
    /// When the error occurred (for debugging and ordering)
    pub timestamp: Instant,
    /// The actual registry error that occurred
    pub error: RegistryError,
    /// Optional debugging context (parameter values, etc.)
    pub context: Option<String>,
}

/// Enhanced ConfigRegistry with error collection
pub struct ConfigRegistry {
    // ... existing fields ...
    /// Collected errors from try_* methods for permissive error handling
    collected_errors: Arc<RwLock<Vec<FluentError>>>,
}
```

## 🔧 Implementation Strategy

### **Phase 1: Core Error Collection Infrastructure (2 hours)**

#### Task 1.1: Add Error Collection to Registry

```rust
impl ConfigRegistry {
    /// Add error collection field to new() and custom() constructors
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            collected_errors: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Internal helper to collect errors from try_* methods
    fn collect_error(&self, operation: &str, error: RegistryError, context: Option<String>) {
        let fluent_error = FluentError {
            operation: operation.to_string(),
            timestamp: Instant::now(),
            error,
            context,
        };
        
        let mut errors = self.collected_errors.write();
        errors.push(fluent_error);
    }
}
```

#### Task 1.2: Error Management Methods

```rust
impl ConfigRegistry {
    /// Get all collected errors and clear internal storage
    pub fn flush_errors(&self) -> Vec<FluentError> {
        let mut errors = self.collected_errors.write();
        std::mem::take(&mut *errors) // Move out, leave empty vec
    }
    
    /// Peek at errors without clearing (for inspection)
    pub fn errors(&self) -> Vec<FluentError> {
        self.collected_errors.read().clone()
    }
    
    /// Check if any errors have been collected
    pub fn has_errors(&self) -> bool {
        !self.collected_errors.read().is_empty()
    }
    
    /// Clear all collected errors without returning them
    pub fn clear_errors(&self) {
        let mut errors = self.collected_errors.write();
        errors.clear();
    }
}
```

### **Phase 2: Macro System Design (2-3 hours)**

#### Macro 1: `#[generate_try_method]` - Try Variant Generator

**Purpose**: Automatically generate `try_*` methods from existing `Result`-returning methods

```rust
// Procedural macro implementation
#[proc_macro_attribute]
pub fn generate_try_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let try_fn_name = format_ident!("try_{}", fn_name);
    
    // Extract parameters and return type
    let params = &input_fn.sig.inputs;
    let return_type = match &input_fn.sig.output {
        ReturnType::Default => quote! { &Self },
        ReturnType::Type(_, ty) => {
            // Handle Result<&Self, E> -> &Self
            // Handle Result<(), E> -> &Self  
            quote! { &Self }
        }
    };
    
    let param_names: Vec<_> = params.iter()
        .filter_map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let Pat::Ident(ident) = &**pat_type.pat {
                    Some(&ident.ident)
                } else { None }
            } else { None }
        })
        .collect();
    
    // Generate context string from parameters
    let context_gen = quote! {
        let context = Some(format!("{}({})", 
            stringify!(#fn_name),
            // TODO: Format parameter values for debugging
        ));
    };
    
    let expanded = quote! {
        // Original method (unchanged)
        #input_fn
        
        // Generated try_* method
        pub fn #try_fn_name(#params) -> #return_type {
            match self.#fn_name(#(#param_names),*) {
                Ok(_) => self,
                Err(e) => {
                    #context_gen
                    self.collect_error(stringify!(#fn_name), e, context);
                    self
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

**Usage**:

```rust
#[generate_try_method]
pub fn enable(&self, flags: u64) -> Result<&Self, RegistryError> { ... }

#[generate_try_method] 
pub fn disable(&self, flags: u64) -> Result<&Self, RegistryError> { ... }

#[generate_try_method]
pub fn set_verbosity(&self, level: u8) -> Result<(), RegistryError> { ... }
```

**Generated Methods**:

- `try_enable(&self, flags: u64) -> &Self`
- `try_disable(&self, flags: u64) -> &Self`
- `try_set_verbosity(&self, level: u8) -> &Self`

#### Macro 2: `#[generate_json_helper]` - FFI JSON Helper Generator

**Purpose**: Automatically generate `_as_json` methods for FFI compatibility

```rust
#[proc_macro_attribute]
pub fn generate_json_helper(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let json_fn_name = format_ident!("{}_as_json", fn_name);
    
    let params = &input_fn.sig.inputs;
    let param_names: Vec<_> = /* extract parameter names */;
    
    let expanded = quote! {
        // Original method (unchanged)
        #input_fn
        
        // Generated JSON helper for FFI
        #[allow(dead_code)] // Used by FFI crates in Phase 4
        pub(crate) fn #json_fn_name(#params) -> String {
            match self.#fn_name(#(#param_names),*) {
                Ok(_) => serde_json::to_string(&serde_json::json!({
                    "success": true
                })).unwrap(),
                Err(e) => serde_json::to_string(&serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })).unwrap(),
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

**Usage**:

```rust
#[generate_try_method]
#[generate_json_helper]
pub fn enable(&self, flags: u64) -> Result<&Self, RegistryError> { ... }
```

**Generated Methods**:

- `try_enable(&self, flags: u64) -> &Self` (from first macro)
- `enable_as_json(&self, flags: u64) -> String` (from second macro)

#### Macro Interaction Strategy

**Macro Execution Order**:

1. `#[generate_json_helper]` processes first (outer attribute)
2. `#[generate_try_method]` processes second (inner attribute)
3. Both macros preserve the original method

**Attribute Stacking**:

```rust
#[generate_json_helper]
#[generate_try_method]
pub fn enable(&self, flags: u64) -> Result<&Self, RegistryError> {
    // Original implementation
}

// Results in 3 methods:
// 1. enable() - original 
// 2. try_enable() - from generate_try_method
// 3. enable_as_json() - from generate_json_helper
```

### **Phase 3: FFI Integration (1-2 hours)**

#### Task 3.1: SuperConfig FFI Wrapper Methods

```rust
// In superconfig-ffi crate
#[superffi]
impl SuperConfig {
    // Pattern 1: Strict methods (raise exceptions in FFI)
    pub fn enable(&self, flags: u64) -> Result<Self, String> {
        self.inner.enable(flags)
            .map(|_| Self { inner: self.inner.clone() })
            .map_err(|e| e.to_string())
    }
    
    // Pattern 2: Permissive methods (never fail in FFI)
    pub fn try_enable(&self, flags: u64) -> Self {
        self.inner.try_enable(flags);
        Self { inner: self.inner.clone() }
    }
    
    // Error management methods
    pub fn flush_errors(&self) -> Vec<FluentError> {
        self.inner.flush_errors()
    }
    
    pub fn errors(&self) -> Vec<FluentError> {
        self.inner.errors()
    }
    
    pub fn has_errors(&self) -> bool {
        self.inner.has_errors()
    }
}
```

#### Task 3.2: Language-Specific Generated APIs

**Python**:

```python
# Strict pattern - raises exceptions
try:
    registry = registry.enable(flags).disable(other_flags) 
except SuperConfigError as e:
    print(f"Failed: {e}")

# Permissive pattern - collects errors
registry = registry.try_enable(flags).try_disable(other_flags)
errors = registry.flush_errors()  # List[FluentError] as Python objects
for error in errors:
    print(f"Warning [{error.operation}]: {error.error}")
```

**Node.js**:

```javascript
// Strict pattern - Promise rejection
try {
  const registry = await registry.enable(flags).disable(otherFlags);
} catch (error) {
  console.log(`Failed: ${error.message}`);
}

// Permissive pattern - error collection
const registry = registry.tryEnable(flags).tryDisable(otherFlags);
const errors = registry.flushErrors(); // Array of error objects
errors.forEach(error => {
  console.log(`Warning [${error.operation}]: ${error.error}`);
});
```

**WASM/Web**:

```javascript
// Same as Node.js - consistent web experience
const registry = new SuperConfig()
  .tryEnable(999)  // Invalid flag
  .tryEnable(1);   // Valid flag

const errors = registry.flushErrors();
if (errors.length > 0) {
  console.warn('Configuration warnings:', errors);
}
```

### **Phase 4: Testing & Validation (2 hours)**

#### Task 4.1: Unit Tests

```rust
#[cfg(test)]
mod fluent_error_tests {
    use super::*;
    use crate::config_flags::{runtime, startup};
    
    #[test]
    fn test_strict_pattern_fail_fast() {
        let registry = ConfigRegistry::new();
        let result = registry.enable(999); // Invalid flag
        assert!(result.is_err());
    }
    
    #[test]
    fn test_permissive_pattern_error_collection() {
        let registry = ConfigRegistry::new();
        let registry = registry
            .try_enable(999)     // Invalid - should collect error
            .try_enable(runtime::STRICT_MODE)  // Valid - should succeed
            .try_disable(888);   // Invalid - should collect error
            
        let errors = registry.flush_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].operation, "enable");
        assert_eq!(errors[1].operation, "disable");
        
        // Errors should be cleared after flush
        assert_eq!(registry.errors().len(), 0);
    }
    
    #[test]
    fn test_error_inspection_without_clearing() {
        let registry = ConfigRegistry::new();
        registry.try_enable(999); // Invalid flag
        
        let errors_peek1 = registry.errors();
        let errors_peek2 = registry.errors();
        assert_eq!(errors_peek1.len(), 1);
        assert_eq!(errors_peek2.len(), 1); // Should be same
        
        let errors_flush = registry.flush_errors();
        assert_eq!(errors_flush.len(), 1);
        assert_eq!(registry.errors().len(), 0); // Now cleared
    }
    
    #[test]
    fn test_mixed_pattern_usage() {
        let registry = ConfigRegistry::new();
        
        // Start with permissive
        let registry = registry
            .try_enable(runtime::STRICT_MODE)  // Valid
            .try_enable(999);                  // Invalid - collected
            
        // Check for critical errors before continuing
        let errors = registry.errors();
        let has_critical = errors.iter().any(|e| e.error.to_string().contains("999"));
        
        if !has_critical {
            // Switch to strict for critical operations
            let result = registry.enable(runtime::PARALLEL);
            assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_error_context_information() {
        let registry = ConfigRegistry::new();
        registry.try_enable(999);
        
        let errors = registry.flush_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].operation, "enable");
        assert!(errors[0].context.is_some());
        // Context should contain parameter information for debugging
    }
}
```

#### Task 4.2: FFI Integration Tests

```rust
#[cfg(test)]
mod ffi_error_tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_json_serialization() {
        let registry = ConfigRegistry::new();
        registry.try_enable(999);
        
        let json_str = registry.flush_errors_as_json();
        let errors: Vec<FluentError> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].operation, "enable");
    }
    
    #[test]
    fn test_try_methods_as_json() {
        let registry = ConfigRegistry::new();
        let result_json = registry.try_enable_as_json(999);
        
        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        assert_eq!(result["success"], true); // try_* always succeeds
        
        // But error should be collected
        let errors = registry.flush_errors();
        assert_eq!(errors.len(), 1);
    }
}
```

#### Task 4.3: Performance Benchmarks

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn benchmark_error_collection_overhead() {
        let registry = ConfigRegistry::new();
        
        // Baseline: successful operations
        let start = Instant::now();
        for _ in 0..10000 {
            registry.try_enable(runtime::STRICT_MODE);
        }
        let success_time = start.elapsed();
        
        // With errors: failing operations  
        let start = Instant::now();
        for _ in 0..10000 {
            registry.try_enable(999); // Invalid flag
        }
        let error_time = start.elapsed();
        
        println!("Success operations: {:?}", success_time);
        println!("Error operations: {:?}", error_time);
        
        // Error collection should not add significant overhead
        let overhead_ratio = error_time.as_nanos() as f64 / success_time.as_nanos() as f64;
        assert!(overhead_ratio < 2.0, "Error collection overhead too high: {}x", overhead_ratio);
    }
    
    #[test] 
    fn benchmark_memory_usage() {
        let registry = ConfigRegistry::new();
        
        // Collect many errors
        for i in 0..1000 {
            registry.try_enable(999 + i); // Different invalid flags
        }
        
        let errors = registry.errors();
        assert_eq!(errors.len(), 1000);
        
        // Memory usage should be reasonable
        let estimated_size = std::mem::size_of::<FluentError>() * errors.len();
        println!("Estimated memory usage for 1000 errors: {} bytes", estimated_size);
        
        // Flush should clear memory
        registry.flush_errors();
        assert_eq!(registry.errors().len(), 0);
    }
}
```

## 📋 Implementation Order & Timeline

### **Week 1: Core Infrastructure**

**Day 1-2: Phase 1 - Core Error Collection (2 hours)**

- [ ] Add `collected_errors` field to ConfigRegistry
- [ ] Implement error management methods (`flush_errors`, `errors`, `has_errors`)
- [ ] Add basic unit tests for error collection
- [ ] Verify no compilation issues with existing code

**Day 3: Phase 2.1 - Try Method Macro (2 hours)**

- [ ] Create `#[generate_try_method]` procedural macro
- [ ] Test macro with simple methods (`enable`, `disable`)
- [ ] Verify generated code compiles and functions correctly
- [ ] Add macro-generated method tests

### **Week 2: Macro System & FFI**

**Day 4: Phase 2.2 - JSON Helper Macro (1 hour)**

- [ ] Create `#[generate_json_helper]` procedural macro
- [ ] Test macro interaction with try method macro
- [ ] Verify both macros work together on same method
- [ ] Add FFI JSON serialization tests

**Day 5-6: Phase 3 - FFI Integration (2 hours)**

- [ ] Update superconfig-ffi crate with both patterns
- [ ] Test Python FFI with strict and permissive patterns
- [ ] Test Node.js FFI with error collection
- [ ] Verify WASM builds work with new error handling

**Day 7: Phase 4 - Testing & Validation (2 hours)**

- [ ] Comprehensive unit test suite
- [ ] Performance benchmarks for error collection overhead
- [ ] Memory usage tests for error accumulation
- [ ] Integration tests across all FFI targets

### **Week 3: Polish & Documentation**

**Day 8: Documentation & Examples**

- [ ] Update API documentation with error handling patterns
- [ ] Create usage examples for all three patterns
- [ ] Add FFI-specific documentation for each language
- [ ] Update Phase 2 plan with completed error handling

**Day 9: Final Testing & Validation**

- [ ] End-to-end testing with real configuration files
- [ ] Performance validation (no significant overhead)
- [ ] Memory leak testing with long-running error collection
- [ ] Cross-platform testing (Linux, macOS, Windows)

## 🎯 Success Criteria

### **Functional Requirements**

- [ ] All three error handling patterns work correctly
- [ ] Macros generate expected methods without conflicts
- [ ] FFI integration provides native error handling for each language
- [ ] Error collection preserves full context for debugging
- [ ] Memory usage remains bounded and reasonable

### **Performance Targets**

- [ ] Error collection adds <10% overhead to failed operations
- [ ] Successful operations have <1% overhead from error infrastructure
- [ ] Memory usage scales linearly with error count
- [ ] Error serialization for FFI completes in <1ms for typical error counts

### **Quality Targets**

- [ ] 100% test coverage for error handling paths
- [ ] Zero unsafe code in error handling implementation
- [ ] Comprehensive documentation with examples
- [ ] All FFI languages provide idiomatic error handling

## 🔍 Open Questions & Decisions Needed

### **1. Error Retention Policy**

**Question**: Should we limit error collection to prevent memory growth?

**Options**:

- **A**: Unlimited collection (current proposal)
- **B**: Ring buffer with max capacity (e.g., 1000 errors)
- **C**: Time-based expiry (e.g., clear errors older than 1 hour)

**Recommendation**: Start with **Option A** for simplicity, add **Option B** if memory becomes an issue.

### **2. Timestamp Precision**

**Question**: What timestamp type should we use?

**Options**:

- **A**: `Instant` (relative, high precision, not serializable)
- **B**: `SystemTime` (absolute, serializable, can go backwards)
- **C**: Custom timestamp with epoch millis (serializable, monotonic)

**Recommendation**: **Option C** for FFI compatibility and serialization.

### **3. Context Information**

**Question**: How much parameter context should we capture?

**Options**:

- **A**: No context (just operation name)
- **B**: Parameter values as strings (current proposal)
- **C**: Full Debug formatting of all parameters

**Recommendation**: **Option B** with sensitive data filtering.

### **4. Macro Naming**

**Question**: What should we name the procedural macros?

**Options**:

- **A**: `#[generate_try_method]` and `#[generate_json_helper]` (descriptive)
- **B**: `#[try_variant]` and `#[json_ffi]` (concise)
- **C**: `#[fluent_try]` and `#[fluent_json]` (branded)

**Recommendation**: **Option A** for clarity during development, can shorten later.

## 🚀 Next Steps

1. **Get approval** for this design and implementation plan
2. **Start with Phase 1** - basic error collection infrastructure
3. **Iterate on macro design** with simple test cases first
4. **Validate FFI integration** early with one language (Python)
5. **Scale to full implementation** once core patterns are proven

---

**Ready for implementation approval and task assignment.**
