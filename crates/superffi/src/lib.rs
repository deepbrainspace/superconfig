//! # SuperFFI - Multi-Language FFI Binding Generator
//!
//! SuperFFI is a powerful procedural macro that automatically generates FFI bindings for multiple target languages
//! from your Rust code. Write your Rust code once, and get Python, Node.js, and WebAssembly bindings automatically.
//!
//! ## Features
//!
//! - **Python bindings** via PyO3 (feature: `python`)
//! - **Node.js bindings** via NAPI (feature: `nodejs`)
//! - **WebAssembly bindings** via wasm-bindgen (feature: `wasm`)
//! - **Zero-cost abstractions** - only generates code for enabled features
//! - **Simple annotation** - just add `#[superffi]` to your items
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! superffi = { version = "0.1", features = ["python", "nodejs", "wasm"] }
//! ```
//!
//! Then annotate your Rust code:
//! ```ignore
//! use superffi::superffi;
//!
//! #[superffi]
//! pub struct Config {
//!     pub name: String,
//!     pub version: u32,
//! }
//!
//! #[superffi]
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
//! #[superffi]
//! pub fn create_default_config() -> Config {
//!     Config::new("MyApp".to_string(), 1)
//! }
//! ```
//!
//! ## Supported Items
//!
//! SuperFFI can be applied to:
//! - **Structs** - Generates language-specific class/object bindings
//! - **Impl blocks** - Generates method bindings for the target languages
//! - **Functions** - Generates standalone function bindings
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
use syn::{Item, ItemFn, ItemImpl, ItemStruct, parse_macro_input};

#[cfg(feature = "wasm")]
use syn::{ImplItem, Ident};

/// A procedural macro that generates FFI bindings for multiple target languages.
///
/// This macro can be applied to structs, impl blocks, and functions to automatically generate
/// bindings for Python (PyO3), Node.js (NAPI), and WebAssembly (wasm-bindgen) based on enabled features.
///
/// ## Usage
///
/// ### On Structs
/// Generates language-specific class/object bindings:
/// ```ignore
/// #[superffi]
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
/// }
/// ```
///
/// ### On Impl Blocks
/// Generates method bindings for the struct:
/// ```ignore
/// #[superffi]
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
/// #[superffi]
/// pub fn add_numbers(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
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
/// use superffi::superffi;
///
/// #[superffi]
/// pub struct DatabaseConfig {
///     pub host: String,
///     pub port: u16,
///     pub database: String,
/// }
///
/// #[superffi]
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
/// #[superffi]
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
pub fn superffi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(input as Item);

    match input_item {
        Item::Struct(item_struct) => generate_struct_bindings(item_struct),
        Item::Impl(item_impl) => generate_impl_bindings(item_impl),
        Item::Fn(item_fn) => generate_fn_bindings(item_fn),
        _ => syn::Error::new_spanned(
            &input_item,
            "superffi can only be applied to structs, impls, or functions",
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

/// Creates a new identifier with camelCase naming for JavaScript targets.
///
/// This helper function takes a Rust identifier and creates a new Ident with the
/// camelCase equivalent for use in JavaScript bindings (Node.js and WebAssembly).
///
/// ## Parameters
///
/// * `ident` - The original Rust identifier
///
/// ## Returns
///
/// A new Ident with camelCase naming, preserving the original span
#[cfg(feature = "wasm")]
fn create_camel_case_ident(ident: &Ident) -> Ident {
    let camel_name = convert_to_camel_case(&ident.to_string());
    Ident::new(&camel_name, ident.span())
}

/// Generates FFI bindings for struct definitions.
///
/// This function takes a parsed struct and generates appropriate bindings for all enabled
/// target languages. Each target language receives the original struct plus language-specific
/// annotations.
///
/// ## Generated Bindings
///
/// - **Python**: Adds `#[pyo3::pyclass]` and `#[derive(Clone)]` for PyO3 compatibility
/// - **Node.js**: Adds `#[napi::napi(object)]` and `#[derive(Clone)]` for NAPI compatibility  
/// - **WebAssembly**: Adds `#[wasm_bindgen::prelude::wasm_bindgen]` and `#[derive(Clone)]`
///
/// ## Parameters
///
/// * `item_struct` - The parsed struct from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the original struct plus any generated language bindings
#[allow(unused_variables)]
fn generate_struct_bindings(item_struct: ItemStruct) -> TokenStream {
    let struct_name = &item_struct.ident;
    let struct_fields = &item_struct.fields;

    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_struct };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_struct };

    // Generate Python bindings
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pyclass]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    // Generate Node.js bindings
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi(object)]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    // Generate WASM bindings
    #[cfg(feature = "wasm")]
    {
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            #[derive(Clone)]
            pub struct #struct_name #struct_fields
        });
    }

    output.into()
}

/// Generates FFI bindings for impl block methods.
///
/// This function takes a parsed impl block and generates method bindings for all enabled
/// target languages. The original impl block is preserved, and additional language-specific
/// impl blocks are generated with appropriate annotations.
///
/// ## Generated Bindings
///
/// - **Python**: Creates a `#[pyo3::pymethods]` impl block with all methods
/// - **Node.js**: Creates a `#[napi::napi]` impl block with `#[napi::napi]` on each method
/// - **WebAssembly**: Creates a `#[wasm_bindgen::prelude::wasm_bindgen]` impl block with annotations on each method
///
/// ## Parameters
///
/// * `item_impl` - The parsed impl block from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the original impl block plus any generated language bindings
///
/// ## Note
///
/// The generated bindings assume that the target struct has already been annotated with
/// appropriate language-specific attributes via `generate_struct_bindings`.
#[allow(unused_variables)]
fn generate_impl_bindings(item_impl: ItemImpl) -> TokenStream {
    let struct_type = &item_impl.self_ty;
    let impl_items = &item_impl.items;

    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_impl };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_impl };

    // Generate Python method bindings
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pymethods]
            impl #struct_type {
                #(#impl_items)*
            }
        });
    }

    // Generate Node.js method bindings (Node.js automatically converts to camelCase)
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi]
            impl #struct_type {
                #(
                    #[napi::napi]
                    #impl_items
                )*
            }
        });
    }

    // Generate WASM method bindings with camelCase naming
    #[cfg(feature = "wasm")]
    {
        let wasm_methods = impl_items.iter().map(|item| {
            if let ImplItem::Fn(method) = item {
                let original_name = &method.sig.ident;
                let camel_name = create_camel_case_ident(original_name);
                let sig = &method.sig;
                let block = &method.block;
                let attrs = &method.attrs;
                let vis = &method.vis;
                
                // Create new signature with camelCase name
                let mut new_sig = sig.clone();
                new_sig.ident = camel_name;
                
                quote! {
                    #(#attrs)*
                    #[wasm_bindgen::prelude::wasm_bindgen]
                    #vis #new_sig #block
                }
            } else {
                quote! { #item }
            }
        });
        
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            impl #struct_type {
                #(#wasm_methods)*
            }
        });
    }

    output.into()
}

/// Generates FFI bindings for standalone functions.
///
/// This function takes a parsed function and generates appropriate bindings for all enabled
/// target languages. The original function is preserved, and additional language-specific
/// function definitions are generated with appropriate annotations.
///
/// ## Generated Bindings
///
/// - **Python**: Adds `#[pyo3::pyfunction]` annotation to make the function callable from Python
/// - **Node.js**: Adds `#[napi::napi]` annotation to make the function callable from Node.js
/// - **WebAssembly**: Adds `#[wasm_bindgen::prelude::wasm_bindgen]` annotation for WASM exports
///
/// ## Parameters
///
/// * `item_fn` - The parsed function from the original Rust code
///
/// ## Returns
///
/// A `TokenStream` containing the original function plus any generated language bindings
///
/// ## Limitations
///
/// - Async functions are not currently supported
/// - Functions with complex generic parameters may not work across all target languages
/// - Some Rust-specific types may need manual conversion for certain target languages
fn generate_fn_bindings(item_fn: ItemFn) -> TokenStream {
    // Only make output mutable if we actually have features that will extend it
    #[cfg(any(feature = "python", feature = "nodejs", feature = "wasm"))]
    let mut output = quote! { #item_fn };

    #[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
    let output = quote! { #item_fn };

    // Generate Python function binding
    #[cfg(feature = "python")]
    {
        output.extend(quote! {
            #[pyo3::pyfunction]
            #item_fn
        });
    }

    // Generate Node.js function binding (Node.js automatically converts to camelCase)
    #[cfg(feature = "nodejs")]
    {
        output.extend(quote! {
            #[napi::napi]
            #item_fn
        });
    }

    // Generate WASM function binding with camelCase naming
    #[cfg(feature = "wasm")]
    {
        let original_name = &item_fn.sig.ident;
        let camel_name = create_camel_case_ident(original_name);
        let mut wasm_fn = item_fn.clone();
        wasm_fn.sig.ident = camel_name;
        
        output.extend(quote! {
            #[wasm_bindgen::prelude::wasm_bindgen]
            #wasm_fn
        });
    }

    output.into()
}

// Tests are in a separate module to keep lib.rs clean
#[cfg(test)]
mod tests;
