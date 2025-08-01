//! Error types for the `SuperConfig` V2 registry system

use crate::config_flags::FlagError;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Unique identifier for configuration handles
pub type HandleId = u64;

/// FFI-compatible error struct with timestamp for comprehensive error tracking
///
/// This struct is designed for cross-language compatibility and provides rich
/// error context including timing information for debugging and monitoring.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuperConfigError {
    /// Error code for programmatic handling (see error code constants)
    pub code: u32,
    /// Human-readable error message
    pub message: String,
    /// Additional context (operation name, parameters, etc.)
    pub context: String,
    /// Unix timestamp in milliseconds when error occurred
    pub timestamp: u64,
}

impl SuperConfigError {
    /// Create a new `SuperConfigError` with current timestamp
    pub fn new(code: u32, message: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: context.into(),
            #[allow(clippy::cast_possible_truncation)] // Intentional for FFI compatibility
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Create error for invalid runtime flags
    #[must_use]
    pub fn invalid_runtime_flag(flags: u64) -> Self {
        Self::new(
            ERROR_INVALID_RUNTIME_FLAG,
            "Invalid runtime flag value",
            format!("flags: 0x{flags:X}"),
        )
    }

    /// Create error for invalid startup flags
    #[must_use]
    pub fn invalid_startup_flag(flags: u32) -> Self {
        Self::new(
            ERROR_INVALID_STARTUP_FLAG,
            "Invalid startup flag value",
            format!("flags: 0x{flags:X}"),
        )
    }

    /// Create error for handle not found
    #[must_use]
    pub fn handle_not_found(handle_id: HandleId) -> Self {
        Self::new(
            ERROR_HANDLE_NOT_FOUND,
            "Handle not found in registry",
            format!("handle_id: {handle_id}"),
        )
    }

    /// Create error for wrong handle type
    #[must_use]
    pub fn wrong_type(handle_id: HandleId, expected: &str, found: &str) -> Self {
        Self::new(
            ERROR_WRONG_TYPE,
            "Handle has wrong type",
            format!("handle_id: {handle_id}, expected: {expected}, found: {found}"),
        )
    }
}

impl std::fmt::Display for SuperConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Error {}: {} - {}",
            self.timestamp, self.code, self.message, self.context
        )
    }
}

impl std::error::Error for SuperConfigError {}

/// Error code constants for programmatic error handling
///
/// These codes are stable across versions and safe to use in FFI scenarios.
/// Each range represents a different category of errors.
/// Flag validation errors (1000-1099)
pub const ERROR_INVALID_RUNTIME_FLAG: u32 = 1001;
/// Error code for invalid startup flag values
pub const ERROR_INVALID_STARTUP_FLAG: u32 = 1002;
/// Error code for invalid verbosity levels
pub const ERROR_INVALID_VERBOSITY: u32 = 1003;
/// Error code for attempting to modify immutable startup flags
pub const ERROR_IMMUTABLE_STARTUP_FLAG: u32 = 1004;

/// Handle operation errors (1100-1199)  
pub const ERROR_HANDLE_NOT_FOUND: u32 = 1101;
/// Error code for handle type mismatches
pub const ERROR_WRONG_TYPE: u32 = 1102;
/// Error code for invalidated handles
pub const ERROR_INVALID_HANDLE: u32 = 1103;

/// Registry operation errors (1200-1299)
pub const ERROR_REGISTRY_FULL: u32 = 1201;
/// Error code for serialization failures
pub const ERROR_SERIALIZATION: u32 = 1202;

/// Errors that can occur during registry operations
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Handle not found in registry
    #[error("Handle {handle_id} not found in registry")]
    HandleNotFound {
        /// The handle ID that was not found
        handle_id: HandleId,
    },

    /// Handle has wrong type
    #[error("Handle {handle_id} has wrong type: expected {expected}, found {found}")]
    WrongType {
        /// The handle ID with wrong type
        handle_id: HandleId,
        /// Expected type name
        expected: &'static str,
        /// Found type name
        found: &'static str,
    },

    /// Handle has been invalidated
    #[error("Handle {handle_id} has been invalidated")]
    InvalidHandle {
        /// The invalidated handle ID
        handle_id: HandleId,
    },

    /// Registry is at capacity
    #[error("Registry is at maximum capacity")]
    RegistryFull,

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Error message
        message: String,
    },

    /// Flag operation error
    #[error("Flag operation failed: {0}")]
    FlagError(#[from] FlagError),
}

/// Error type for fluent API error collection patterns
///
/// Uses `SuperConfigError` for FFI compatibility and consistent error handling
/// across all `SuperConfig` operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FluentError {
    /// The underlying `SuperConfig` error
    pub error: SuperConfigError,
    /// Which operation caused this error (e.g., "enable", "disable", "`set_verbosity`")
    pub operation: String,
}

impl FluentError {
    /// Create a new `FluentError` from a `SuperConfigError` and operation name
    pub fn new(error: SuperConfigError, operation: impl Into<String>) -> Self {
        Self {
            error,
            operation: operation.into(),
        }
    }

    /// Create `FluentError` for registry operations
    pub fn from_registry_error(error: RegistryError, operation: impl Into<String>) -> Self {
        let super_error = match error {
            RegistryError::HandleNotFound { handle_id } => {
                SuperConfigError::handle_not_found(handle_id)
            }
            RegistryError::WrongType {
                handle_id,
                expected,
                found,
            } => SuperConfigError::wrong_type(handle_id, expected, found),
            RegistryError::InvalidHandle { handle_id } => SuperConfigError::new(
                ERROR_INVALID_HANDLE,
                "Handle has been invalidated",
                format!("handle_id: {handle_id}"),
            ),
            RegistryError::RegistryFull => {
                SuperConfigError::new(ERROR_REGISTRY_FULL, "Registry is at maximum capacity", "")
            }
            RegistryError::SerializationError { message } => {
                SuperConfigError::new(ERROR_SERIALIZATION, "Serialization error", message)
            }
            RegistryError::FlagError(flag_error) => Self::convert_flag_error(&flag_error),
        };
        Self::new(super_error, operation)
    }

    /// Convert `FlagError` to `SuperConfigError`
    fn convert_flag_error(flag_error: &FlagError) -> SuperConfigError {
        match flag_error {
            FlagError::ImmutableStartupFlag => SuperConfigError::new(
                ERROR_IMMUTABLE_STARTUP_FLAG,
                "Cannot modify startup flags at runtime",
                "",
            ),
            FlagError::InvalidVerbosity { level } => SuperConfigError::new(
                ERROR_INVALID_VERBOSITY,
                "Invalid verbosity level",
                format!("level: {level} (valid range: 0-3)"),
            ),
            FlagError::InvalidFlag { flags } => SuperConfigError::invalid_runtime_flag(*flags),
            FlagError::InvalidRuntimeFlag { flag } => SuperConfigError::invalid_runtime_flag(*flag),
            FlagError::InvalidStartupFlag { flag } => SuperConfigError::invalid_startup_flag(*flag),
        }
    }
}

impl std::fmt::Display for FluentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation '{}' failed: {}", self.operation, self.error)
    }
}

impl std::error::Error for FluentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = RegistryError::HandleNotFound { handle_id: 123 };
        assert!(format!("{}", error).contains("123"));
        assert!(format!("{}", error).contains("not found"));

        let error = RegistryError::WrongType {
            handle_id: 456,
            expected: "String",
            found: "i32",
        };
        assert!(format!("{}", error).contains("456"));
        assert!(format!("{}", error).contains("String"));
        assert!(format!("{}", error).contains("i32"));
    }

    #[test]
    fn test_error_from_flag_error() {
        let flag_error = FlagError::InvalidVerbosity { level: 99 };
        let registry_error: RegistryError = flag_error.into();

        match registry_error {
            RegistryError::FlagError(inner) => match inner {
                FlagError::InvalidVerbosity { level } => assert_eq!(level, 99),
                _ => panic!("Wrong flag error type"),
            },
            _ => panic!("Wrong registry error type"),
        }
    }
}
