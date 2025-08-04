//! # meta-rust - Universal Rust Meta-Programming Toolkit
//!
//! Provides the `for_each!` macro for eliminating repetitive code patterns through powerful iteration.
//!
//! ## Features
//!
//! - **Multiple item types**: Identifiers, strings, numbers, and arrays
//! - **Array indexing**: Access elements with `%{param[0]}`, `%{param[1]}`, etc.
//! - **Clean syntax**: Uses `%{param}` to avoid conflicts with Rust syntax  
//! - **Template flexibility**: Multiple parameter references in same template
//! - **Production ready**: Comprehensive test coverage and optimized implementation
//!
//! ## Quick Start
//!
//! ```rust
//! use meta_rust::for_each;
//!
//! // Single items - generates error(), warn(), info() functions
//! for_each!([error, warn, info], |level| {
//!     pub fn %{level}(msg: &str) {
//!         println!("[{}] {}", stringify!(%{level}).to_uppercase(), msg);
//!     }
//! });
//!
//! // Array items - generates status_GET(), status_POST() functions  
//! for_each!([["GET", 200], ["POST", 201]], |req| {
//!     pub fn status_%{req[0]}() -> u16 {
//!         %{req[1]}
//!     }
//! });
//!
//! // Macro generation - LogFFI use case
//! for_each!([error, warn, info], |level| {
//!     macro_rules! %{level}_log {
//!         ($msg:expr) => {
//!             format!("[{}] {}", stringify!(%{level}).to_uppercase(), $msg)
//!         };
//!     }
//! });
//! ```
//!
//! ## Template Syntax - `%{}` Notation
//!
//! The `%{}` notation is our special template syntax that gets replaced during macro expansion:
//!
//! - `%{param}` - Replace with single item value
//! - `%{param[0]}` - Replace with first array element  
//! - `%{param[1]}` - Replace with second array element
//! - Multiple references supported in same template
//!
//! **Why `%{}`?** We chose this syntax to avoid conflicts with Rust's native syntax:
//! - Doesn't conflict with Rust 2024's prefix identifier parsing (`#identifier`)
//! - Visually distinct from standard Rust syntax
//! - Allows natural-looking templates that remain readable
//! - Supports complex expressions like `%{param[0]}` for array indexing
//!
//! ## Supported Item Types
//!
//! - **Identifiers**: `error`, `warn`, `info`
//! - **Strings**: `"GET"`, `"POST"`, `"PUT"`
//! - **Numbers**: `200`, `404`, `500`
//! - **Arrays**: `["GET", 200]`, `["POST", 201]`
//! - **Mixed**: `[error, "GET", 200]`

use proc_macro::TokenStream;

mod loops;

/// Universal iteration macro supporting single items and arrays.
///
/// This macro eliminates repetitive code patterns by iterating over a list of items
/// and applying a template to each one. Supports identifiers, strings, numbers, and arrays.
///
/// # Syntax
///
/// ```text
/// for_each!([item1, item2, ...], |param| {
///     // template using %{param} syntax
/// });
/// ```
///
/// # Examples
///
/// ## Single Items
/// ```rust
/// # use meta_rust::for_each;
/// for_each!([debug, info], |level| {
///     pub fn %{level}() -> &'static str {
///         stringify!(%{level})
///     }
/// });
/// # assert_eq!(debug(), "debug");
/// # assert_eq!(info(), "info");
/// ```
///
/// ## Array Items with Indexing
/// ```rust
/// # use meta_rust::for_each;
/// for_each!([["users", 1], ["posts", 2]], |table| {
///     pub fn get_%{table[0]}_id() -> u32 {
///         %{table[1]}
///     }
/// });
/// # assert_eq!(get_users_id(), 1);
/// # assert_eq!(get_posts_id(), 2);
/// ```
///
/// ## Macro Generation
/// ```rust
/// # use meta_rust::for_each;
/// for_each!([error, warn], |level| {
///     macro_rules! %{level}_msg {
///         ($msg:expr) => {
///             format!("[{}] {}", stringify!(%{level}).to_uppercase(), $msg)
///         };
///     }
/// });
/// # assert_eq!(error_msg!("failed"), "[ERROR] failed");
/// # assert_eq!(warn_msg!("warning"), "[WARN] warning");
/// ```
#[proc_macro]
pub fn for_each(input: TokenStream) -> TokenStream {
    loops::for_each::main(input)
}
