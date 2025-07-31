//! Procedural macros for SuperConfig fluent API error handling and FFI integration
//!
//! This crate provides two key procedural macros:
//!
//! - [`generate_try_method`] - Automatically generates `try_*` method variants that collect errors instead of returning them
//! - [`generate_json_helper`] - Automatically generates `*_as_json` method variants for FFI compatibility
//!
//! These macros work together to enable both strict error handling (with `?`) and permissive error collection
//! patterns in fluent APIs, while providing seamless FFI integration.

use proc_macro::TokenStream;

/// Generates a `try_*` variant of a method that collects errors instead of returning them.
///
/// This macro transforms methods that return `Result<T, E>` into non-failing variants
/// that collect errors in an internal registry for later inspection.
///
/// # Example
///
/// ```ignore
/// #[generate_try_method]
/// pub fn enable(self, flags: u64) -> Result<Self, RegistryError> {
///     // Original implementation
/// }
///
/// // Generates:
/// // pub fn try_enable(self, flags: u64) -> Self {
/// //     match self.enable(flags) {
/// //         Ok(result) => result,
/// //         Err(e) => {
/// //             self.collect_error("enable", e, Some(format!("enable({})", flags)));
/// //             self
/// //         }
/// //     }
/// // }
/// ```
#[proc_macro_attribute]
pub fn generate_try_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    crate::try_method::generate_try_method_impl(_args, input)
}

/// Generates JSON helper methods for FFI compatibility with bidirectional support.
///
/// This macro can generate `*_as_json` (outgoing) and/or `*_from_json` (incoming) methods
/// based on the direction parameter. It automatically detects complex types to determine
/// which directions are needed when using `auto` mode.
///
/// # Parameters
///
/// - `auto` (default): Auto-detect based on method signature
/// - `out`: Generate only `*_as_json` method (outgoing)
/// - `in`: Generate only `*_from_json` method (incoming)  
/// - `in,out`: Generate both methods (bidirectional)
///
/// # Example
///
/// ```ignore
/// #[generate_json_helper(auto)]
/// pub fn enable(self, flags: u64) -> Result<Self, RegistryError> {
///     // Original implementation
/// }
///
/// // Auto-detects: simple params + complex return = generates _as_json only
/// // pub fn enable_as_json(self, flags: u64) -> String {
/// //     match self.enable(flags) {
/// //         Ok(result) => json!({"success": true, "data": result}),
/// //         Err(e) => json!({"success": false, "error": e.to_string()})
/// //     }
/// // }
/// ```
///
/// # Complex Type Detection
///
/// - **Incoming**: Methods with complex parameter types get `*_from_json`
/// - **Outgoing**: Methods with complex return types get `*_as_json`
/// - **Both**: Methods with both complex params and returns get both variants
#[proc_macro_attribute]
pub fn generate_json_helper(_args: TokenStream, input: TokenStream) -> TokenStream {
    crate::json_helper::generate_json_helper_impl(_args, input)
}

mod json_helper;
mod try_method;
