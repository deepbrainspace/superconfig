//! # SuperConfig Macros
//!
//! Procedural macros for SuperConfig fluent API error handling and FFI integration.
//!
//! ## Features
//!
//! - **Automatic try method generation** - Transform fallible methods into error-collecting variants
//! - **Bidirectional JSON helpers** - Generate FFI-compatible JSON serialization methods  
//! - **Intelligent type detection** - Auto-detect complex types for optimal JSON helper generation
//! - **Fluent API support** - Seamless integration with method chaining patterns
//! - **Zero runtime overhead** - Pure compile-time code generation
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! superconfig-macros = "0.1"
//! ```
//!
//! ## Core Macros
//!
//! This crate provides two key procedural macros:
//!
//! - [`macro@generate_try_method`] - Automatically generates `try_*` method variants that collect errors instead of returning them
//! - [`macro@generate_json_helper`] - Automatically generates `*_as_json` method variants for FFI compatibility
//!
//! ## Error Handling Philosophy
//!
//! The macros implement a dual error handling strategy:
//!
//! 1. **Strict Mode**: Use original methods with `?` operator for immediate error propagation
//! 2. **Permissive Mode**: Use `try_*` variants for error collection and deferred handling
//!
//! ```rust
//! // Example showing the dual error handling pattern
//! // (This is conceptual - actual implementation requires the ConfigBuilder type)
//!
//! // Fail-fast approach with ? operator:
//! // let config = builder.set_database_url("postgres://localhost")?;
//!
//! // Permissive approach with try_ variants:  
//! // let config = builder.try_set_database_url("invalid-url").try_build();
//! ```
//!
//! ## FFI Integration
//!
//! Generate JSON-compatible methods for seamless FFI integration:
//!
//! ```rust
//! // Example showing JSON helper generation
//! use superconfig_macros::generate_json_helper;
//!
//! // #[generate_json_helper(auto)]
//! // pub fn configure(self, settings: Settings) -> Result<Self, Error> {
//! //     // Implementation
//! // }
//!
//! // Auto-generates both directions based on type complexity:
//! // - configure_from_json(json_str: &str) -> Self
//! // - configure_as_json(settings: Settings) -> String
//! ```

use proc_macro::TokenStream;

/// Generates a `try_*` variant of a method that collects errors instead of returning them.
///
/// This macro transforms methods that return `Result<T, E>` into non-failing variants
/// that collect errors in an internal registry for later inspection, enabling both
/// strict error handling and permissive error collection in fluent APIs.
///
/// # Generated Pattern
///
/// For any method `foo` returning `Result<T, E>`, this macro generates:
///
/// ```rust,ignore
/// pub fn try_foo(self, param: P) -> Self {
///     match self.foo(param) {
///         Ok(result) => result,
///         Err(e) => {
///             self.collect_error("foo", e, Some(format!("foo({:?})", param)));
///             self
///         }
///     }
/// }
/// ```
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,ignore
/// use superconfig_macros::generate_try_method;
///
/// #[generate_try_method]
/// pub fn enable(self, flags: u64) -> Result<Self, RegistryError> {
///     if flags == 0 {
///         return Err(RegistryError::InvalidFlags);
///     }
///     Ok(Self { flags, ..self })
/// }
///
/// // Usage patterns:
/// // Strict: let config = config.enable(42)?;
/// // Permissive: let config = config.try_enable(42).try_set_timeout(5000);
/// ```
///
/// # Requirements
///
/// The target type must implement a `collect_error` method with the signature:
/// ```rust,ignore
/// fn collect_error(&self, method_name: &str, error: E, context: Option<String>);
/// ```
#[proc_macro_attribute]
pub fn generate_try_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    crate::try_method::generate_try_method_impl(_args, input)
}

/// Generates JSON helper methods for FFI compatibility with intelligent bidirectional support.
///
/// This macro can generate `*_as_json` (outgoing) and/or `*_from_json` (incoming) methods
/// based on the direction parameter. It automatically detects complex types to determine
/// which directions are needed when using `auto` mode.
///
/// # Direction Parameters
///
/// | Parameter | Generated Methods | Use Case |
/// |-----------|------------------|----------|
/// | `auto` (default) | Auto-detect based on signature | Most flexible |
/// | `out` | `*_as_json` only | Return values to FFI |
/// | `in` | `*_from_json` only | Accept JSON from FFI |
/// | `in,out` | Both directions | Full bidirectional FFI |
///
/// # Complex Type Detection
///
/// **Simple Types** (no JSON conversion needed):
/// - Primitives: `i32`, `u64`, `bool`, `f64`, etc.
/// - Basic strings: `String`, `&str`
/// - Basic containers: `Vec<T>` where T is simple
///
/// **Complex Types** (JSON conversion generated):
/// - Custom structs and enums
/// - Nested generics: `HashMap<String, Vec<CustomStruct>>`
/// - Result types: `Result<T, E>`
/// - Option types with complex inner types
///
/// # Examples
///
/// Auto-detection based on method signature:
///
/// ```rust,ignore
/// use superconfig_macros::generate_json_helper;
///
/// #[generate_json_helper(auto)]
/// pub fn configure(self, settings: DatabaseSettings) -> Result<Self, Error> {
///     // Complex param + complex return = generates both directions
/// }
///
/// // Auto-generates:
/// // pub fn configure_from_json(self, json_str: &str) -> Self { ... }
/// // pub fn configure_as_json(self, settings: DatabaseSettings) -> String { ... }
/// ```
///
/// Explicit direction control:
///
/// ```rust,ignore
/// #[generate_json_helper(out)]
/// pub fn get_status(self) -> Result<StatusInfo, Error> {
///     // Only generates get_status_as_json
/// }
///
/// #[generate_json_helper(in)]  
/// pub fn apply_config(self, config: Config) -> Result<Self, Error> {
///     // Only generates apply_config_from_json
/// }
/// ```
///
/// # Generated Method Patterns
///
/// **Incoming JSON** (`*_from_json`):
/// ```rust,ignore
/// pub fn method_from_json(self, json_str: &str) -> Self {
///     match serde_json::from_str::<ParamType>(json_str) {
///         Ok(param) => self.try_method(param),
///         Err(e) => {
///             self.collect_error("method_from_json", e, Some(json_str.to_string()));
///             self
///         }
///     }
/// }
/// ```
///
/// **Outgoing JSON** (`*_as_json`):
/// ```rust,ignore
/// pub fn method_as_json(self, param: ParamType) -> String {
///     match self.method(param) {
///         Ok(result) => json!({"success": true, "data": result}).to_string(),
///         Err(e) => json!({"success": false, "error": e.to_string()}).to_string()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn generate_json_helper(_args: TokenStream, input: TokenStream) -> TokenStream {
    crate::json_helper::generate_json_helper_impl(_args, input)
}

mod json_helper;
mod try_method;
