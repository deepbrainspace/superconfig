# SuperConfig FFI Compatibility Refactor Plan

**Created**: July 28, 2025  
**Status**: Ready for Implementation  
**Priority**: High  
**Goal**: Refactor SuperConfig to use `serde_json::Value` natively, making it naturally FFI-compatible

## 🎯 Refactoring Strategy

Convert SuperConfig from generic-based API to `serde_json::Value`-based API with convenience methods for Rust ergonomics.

### Before (Generic-based)
```rust
pub fn with_defaults<T: Serialize>(self, defaults: T) -> Self
pub fn extract<T: DeserializeOwned>(&self) -> Result<T, Error>
```

### After (Value-based + Convenience)
```rust
// Core API uses serde_json::Value
pub fn with_defaults(self, defaults: serde_json::Value) -> Self
pub fn extract(&self) -> Result<serde_json::Value, Error>

// Convenience methods for Rust ergonomics
pub fn with_defaults_typed<T: Serialize>(self, defaults: T) -> Self
pub fn extract_as<T: DeserializeOwned>(&self) -> Result<T, Error>
```

## 📋 Implementation Tasks

### Phase 1: Core Structure Updates (2 hours)

#### 1.1 Update lib.rs - Core API Signatures
- [ ] Change `extract<T>()` to `extract() -> Result<serde_json::Value, Error>`
- [ ] Add `extract_as<T>() -> Result<T, Error>` convenience method
- [ ] Update documentation examples to show both approaches

#### 1.2 Update fluent.rs - Builder Method Signatures  
- [ ] Change `with_defaults<T: Serialize>()` to `with_defaults(serde_json::Value)`
- [ ] Add `with_defaults_typed<T: Serialize>()` convenience method
- [ ] Change `with_cli_opt<T: Serialize>()` to `with_cli_opt(Option<serde_json::Value>)`
- [ ] Add `with_cli_typed<T: Serialize>()` convenience method
- [ ] Update all internal serialization calls to use Value

#### 1.3 Update access.rs - Accessor Method Returns
- [ ] Change `get_array<T>()` to `get_value() -> Result<serde_json::Value, Error>`
- [ ] Add `get_array_as<T>() -> Result<Vec<T>, Error>` convenience method
- [ ] Update `debug_messages()` to return serializable types or JSON strings
- [ ] Update `debug_sources()` to return JSON string

### Phase 2: Internal Implementation Updates (3 hours)

#### 2.1 Update Figment Integration
- [ ] Ensure all Figment interactions use `serde_json::Value` as intermediate type
- [ ] Update error handling to work with Value-based extraction
- [ ] Verify Deref implementation still provides access to all Figment methods

#### 2.2 Update Provider Integrations
- [ ] Verify Universal provider works with Value-based merging
- [ ] Verify Nested provider works with Value-based merging  
- [ ] Verify Wildcard provider works with Value-based merging
- [ ] Update Empty provider integration if needed

#### 2.3 Update Array Merging Logic (merge.rs)
- [ ] Verify array merging works with `serde_json::Value` throughout
- [ ] Update `apply_array_merging()` to handle Value types
- [ ] Test `_add`/`_remove` patterns work with new API

### Phase 3: Debug and Verbosity System Updates (1 hour)

#### 3.1 Debug Message Serialization
- [ ] Ensure `DebugMessage` derives `Serialize, Deserialize`
- [ ] Update `debug_messages()` to return `Vec<DebugMessage>` (already serializable)
- [ ] Add `debug_messages_json() -> String` method for FFI
- [ ] Ensure `VerbosityLevel` derives `Serialize, Deserialize`

#### 3.2 Metadata Serialization  
- [ ] Create wrapper type for `figment::Metadata` if not serializable
- [ ] Add `debug_sources_json() -> String` method
- [ ] Update `debug_config()` output format if needed

### Phase 4: Examples and Tests Updates (2 hours)

#### 4.1 Update Examples
- [ ] Update `examples/guardy_usage.rs` to use new API
- [ ] Show both convenience methods and direct Value usage
- [ ] Update documentation in examples

#### 4.2 Update Tests
- [ ] Update `tests/integration_tests.rs` for new API
- [ ] Update `tests/verbosity_tests.rs` for new API  
- [ ] Add tests for convenience methods
- [ ] Add tests for Value-based workflow
- [ ] Verify all existing functionality works

#### 4.3 Update Documentation
- [ ] Update README.md examples
- [ ] Update lib.rs documentation examples  
- [ ] Update CHANGELOG.md with breaking changes
- [ ] Update method documentation

### Phase 5: FFI Compatibility Verification (1 hour)

#### 5.1 Verify FFI-Ready Signatures
- [ ] Audit all public methods for FFI compatibility
- [ ] Ensure no remaining generic type parameters in core API
- [ ] Ensure all return types are FFI-compatible
- [ ] Document any remaining FFI limitations

#### 5.2 Create FFI Compatibility Test
- [ ] Create test that verifies all methods can be called with basic types
- [ ] Test JSON serialization/deserialization of all complex types
- [ ] Verify no Arc/Rc/RefCell in public API surface

## 📊 Method Conversion Table

| Current Method | New Core Method | Convenience Method | FFI Compatible |
|----------------|-----------------|-------------------|----------------|
| `with_defaults<T>(defaults)` | `with_defaults(Value)` | `with_defaults_typed<T>(T)` | ✅ |
| `with_cli_opt<T>(cli)` | `with_cli_opt(Option<Value>)` | `with_cli_typed<T>(T)` | ✅ |
| `extract<T>()` | `extract() -> Value` | `extract_as<T>() -> T` | ✅ |
| `get_array<T>(key)` | `get_value(key) -> Value` | `get_array_as<T>(key) -> Vec<T>` | ✅ |
| `debug_messages()` | `debug_messages() -> Vec<DebugMessage>` | N/A | ✅ |
| `debug_sources()` | `debug_sources_json() -> String` | N/A | ✅ |

## 🔧 Implementation Notes

### Error Handling Strategy
```rust
impl SuperConfig {
    pub fn extract(&self) -> Result<serde_json::Value, Error> {
        self.figment.extract::<serde_json::Value>()
    }
    
    pub fn extract_as<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let value = self.extract()?;
        serde_json::from_value(value).map_err(|e| 
            Error::from(figment::error::Kind::InvalidType(
                Actual::Other(e.to_string()),
                format!("valid {}", std::any::type_name::<T>()).into()
            ))
        )
    }
}
```

### Convenience Method Pattern
```rust
pub fn with_defaults_typed<T: Serialize>(self, defaults: T) -> Result<Self, Error> {
    let value = serde_json::to_value(defaults).map_err(|e|
        Error::from(figment::error::Kind::InvalidType(
            Actual::Other(e.to_string()),
            "serializable value".into()
        ))
    )?;
    Ok(self.with_defaults(value))
}
```

## ✅ Success Criteria

- [ ] All public methods use basic types (String, bool, i32, etc.) or `serde_json::Value`
- [ ] No generic type parameters in core API surface  
- [ ] All complex types implement `Serialize + Deserialize`
- [ ] Examples demonstrate both Value-based and typed workflows
- [ ] All tests pass with new API
- [ ] FFI compatibility verified through test suite
- [ ] Performance maintained or improved
- [ ] Rust ergonomics preserved through convenience methods

## 🚀 Expected Outcomes

1. **Multi-FFI Ready**: All methods can be wrapped directly by multi-ffi macro
2. **Zero FFI Exceptions**: No special cases or JSON conversion needed in macro
3. **Dual Ergonomics**: Rust users get typed convenience, FFI users get natural objects
4. **Performance**: No extra serialization overhead in critical paths
5. **Maintainability**: Single API to maintain instead of dual APIs

**Estimated Total Time**: 9 hours

After this refactor, SuperConfig will be naturally FFI-compatible and the multi-ffi macro can work "out of the box" on the entire API surface.