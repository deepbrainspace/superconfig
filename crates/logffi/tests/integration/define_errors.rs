// Independent test for define_errors! macro

use logffi::{define_errors, error};

// Test simple variants (existing functionality)
define_errors! {
    pub enum SimpleError {
        #[error("Connection failed")]
        ConnectionFailed,

        #[error("Parse error occurred")]
        ParseError,

        #[error("Timeout")]
        Timeout,
    }
}

// Test variants with fields (new functionality)
define_errors! {
    pub enum FieldError {
        #[error("IO error occurred")]
        IoError {
            source: std::io::Error,
        },

        #[error("Network error on port {port}")]
        NetworkError {
            port: u16,
            message: String,
        },

        #[error("Parse failed")]
        ParseFailed,
    }
}

#[test]
fn test_simple_error_creation() {
    let error = SimpleError::ConnectionFailed;
    assert_eq!(error.to_string(), "Connection failed");
}

#[test]
fn test_simple_error_code() {
    let error = SimpleError::ParseError;
    assert_eq!(error.code(), "ParseError");
    assert_eq!(error.kind(), "ParseError");
}

#[test]
fn test_simple_full_message_chain() {
    let error = SimpleError::Timeout;
    assert_eq!(error.full_message_chain(), "Timeout");
}

#[test]
fn test_simple_error_logging() {
    let error = SimpleError::ConnectionFailed;
    // This should not panic
    error.log();
}

#[test]
fn test_field_error_creation() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = FieldError::IoError { source: io_error };
    assert_eq!(error.code(), "IoError");
    assert_eq!(error.kind(), "IoError");
}

#[test]
fn test_field_error_with_data() {
    let error = FieldError::NetworkError {
        port: 8080,
        message: "Connection refused".to_string(),
    };
    assert_eq!(error.code(), "NetworkError");
    assert_eq!(error.kind(), "NetworkError");
    // The actual message format depends on thiserror implementation
    assert!(error.to_string().contains("8080"));
}

#[test]
fn test_mixed_error_variants() {
    let simple_error = FieldError::ParseFailed;
    assert_eq!(simple_error.code(), "ParseFailed");

    let field_error = FieldError::NetworkError {
        port: 443,
        message: "SSL handshake failed".to_string(),
    };
    assert_eq!(field_error.code(), "NetworkError");
}

#[test]
fn test_field_error_logging() {
    let error = FieldError::NetworkError {
        port: 9000,
        message: "Timeout".to_string(),
    };
    // This should not panic
    error.log();
}
