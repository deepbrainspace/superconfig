//! Error types for the `SuperConfig` V2 registry system

use crate::config_flags::FlagError;
use thiserror::Error;

/// Unique identifier for configuration handles
pub type HandleId = u64;

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
