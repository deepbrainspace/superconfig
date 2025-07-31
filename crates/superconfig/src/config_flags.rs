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
        assert!(format!("{}", error).contains("immutable"));

        let error = FlagError::InvalidVerbosity { level: 99 };
        assert!(format!("{}", error).contains("99"));
        assert!(format!("{}", error).contains("0-3"));
    }
}
