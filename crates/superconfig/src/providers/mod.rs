//! Enhanced configuration providers with performance optimizations
//!
//! These providers extend Figment with advanced capabilities:
//! - Universal: Smart format detection with optional caching
//! - Nested: Enhanced environment variables with JSON parsing and caching  
//! - Empty: Empty value filtering for clean CLI argument handling

pub mod cascade;
pub mod env;
pub mod filter;
pub mod format;

pub use cascade::Hierarchical;
pub use env::Nested;
pub use filter::Empty;
pub use format::Universal;
