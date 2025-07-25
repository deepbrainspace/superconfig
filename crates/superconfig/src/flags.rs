//! Configuration flags for SuperConfig macros
//!
//! These bitwise flags control the behavior of configuration macros.
//! Multiple flags can be combined using the `|` operator.
//!
//! # Examples
//!
//! ```rust
//! use superconfig::{SuperConfig, flags, with_env, with_file, with_cli};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct CliArgs { verbose: bool }
//!
//! let config = SuperConfig::new();
//! let cli_args = CliArgs { verbose: true };
//!
//! // Single flag
//! let config = with_env!(config, "APP_", flags::FILTER_EMPTY);
//!
//! // Multiple flags combined  
//! let config = with_file!(config, "config.toml", flags::REQUIRED | flags::FOLLOW_SYMLINKS);
//!
//! // Default behavior (no flags)
//! let config = with_cli!(config, cli_args, flags::DEFAULT);
//! ```

use bitflags::bitflags;

bitflags! {
    /// Configuration flags for SuperConfig macros
    /// 
    /// These flags control the behavior of configuration loading and processing.
    /// Multiple flags can be combined using the `|` operator.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Config: u32 {
        /// Default behavior (no special flags)
        const DEFAULT = 0x00000000;

        /// Filter out empty values (empty strings, arrays, objects)
        /// 
        /// When used with environment variables or CLI arguments, this flag
        /// prevents empty values from overriding meaningful configuration
        /// from other sources.
        const FILTER_EMPTY = 0x00000001;

        /// Require the resource to exist (fail if missing)
        /// 
        /// When used with files or other resources, this flag makes
        /// the configuration fail if the resource cannot be found.
        const REQUIRED = 0x00000002;

        /// Follow symbolic links when accessing files
        /// 
        /// When used with file operations, this flag enables
        /// following symbolic links to their target files.
        const FOLLOW_SYMLINKS = 0x00000004;

        /// Enable strict mode validation
        /// 
        /// When enabled, this flag makes configuration parsing
        /// more strict, failing on type mismatches or invalid values.
        const STRICT_MODE = 0x00000008;

        /// Cache results for performance
        /// 
        /// When enabled, this flag caches the results of expensive
        /// operations like file parsing or network requests.
        const CACHE_RESULTS = 0x00000010;

        /// Skip system-wide configuration locations
        /// 
        /// When used with hierarchical configuration, this flag
        /// skips system-wide config directories like /etc or ~/.config.
        const SKIP_SYSTEM = 0x00000020;

        /// Skip user-level configuration locations
        /// 
        /// When used with hierarchical configuration, this flag  
        /// skips user-level config directories in the home folder.
        const SKIP_USER = 0x00000040;
        
        // 25 more flags available with u32 (up to 0x80000000)
    }
}

// Re-export constants at the flags module level for clean usage
pub const DEFAULT: Config = Config::DEFAULT;
pub const FILTER_EMPTY: Config = Config::FILTER_EMPTY;
pub const REQUIRED: Config = Config::REQUIRED;
pub const FOLLOW_SYMLINKS: Config = Config::FOLLOW_SYMLINKS;
pub const STRICT_MODE: Config = Config::STRICT_MODE;
pub const CACHE_RESULTS: Config = Config::CACHE_RESULTS;
pub const SKIP_SYSTEM: Config = Config::SKIP_SYSTEM;
pub const SKIP_USER: Config = Config::SKIP_USER;