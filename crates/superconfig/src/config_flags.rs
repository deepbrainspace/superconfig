//! Configuration flags for controlling `SuperConfig` V2 behavior
//!
//! This module provides three separate flag systems for different aspects of registry configuration:
//!
//! - **startup**: Flags that affect internal structures and must be set at registry creation
//! - **runtime**: Flags that can be toggled during registry operation
//! - **verbosity**: Logging verbosity levels that can be changed at runtime

use std::fmt;
use thiserror::Error;

/// Startup flags - affect internal structures and cannot be changed after registry creation
pub mod startup {
    /// Enable SIMD acceleration for parsing operations
    /// Affects parser pipeline initialization
    pub const SIMD: u32 = 1 << 0;

    /// Pre-allocate thread pool for parallel operations
    /// Thread pool cannot be created/destroyed at runtime
    pub const THREAD_POOL: u32 = 1 << 1;

    /// Enable detailed statistics collection with comprehensive metrics
    /// Statistics structure affects memory layout
    pub const DETAILED_STATS: u32 = 1 << 2;
}

/// Runtime flags - can be enabled/disabled freely without affecting core structures
pub mod runtime {
    /// Enable array merge operations with _ADD/_REMOVE suffixes
    /// Can be disabled for security in production environments
    pub const ARRAY_MERGE: u64 = 1 << 0;

    /// Enable parallel loading for multiple configuration files
    /// Can be disabled to reduce resource usage
    pub const PARALLEL: u64 = 1 << 1;

    /// Enable strict validation mode with comprehensive error checking
    /// Can be toggled based on environment (development vs production)
    pub const STRICT_MODE: u64 = 1 << 2;

    /// Enable environment variable expansion (${VAR} syntax)
    /// Can be disabled for security in production environments
    pub const ENV_EXPANSION: u64 = 1 << 3;

    /// Enable format auto-detection fallbacks when explicit format fails
    /// Can be disabled for strict format requirements
    pub const FORMAT_FALLBACK: u64 = 1 << 4;
}

/// Verbosity levels for logging and debugging output
pub mod verbosity {
    /// No logging output
    pub const NONE: u8 = 0;

    /// Warning messages only
    pub const WARN: u8 = 1;

    /// Debug information
    pub const DEBUG: u8 = 2;

    /// Detailed trace information
    pub const TRACE: u8 = 3;
}

/// Verbosity level enum for type-safe usage (convenience wrapper around u8 constants)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbosityLevel {
    /// No logging output
    None = verbosity::NONE,
    /// Warning messages only
    Warn = verbosity::WARN,
    /// Debug information
    Debug = verbosity::DEBUG,
    /// Detailed trace information
    Trace = verbosity::TRACE,
}

impl fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Warn => write!(f, "warn"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

impl From<u8> for VerbosityLevel {
    fn from(value: u8) -> Self {
        match value {
            verbosity::WARN => Self::Warn,
            verbosity::DEBUG => Self::Debug,
            verbosity::TRACE => Self::Trace,
            _ => Self::None, // Default to None for invalid values (includes NONE)
        }
    }
}

impl From<VerbosityLevel> for u8 {
    fn from(level: VerbosityLevel) -> Self {
        level as Self
    }
}

/// Errors that can occur during flag operations
#[derive(Error, Debug, Clone)]
pub enum FlagError {
    /// Attempted to modify startup flags at runtime
    #[error("Cannot modify startup flags at runtime - they are immutable after registry creation")]
    ImmutableStartupFlag,

    /// Invalid verbosity level
    #[error("Invalid verbosity level: {level} (valid range: 0-3)")]
    InvalidVerbosity {
        /// The invalid level value
        level: u8,
    },

    /// Invalid runtime flag value
    #[error("Invalid runtime flag value: 0x{flags:X}")]
    InvalidFlag {
        /// The invalid flag value
        flags: u64,
    },

    /// Invalid runtime flag value (legacy)
    #[error("Invalid runtime flag value: 0x{flag:X}")]
    InvalidRuntimeFlag {
        /// The invalid flag value
        flag: u64,
    },

    /// Invalid startup flag value  
    #[error("Invalid startup flag value: 0x{flag:X}")]
    InvalidStartupFlag {
        /// The invalid flag value
        flag: u32,
    },
}

/// All valid runtime flags combined
const ALL_RUNTIME_FLAGS: u64 = runtime::ARRAY_MERGE
    | runtime::PARALLEL
    | runtime::STRICT_MODE
    | runtime::ENV_EXPANSION
    | runtime::FORMAT_FALLBACK;

/// All valid startup flags combined  
const ALL_STARTUP_FLAGS: u32 = startup::SIMD | startup::THREAD_POOL | startup::DETAILED_STATS;

/// Check if a runtime flag value contains only valid flags
///
/// # Examples
/// ```
/// use superconfig::config_flags::{self, runtime};
///
/// assert!(config_flags::is_valid_runtime_flag(runtime::STRICT_MODE));
/// assert!(config_flags::is_valid_runtime_flag(runtime::PARALLEL | runtime::STRICT_MODE));
/// assert!(!config_flags::is_valid_runtime_flag(0xFFFFFFFF)); // Invalid flag
/// ```
#[must_use]
pub const fn is_valid_runtime_flag(flags: u64) -> bool {
    // Check if all bits in flags are covered by valid runtime flags
    (flags & !ALL_RUNTIME_FLAGS) == 0
}

/// Check if a startup flag value contains only valid flags
///
/// # Examples  
/// ```
/// use superconfig::config_flags::{self, startup};
///
/// assert!(config_flags::is_valid_startup_flag(startup::SIMD));
/// assert!(config_flags::is_valid_startup_flag(startup::SIMD | startup::THREAD_POOL));
/// assert!(!config_flags::is_valid_startup_flag(0xFFFFFFFF)); // Invalid flag
/// ```
#[must_use]
pub const fn is_valid_startup_flag(flags: u32) -> bool {
    // Check if all bits in flags are covered by valid startup flags
    (flags & !ALL_STARTUP_FLAGS) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_flag_constants() {
        assert_eq!(startup::SIMD, 1);
        assert_eq!(startup::THREAD_POOL, 2);
        assert_eq!(startup::DETAILED_STATS, 4);

        // Ensure flags are unique (can be combined with |)
        let combined = startup::SIMD | startup::THREAD_POOL;
        assert_eq!(combined, 3);
    }

    #[test]
    fn test_runtime_flag_constants() {
        assert_eq!(runtime::ARRAY_MERGE, 1);
        assert_eq!(runtime::PARALLEL, 2);
        assert_eq!(runtime::STRICT_MODE, 4);
        assert_eq!(runtime::ENV_EXPANSION, 8);
        assert_eq!(runtime::FORMAT_FALLBACK, 16);

        // Ensure flags are unique
        let combined = runtime::ARRAY_MERGE | runtime::STRICT_MODE;
        assert_eq!(combined, 5);
    }

    #[test]
    fn test_verbosity_constants() {
        assert_eq!(verbosity::NONE, 0);
        assert_eq!(verbosity::WARN, 1);
        assert_eq!(verbosity::DEBUG, 2);
        assert_eq!(verbosity::TRACE, 3);
    }

    #[test]
    fn test_verbosity_level_enum() {
        assert_eq!(VerbosityLevel::None as u8, 0);
        assert_eq!(VerbosityLevel::Warn as u8, 1);
        assert_eq!(VerbosityLevel::Debug as u8, 2);
        assert_eq!(VerbosityLevel::Trace as u8, 3);
    }

    #[test]
    fn test_verbosity_level_conversions() {
        // u8 to VerbosityLevel
        assert_eq!(VerbosityLevel::from(0), VerbosityLevel::None);
        assert_eq!(VerbosityLevel::from(1), VerbosityLevel::Warn);
        assert_eq!(VerbosityLevel::from(2), VerbosityLevel::Debug);
        assert_eq!(VerbosityLevel::from(3), VerbosityLevel::Trace);
        assert_eq!(VerbosityLevel::from(99), VerbosityLevel::None); // Invalid -> None

        // VerbosityLevel to u8
        assert_eq!(u8::from(VerbosityLevel::None), 0);
        assert_eq!(u8::from(VerbosityLevel::Warn), 1);
        assert_eq!(u8::from(VerbosityLevel::Debug), 2);
        assert_eq!(u8::from(VerbosityLevel::Trace), 3);
    }

    #[test]
    fn test_verbosity_level_display() {
        assert_eq!(format!("{}", VerbosityLevel::None), "none");
        assert_eq!(format!("{}", VerbosityLevel::Warn), "warn");
        assert_eq!(format!("{}", VerbosityLevel::Debug), "debug");
        assert_eq!(format!("{}", VerbosityLevel::Trace), "trace");
    }

    #[test]
    fn test_flag_combinations() {
        // Test that startup and runtime can use same bit positions without conflict
        assert_eq!(startup::SIMD, 1); // u32
        assert_eq!(runtime::ARRAY_MERGE, 1); // u64

        // They're different types so no collision
        let startup_flags = startup::SIMD | startup::THREAD_POOL;
        let runtime_flags = runtime::ARRAY_MERGE | runtime::PARALLEL;

        assert_eq!(startup_flags, 3u32);
        assert_eq!(runtime_flags, 3u64);
    }

    #[test]
    fn test_error_display() {
        let error = FlagError::ImmutableStartupFlag;
        assert!(format!("{error}").contains("immutable"));

        let error = FlagError::InvalidVerbosity { level: 99 };
        assert!(format!("{error}").contains("99"));
        assert!(format!("{error}").contains("0-3"));
    }

    #[test]
    fn test_runtime_flag_validation() {
        // Valid individual flags
        assert!(is_valid_runtime_flag(runtime::ARRAY_MERGE));
        assert!(is_valid_runtime_flag(runtime::PARALLEL));
        assert!(is_valid_runtime_flag(runtime::STRICT_MODE));
        assert!(is_valid_runtime_flag(runtime::ENV_EXPANSION));
        assert!(is_valid_runtime_flag(runtime::FORMAT_FALLBACK));

        // Valid combinations
        assert!(is_valid_runtime_flag(
            runtime::STRICT_MODE | runtime::PARALLEL
        ));
        assert!(is_valid_runtime_flag(
            runtime::ARRAY_MERGE | runtime::ENV_EXPANSION
        ));

        // All flags combined should be valid
        assert!(is_valid_runtime_flag(ALL_RUNTIME_FLAGS));

        // Invalid flags
        assert!(!is_valid_runtime_flag(0xFFFFFFFF)); // Invalid flag
        assert!(!is_valid_runtime_flag(1 << 10)); // Unused bit
        assert!(!is_valid_runtime_flag(0xFF00)); // High bits

        // Zero should be valid (no flags set)
        assert!(is_valid_runtime_flag(0));
    }

    #[test]
    fn test_startup_flag_validation() {
        // Valid individual flags
        assert!(is_valid_startup_flag(startup::SIMD));
        assert!(is_valid_startup_flag(startup::THREAD_POOL));
        assert!(is_valid_startup_flag(startup::DETAILED_STATS));

        // Valid combinations
        assert!(is_valid_startup_flag(startup::SIMD | startup::THREAD_POOL));
        assert!(is_valid_startup_flag(
            startup::SIMD | startup::DETAILED_STATS
        ));

        // All flags combined should be valid
        assert!(is_valid_startup_flag(ALL_STARTUP_FLAGS));

        // Invalid flags
        assert!(!is_valid_startup_flag(0xFFFFFFFF)); // Invalid flag
        assert!(!is_valid_startup_flag(1 << 10)); // Unused bit
        assert!(!is_valid_startup_flag(0xFF00)); // High bits

        // Zero should be valid (no flags set)
        assert!(is_valid_startup_flag(0));
    }
}
