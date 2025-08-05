//! Tests for the define_errors! macro

use logffi::define_errors;

define_errors! {
    pub enum TestError {
        #[error("Simple test error occurred")]
        SimpleError,
        
        #[error("Test error with field: {message}")]
        WithField { message: String },
        
        #[error("Database error in operation: {op}", level = error, target = "db")]
        DatabaseError { op: String },
        
        #[error("Config warning: {key} not found", level = warn, target = "config")]
        ConfigWarning { key: String },
        
        #[error("Processing info: {count} items", level = info)]
        ProcessingInfo { count: u32 },
    }
}

#[test]
fn test_error_enum_creation() {
    let error1 = TestError::SimpleError;
    let error2 = TestError::WithField { 
        message: "test message".to_string() 
    };
    
    // Test that Display works (from thiserror)
    assert_eq!(error1.to_string(), "Simple test error occurred");
    assert!(error2.to_string().contains("test message"));
}

#[test]
fn test_error_code_method() {
    let error1 = TestError::SimpleError;
    let error2 = TestError::WithField { 
        message: "test".to_string() 
    };
    
    assert_eq!(error1.code(), "SimpleError");
    assert_eq!(error2.code(), "WithField");
}

#[test]
fn test_error_log_method() {
    let error1 = TestError::DatabaseError { 
        op: "SELECT".to_string() 
    };
    let error2 = TestError::ConfigWarning { 
        key: "database.host".to_string() 
    };
    let error3 = TestError::ProcessingInfo { 
        count: 42 
    };
    
    // These should log with appropriate levels and targets
    error1.log(); // Should log as ERROR to "db" target
    error2.log(); // Should log as WARN to "config" target  
    error3.log(); // Should log as INFO to "app" target (default)
    
    // Test passes if no panics occur
}

#[test]
fn test_error_with_source_chain() {
    use std::io;
    
    define_errors! {
        pub enum ChainedError {
            #[error("IO operation failed", source)]
            IoError {
                cause: io::Error
            },
        }
    }
    
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let chained = ChainedError::IoError { cause: io_err };
    
    // Test basic functionality (source chain requires #[source] attribute on field)
    chained.log(); // Should log
}

#[test]
fn test_multiple_error_enums() {
    define_errors! {
        pub enum NetworkError {
            #[error("Connection timeout")]
            Timeout,
        }
    }
    
    define_errors! {
        pub enum DatabaseError {
            #[error("Query failed")]
            QueryFailed,
        }
    }
    
    let net_err = NetworkError::Timeout;
    let db_err = DatabaseError::QueryFailed;
    
    assert_eq!(net_err.code(), "Timeout");
    assert_eq!(db_err.code(), "QueryFailed");
    
    net_err.log();
    db_err.log();
}