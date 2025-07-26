//! Extension traits for enhanced Figment functionality
//!
//! This module provides several extension traits that add powerful capabilities to regular Figment:
//!
//! ## Extension Traits
//!
//! - **`ExtendExt`** - Array merging with `_add` and `_remove` patterns
//! - **`FluentExt`** - Fluent builder methods (`with_file`, `with_env`, etc.)
//!   - ⚠️ **Note**: Automatically includes `ExtendExt` functionality (array merging)
//! - **`AccessExt`** - Convenience methods (`as_json`, `get_string`, etc.)
//!
//! ## Usage Examples
//!
//! ### With Regular Figment (Extension Traits)
//! ```rust
//! use figment::Figment;
//! use superconfig::prelude::*;  // Import all extension traits
//!
//! let config = Figment::new()
//!     .with_file("config")    // FluentExt method
//!     .with_env("APP_");      // FluentExt method  
//! ```
//!
//! ### With SuperConfig (Built-in Methods)
//! ```rust
//! use superconfig::{SuperConfig, with_file, with_env};
//!
//! let config = with_env!(
//!     with_file!(SuperConfig::new(), "config"),
//!     "APP_"
//! );
//! ```
//!
//! ### Selective Import (Advanced)
//! ```rust
//! use superconfig::ExtendExt;  // Just array merging
//! use superconfig::{FluentExt, AccessExt};  // Builder + convenience
//! ```

pub mod access;
pub mod extend;
pub mod fluent;

// Individual extension traits
pub use access::AccessExt;
pub use extend::ExtendExt;
pub use fluent::FluentExt;

/// Prelude module for convenient imports of all SuperConfig functionality
///
/// Import this module with `use superconfig::prelude::*` to get everything:
/// - Extension traits: `ExtendExt`, `FluentExt`, `AccessExt`
/// - Enhanced providers: `Universal`, `Nested`, `Empty`, `Hierarchical`
/// - SuperConfig struct and flags for advanced functionality
///
/// ## Basic Figment Enhancement
/// ```rust,no_run
/// use figment::Figment;
/// use superconfig::prelude::*;  // Everything you need!
///
/// let config = Figment::new()
///     .merge(Universal::file("config"))  // Enhanced provider
///     .with_env("APP_")                  // Extension trait method
///     .merge_extend(Nested::prefixed("DB_")); // Extension trait method
/// let json = config.as_json()?;          // Extension trait method
/// # Ok::<(), figment::Error>(())
/// ```
///
/// ## Full SuperConfig Power
/// ```rust,no_run
/// use superconfig::prelude::*;  // Everything you need!
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct CliArgs { debug: bool }
///
/// let cli_args = CliArgs { debug: true };
/// let config = SuperConfig::new()
///     .add_flags(flags::FILTER_EMPTY)     // Flag management
///     .with_file(Some("config.toml"))     // Enhanced file loading
///     .with_cli(Some(cli_args))           // CLI integration
///     .with_env("APP_");                  // Environment variables
/// let json = config.as_json()?;          // Convenience methods
/// # Ok::<(), figment::Error>(())
/// ```
pub mod prelude {
    // Extension traits - add methods to regular Figment
    pub use super::access::AccessExt;
    pub use super::extend::ExtendExt;
    pub use super::fluent::FluentExt;

    // Enhanced providers - drop-in replacements with superpowers
    pub use crate::providers::{Empty, Hierarchical, Nested, Universal};
    
    // SuperConfig struct for advanced functionality
    pub use crate::SuperConfig;
    
    // Flags module for configuration control
    pub use crate::flags;
}
