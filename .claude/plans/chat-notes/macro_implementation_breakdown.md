# Multi-FFI Macro Implementation Breakdown

**Created**: July 28, 2025\
**Estimated Total Time**: 8-12 hours with AI assistance\
**Complexity**: Moderate (proc macro + code generation)

## 📋 Implementation Tasks

### Phase 1: Proc Macro Foundation (2-3 hours)

**Task 1.1: Setup Proc Macro Crate (30 minutes)**

```toml
# Cargo.toml
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
```

**Task 1.2: Basic Attribute Parsing (1.5 hours)**

```rust
use syn::{parse_macro_input, AttributeArgs, ItemImpl, NestedMeta, Lit, Meta};

#[proc_macro_attribute]
pub fn multi_ffi(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemImpl);
    
    // Extract target languages: ["nodejs", "python", "uniffi"]
    let targets = parse_targets(args);
    
    // Generate wrappers for each target
    let wrappers = generate_all_wrappers(&input, &targets);
    
    quote! {
        #input  // Original implementation
        #(#wrappers)*  // Generated wrappers
    }.into()
}
```

**Task 1.3: Target Language Detection (1 hour)**

```rust
fn parse_targets(args: AttributeArgs) -> Vec<String> {
    args.into_iter()
        .filter_map(|arg| match arg {
            NestedMeta::Meta(Meta::Path(path)) => {
                path.get_ident().map(|i| i.to_string())
            }
            _ => None,
        })
        .collect()
}
```

### Phase 2: Code Generation Engine (3-4 hours)

**Task 2.1: Method Signature Analysis (1.5 hours)**

```rust
struct MethodInfo {
    name: syn::Ident,
    inputs: Vec<syn::FnArg>,
    output: syn::ReturnType,
    is_constructor: bool,
    visibility: syn::Visibility,
}

fn analyze_method(method: &syn::ImplItemMethod) -> MethodInfo {
    // Parse method signature
    // Detect #[constructor] attribute
    // Extract parameter types and return type
}
```

**Task 2.2: Node.js (napi-rs) Generator (1 hour)**

```rust
fn generate_nodejs_wrapper(method: &MethodInfo) -> TokenStream2 {
    let method_name = &method.name;
    let napi_name = format_ident!("nodejs_{}", method_name);
    
    if method.is_constructor {
        quote! {
            #[napi(constructor)]
            pub fn #napi_name() -> Self {
                Self::#method_name()
            }
        }
    } else {
        // Generate regular method wrapper
        let params = generate_napi_params(&method.inputs);
        let return_type = map_return_type(&method.output, "nodejs");
        
        quote! {
            #[napi]
            pub fn #napi_name(#params) -> #return_type {
                self.#method_name(#param_names)
                    .map_err(|e| napi::Error::from_reason(e.to_string()))
            }
        }
    }
}
```

**Task 2.3: Python (PyO3) Generator (1 hour)**

```rust
fn generate_python_wrapper(method: &MethodInfo) -> TokenStream2 {
    let method_name = &method.name;
    let py_name = format_ident!("python_{}", method_name);
    
    if method.is_constructor {
        quote! {
            #[new]
            fn #py_name() -> Self {
                Self::#method_name()
            }
        }
    } else {
        let params = generate_pyo3_params(&method.inputs);
        let return_type = map_return_type(&method.output, "python");
        
        quote! {
            fn #py_name(#params) -> #return_type {
                self.#method_name(#param_names)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
            }
        }
    }
}
```

**Task 2.4: UniFFI Generator (30 minutes)**

```rust
fn generate_uniffi_wrapper(method: &MethodInfo) -> TokenStream2 {
    let method_name = &method.name;
    let uniffi_name = format_ident!("uniffi_{}", method_name);
    
    if method.is_constructor {
        quote! {
            #[uniffi::constructor]
            pub fn #uniffi_name() -> Arc<Self> {
                Arc::new(Self::#method_name())
            }
        }
    } else {
        // Handle Arc<Self> wrapping for UniFFI
        quote! {
            pub fn #uniffi_name(self: Arc<Self>, #params) -> Arc<Self> {
                Arc::new(Arc::try_unwrap(self).unwrap().#method_name(#param_names))
            }
        }
    }
}
```

### Phase 3: Type System (2-3 hours)

**Task 3.1: Type Mapping Rules (1.5 hours)**

```rust
fn map_return_type(return_type: &syn::ReturnType, target: &str) -> TokenStream2 {
    match return_type {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => {
            match target {
                "nodejs" => map_to_napi_type(ty),
                "python" => map_to_pyo3_type(ty), 
                "uniffi" => map_to_uniffi_type(ty),
                _ => panic!("Unknown target: {}", target),
            }
        }
    }
}

fn map_to_napi_type(ty: &syn::Type) -> TokenStream2 {
    // Result<T, E> -> napi::Result<T>
    // String -> String
    // i32 -> i32
    // Self -> Self
}

fn map_to_pyo3_type(ty: &syn::Type) -> TokenStream2 {
    // Result<T, E> -> PyResult<T>
    // String -> String
    // Self -> Self
}

fn map_to_uniffi_type(ty: &syn::Type) -> TokenStream2 {
    // Self -> Arc<Self>
    // Result<T, E> -> Result<T, String>
}
```

**Task 3.2: Parameter Handling (1 hour)**

```rust
fn generate_napi_params(inputs: &[syn::FnArg]) -> TokenStream2 {
    // Convert &self to self, handle mut, etc.
}

fn generate_pyo3_params(inputs: &[syn::FnArg]) -> TokenStream2 {
    // Similar parameter transformation
}
```

**Task 3.3: Error Type Conversion (30 minutes)**

```rust
// Convert Result<T, MyError> to target-specific error types
fn generate_error_conversion(target: &str) -> TokenStream2 {
    match target {
        "nodejs" => quote! { .map_err(|e| napi::Error::from_reason(e.to_string())) },
        "python" => quote! { .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())) },
        "uniffi" => quote! { .map_err(|e| e.to_string()) },
        _ => quote! {},
    }
}
```

### Phase 4: Integration & Testing (1-2 hours)

**Task 4.1: Feature Flag Generation (30 minutes)**

```rust
fn wrap_with_cfg(tokens: TokenStream2, target: &str) -> TokenStream2 {
    let feature = match target {
        "nodejs" => "nodejs",
        "python" => "python", 
        "uniffi" => "uniffi",
        _ => return tokens,
    };
    
    quote! {
        #[cfg(feature = #feature)]
        #tokens
    }
}
```

**Task 4.2: Test Suite (1 hour)**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_method_generation() {
        // Test macro expansion on simple methods
    }
    
    #[test] 
    fn test_constructor_generation() {
        // Test constructor handling
    }
    
    #[test]
    fn test_error_handling() {
        // Test Result type conversion
    }
}
```

**Task 4.3: Documentation (30 minutes)**

- Usage examples
- Supported types
- Limitations

### Phase 5: Advanced Features (1-2 hours)

**Task 5.1: Generic Type Support (1 hour)**

```rust
// Handle generic methods like extract<T>()
fn handle_generics(method: &MethodInfo) -> Vec<TokenStream2> {
    // Generate type-specific methods for common types
    // extract_json(), extract_toml(), etc.
}
```

**Task 5.2: Async Method Support (1 hour)**

```rust
// Handle async methods appropriately per target
async fn async_method() -> Result<String, Error>
// becomes:
// Node.js: Promise<string>
// Python: Coroutine[str]  
// UniFFI: Future<String>
```

## 🔍 Example Usage After Implementation

```rust
// multi_ffi/src/lib.rs
use multi_ffi::multi_ffi;

#[multi_ffi(nodejs, python, uniffi)]
impl SuperConfig {
    #[constructor]
    pub fn new() -> Self {
        Self { builder: Figment::new() }
    }
    
    pub fn with_file(self, path: String) -> Self {
        self.with_file_impl(path)
    }
    
    pub fn extract_json(&self) -> Result<String, String> {
        // Implementation
    }
}
```

**Expands to ~200 lines of generated wrapper code automatically!**

## ⚡ Development Acceleration with AI

**With Claude Code assistance:**

- **Syntax parsing**: AI helps with syn/quote syntax
- **Code generation**: AI writes repetitive wrapper patterns
- **Testing**: AI generates comprehensive test cases
- **Debugging**: AI helps debug macro expansion issues

**Realistic timeline: 8-12 hours over 2-3 days**

## 🎯 Return on Investment

**One-time cost**: 8-12 hours macro development\
**Ongoing benefit**: Zero signature duplication forever

Every new method in SuperConfig:

- **Without macro**: Write 4 signatures (Rust + Node.js + Python + UniFFI)
- **With macro**: Write 1 signature, get 3 wrappers automatically

After 5-10 methods, the macro pays for itself in maintenance time saved.

## 🚀 Deployment Strategy

1. **MVP macro** (basic method wrapping) - 6 hours
2. **Test with SuperConfig** - validate it works
3. **Add advanced features** (generics, async) - 4 hours
4. **Open source the macro** - others can benefit too

The macro becomes a **reusable tool for the entire Rust ecosystem** - potentially more valuable than SuperConfig itself!
