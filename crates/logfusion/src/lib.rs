//! # LogFusion - Unified Logging and Error Handling for Rust
//!
//! A tracing-native bridge that provides auto-initialization, complete tracing API coverage,
//! enhanced error handling, structured logging, and optional cross-language support.

// Include macros.rs first, before any logging macros are defined
include!("macros.rs");

#[doc(hidden)]
pub mod callback;
mod tracing;

pub use crate::tracing::*;

#[cfg(feature = "callback")]
pub use crate::callback::{Callback, set as set_callback};
