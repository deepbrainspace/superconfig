//! Core registry system for `SuperConfig` V2
//!
//! This module provides the foundational handle-based registry system that enables
//! zero-copy configuration access with sub-microsecond lookup times.
//!
//! ## Modules
//!
//! - [`errors`] - Error types and handle ID definitions
//! - [`stats`] - Statistics tracking for registry operations
//! - [`handle`] - Type-safe handles for configuration access
//! - [`registry`] - Main configuration registry implementation
//!
//! ## Key Components
//!
//! - **`ConfigRegistry`**: The main registry for storing and accessing configuration data
//! - **`ConfigHandle`<T>**: Type-safe handles that provide zero-cost access
//! - **`RegistryStats`**: Performance and usage statistics
//! - **`RegistryError`**: Comprehensive error handling
//!
//! ## Examples
//!
//! ```
//! use superconfig::{ConfigRegistry, config_flags::{startup, runtime}};
//!
//! // Create registry with configuration flags
//! let registry = ConfigRegistry::custom(startup::SIMD | startup::THREAD_POOL)
//!     .enable(runtime::STRICT_MODE);
//!
//! // Store and retrieve configuration
//! let handle = registry.create("localhost".to_string()).unwrap();
//! let config = registry.read(&handle).unwrap();
//! assert_eq!(*config, "localhost");
//! ```

pub mod errors;
pub mod handle;
pub mod registry;
pub mod stats;

// Re-export key types for convenient access
pub use errors::{HandleId, RegistryError};
pub use handle::ConfigHandle;
pub use registry::{ConfigRegistry, global_registry};
pub use stats::RegistryStats;
