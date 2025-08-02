//! # MultiFFI - Multi-Language FFI Binding Generator
//!
//! MultiFFI is a powerful procedural macro that automatically generates FFI bindings for multiple target languages
//! from your Rust code. Write your Rust code once, and get Python, Node.js, and WebAssembly bindings automatically.
//!
//! ## Features
//!
//! - **Python bindings** via PyO3 (feature: `python`) - preserves `snake_case`
//! - **Node.js bindings** via NAPI (feature: `nodejs`) - automatic `camelCase` conversion
//! - **WebAssembly bindings** via wasm-bindgen (feature: `wasm`) - automatic `camelCase` conversion
//! - **Automatic naming conventions** for consistent JavaScript APIs
//! - **Zero-cost abstractions** - only generates code for enabled features
//! - **Simple annotation** - just add `#[multiffi]` to your items
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! multiffi = { version = "0.1", features = ["python", "nodejs", "wasm"] }
//! ```
//!
//! Then annotate your Rust code:
//! ```ignore
//! use multiffi::multiffi;
//!
//! #[multiffi]
//! pub struct Config {
//!     pub name: String,
//!     pub version: u32,
//! }
//!
//! #[multiffi]
//! impl Config {
//!     pub fn new(name: String, version: u32) -> Self {
//!         Self { name, version }
//!     }
//!     
//!     pub fn get_info(&self) -> String {
//!         format!("{} v{}", self.name, self.version)
//!     }
//! }
//!
//! #[multiffi]
//! pub fn create_default_config() -> Config {
//!     Config::new("MyApp".to_string(), 1)
//! }
//! ```
//!
//! ## Supported Items
//!
//! MultiFFI can be applied to:
//! - **Structs** - Generates language-specific class/object bindings
//! - **Impl blocks** - Generates method bindings for the target languages
//! - **Functions** - Generates standalone function bindings
//!
//! ## Automatic Naming Conventions
//!
//! MultiFFI automatically converts function names to match target language conventions:
//!
//! | Rust Function | Python | Node.js | WebAssembly |
//! |---------------|--------|---------|-------------|
//! | `get_info()` | `get_info()` | `getInfo()` | `getInfo()` |
//! | `set_debug()` | `set_debug()` | `setDebug()` | `setDebug()` |
//! | `with_file()` | `with_file()` | `withFile()` | `withFile()` |
//!
//! This ensures APIs feel natural in each target language while maintaining consistency.
//!
//! ## Feature Flags
//!
//! Enable only the target languages you need:
//! - `python` - Generates PyO3 bindings for Python
//! - `nodejs` - Generates NAPI bindings for Node.js
//! - `wasm` - Generates wasm-bindgen bindings for WebAssembly
//! - `all` - Enables all target languages
//!
//! ## Safety and Limitations
//!
//! - All generated bindings follow the safety requirements of their respective FFI frameworks
//! - Complex generic types may not be supported across all target languages
//! - Async functions are not currently supported
//! - Some Rust-specific features (like advanced lifetime annotations) may not translate directly

use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, Item, ItemFn, ItemImpl, ItemStruct, parse_macro_input};

/// A procedural macro that generates FFI bindings for multiple target languages.
///
/// This macro can be applied to structs, impl blocks, and functions to automatically generate
/// bindings for Python (PyO3), Node.js (NAPI), and WebAssembly (wasm-bindgen) based on enabled features.
///
/// **Naming Conventions:** MultiFFI automatically converts `snake_case` function names to `camelCase`
/// for JavaScript targets (Node.js and WebAssembly), while preserving `snake_case` for Python.
///
/// ## Usage
///
/// ### On Structs
/// Generates language-specific class/object bindings:
/// ```ignore
/// #[multiffi]
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
/// }
/// ```
///
/// ### On Impl Blocks
/// Generates method bindings for the struct:
/// ```ignore
/// #[multiffi]
/// impl Person {
///     pub fn new(name: String, age: u32) -> Self {
///         Self { name, age }
///     }
///     
///     pub fn greet(&self) -> String {
///         format!("Hello, I'm {} and I'm {} years old", self.name, self.age)
///     }
/// }
/// ```
///
/// ### On Functions
/// Generates standalone function bindings:
/// ```ignore
/// #[multiffi]
/// pub fn add_numbers(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
///
/// ## Naming Convention Examples
///
/// For the Rust function `pub fn get_user_info()`, MultiFFI generates:
/// - **Python**: `get_user_info()` (preserved snake_case)
/// - **Node.js**: `getUserInfo()` (NAPI converts automatically)
/// - **WebAssembly**: `getUserInfo()` (MultiFFI converts manually)
///
/// ## Generated Bindings
///
/// Based on enabled features, this macro generates appropriate annotations:
/// - **Python**: `#[pyo3::pyclass]`, `#[pyo3::pymethods]`, `#[pyo3::pyfunction]`
/// - **Node.js**: `#[napi::napi]`, `#[napi::napi(object)]`
/// - **WebAssembly**: `#[wasm_bindgen::prelude::wasm_bindgen]`
///
/// ## Arguments
///
/// Currently, this macro doesn't accept any arguments. Configuration is done through Cargo features.
///
/// ## Errors
///
/// This macro will produce a compilation error if applied to unsupported items:
/// - Enums (not yet supported)
/// - Traits (not supported)
/// - Modules (not supported)
/// - Other item types
///
/// ## Examples
///
/// ### Basic Configuration Struct
/// ```ignore
/// use multiffi::multiffi;
///
/// #[multiffi]
/// pub struct DatabaseConfig {
///     pub host: String,
///     pub port: u16,
///     pub database: String,
/// }
///
/// #[multiffi]
/// impl DatabaseConfig {
///     pub fn new(host: String, port: u16, database: String) -> Self {
///         Self { host, port, database }
///     }
///     
///     pub fn connection_string(&self) -> String {
///         format!("{}:{}/{}", self.host, self.port, self.database)
///     }
/// }
/// ```
///
/// ### Utility Functions
/// ```ignore
/// #[multiffi]
/// pub fn calculate_hash(input: String) -> String {
///     use std::collections::hash_map::DefaultHasher;
///     use std::hash::{Hash, Hasher};
///     
///     let mut hasher = DefaultHasher::new();
///     input.hash(&mut hasher);
///     format!("{:x}", hasher.finish())
/// }
/// ```
#[proc_macro_attribute]
pub fn multiffi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(input as Item);

    match input_item {
        Item::Struct(item_struct) => generate_struct_bindings(item_struct),
        Item::Impl(item_impl) => generate_impl_bindings(item_impl),
        Item::Fn(item_fn) => generate_fn_bindings(item_fn),
        _ => syn::Error::new_spanned(
            &input_item,
            "multiffi can only be applied to structs, impls, or functions",
        )
        .to_compile_error()
        .into(),
    }
}

// ============================================================================
// Naming conversion utilities for cross-language consistency
// ============================================================================

/// Converts snake_case identifiers to camelCase for JavaScript environments.
///
/// This function provides generic naming conversion to ensure consistent APIs across
/// Node.js and WebAssembly targets, both of which use camelCase conventions.
///
/// ## Examples
///
/// ```ignore
/// assert_eq!(convert_to_camel_case("with_file"), "withFile");
/// assert_eq!(convert_to_camel_case("set_debug"), "setDebug");
/// assert_eq!(convert_to_camel_case("extract_json"), "extractJson");
/// assert_eq!(convert_to_camel_case("single"), "single");
/// ```
///
/// ## Rules
///
/// - First word remains lowercase
/// - Subsequent words have their first letter capitalized
/// - Underscores are removed
/// - Empty segments are ignored
/// - Single words are returned unchanged
///
/// ## Parameters
///
/// * `snake_name` - The snake_case identifier to convert
///
/// ## Returns
///
/// A String containing the camelCase equivalent
#[cfg(feature = "wasm")]
fn convert_to_camel_case(snake_name: &str) -> String {
    let parts: Vec<&str> = snake_name.split('_').filter(|s| !s.is_empty()).collect();

    // Handle empty or single word case
    if parts.is_empty() {
        return String::new();
    }
    if parts.len() == 1 {
        return parts[0].to_string();
    }

    let mut result = String::new();

    // First non-empty part stays lowercase
    if let Some(first) = parts.first() {
        result.push_str(first);
    }

    // Subsequent parts get capitalized
    for part in parts.iter().skip(1) {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            result.push(first_char.to_ascii_uppercase());
            result.extend(chars);
        }
    }

    result
}

/// Generates FFI bindings for struct definitions.
///
/// This function takes a parsed struct and adds appropriate FFI annotations
/// for all enabled target languages to the same struct definition.
///
/// ## Generated Bindings
///
/// The original struct gets annotated with all enabled target bindings:
/// - **Python**: `#[pyo3::pyclass]` for PyO3 compatibility
/// - **Node.js**: `#[napi::napi(object)]` for NAPI compatibility  
/// - **WebAssembly**: `#[wasm_bindgen::prelude::wasm_bindgen]` for wasm-bindgen compatibility
///
/// ## Parameters
///
/// * `item_struct` - The parsed struct from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the struct with all appropriate FFI annotations
#[allow(unused_variables)]
fn generate_struct_bindings(item_struct: ItemStruct) -> TokenStream {
    // Create mutable binding only when features that require mutation are enabled
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut item_struct = item_struct;

    // Add FFI annotations to the original struct based on enabled features

    #[cfg(feature = "python")]
    {
        item_struct.attrs.push(syn::parse_quote!(#[pyo3::pyclass]));
    }

    #[cfg(feature = "nodejs")]
    {
        item_struct
            .attrs
            .push(syn::parse_quote!(#[napi::napi(object)]));
    }

    #[cfg(feature = "wasm")]
    {
        item_struct
            .attrs
            .push(syn::parse_quote!(#[wasm_bindgen::prelude::wasm_bindgen]));
    }

    // Always add Clone derive for FFI compatibility
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    {
        item_struct.attrs.push(syn::parse_quote!(#[derive(Clone)]));
    }

    quote! { #item_struct }.into()
}

/// Generates FFI bindings for impl block methods.
///
/// This function takes a parsed impl block and adds appropriate FFI annotations
/// to each method within the same impl block, avoiding duplicate impl blocks.
///
/// ## Generated Bindings
///
/// Each method in the impl block gets annotated with enabled target bindings:
/// - **Python**: `#[pyo3::pyfunction]` or `#[pyo3::pymethods]` on impl block
/// - **Node.js**: `#[napi::napi]` on each method  
/// - **WebAssembly**: `#[wasm_bindgen::prelude::wasm_bindgen]` on each method
///
/// ## Parameters
///
/// * `item_impl` - The parsed impl block from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the impl block with FFI-annotated methods
#[allow(unused_variables)]
fn generate_impl_bindings(mut item_impl: ItemImpl) -> TokenStream {
    let struct_type = &item_impl.self_ty;

    // Add impl-level annotations for certain targets
    #[cfg(feature = "python")]
    {
        item_impl.attrs.push(syn::parse_quote!(#[pyo3::pymethods]));
    }

    #[cfg(feature = "nodejs")]
    {
        item_impl.attrs.push(syn::parse_quote!(#[napi::napi]));
    }

    #[cfg(feature = "wasm")]
    {
        item_impl
            .attrs
            .push(syn::parse_quote!(#[wasm_bindgen::prelude::wasm_bindgen]));
    }

    // Add method-level annotations to each function
    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            // Add Python method annotation
            #[cfg(feature = "python")]
            {
                // pymethods impl blocks handle individual method binding automatically
                // No per-method annotation needed for Python
            }

            // Add Node.js method annotation
            #[cfg(feature = "nodejs")]
            {
                method.attrs.push(syn::parse_quote!(#[napi::napi]));
            }

            // Add WASM method annotation
            #[cfg(feature = "wasm")]
            {
                method
                    .attrs
                    .push(syn::parse_quote!(#[wasm_bindgen::prelude::wasm_bindgen]));

                // Add js_name attribute for camelCase in JavaScript
                let original_name = &method.sig.ident;
                let camel_name = convert_to_camel_case(&original_name.to_string());
                if *original_name != camel_name {
                    method
                        .attrs
                        .push(syn::parse_quote!(#[wasm_bindgen(js_name = #camel_name)]));
                }
            }
        }
    }

    quote! { #item_impl }.into()
}

/// Generates FFI bindings for standalone functions.
///
/// This function takes a parsed function and adds appropriate FFI annotations
/// for all enabled target languages to the same function.
///
/// ## Generated Bindings
///
/// The original function gets annotated with all enabled target bindings:
/// - **Python**: `#[pyo3::pyfunction]` annotation
/// - **Node.js**: `#[napi::napi]` annotation  
/// - **WebAssembly**: `#[wasm_bindgen::prelude::wasm_bindgen]` annotation
///
/// ## Parameters
///
/// * `item_fn` - The parsed function from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the function with all appropriate FFI annotations
fn generate_fn_bindings(item_fn: ItemFn) -> TokenStream {
    // Create mutable binding only when features that require mutation are enabled
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut item_fn = item_fn;

    // Add FFI annotations to the original function based on enabled features

    #[cfg(feature = "python")]
    {
        item_fn.attrs.push(syn::parse_quote!(#[pyo3::pyfunction]));
    }

    #[cfg(feature = "nodejs")]
    {
        item_fn.attrs.push(syn::parse_quote!(#[napi::napi]));
    }

    #[cfg(feature = "wasm")]
    {
        item_fn
            .attrs
            .push(syn::parse_quote!(#[wasm_bindgen::prelude::wasm_bindgen]));
    }

    quote! { #item_fn }.into()
}

// Tests are in a separate module to keep lib.rs clean
#[cfg(test)]
mod tests;
