//! Tests for thiserror integration
//!
//! Verifies that define_errors! macro properly integrates with thiserror,
//! preserving all expected Error trait functionality.

use logfusion::define_errors;
use std::error::Error;
use std::io;

#[test]
fn thiserror_display_trait() {
    define_errors! {
        pub enum DisplayTestError {
            #[error("Simple error message")]
            SimpleError,

            #[error("Error with field: {value}")]
            WithField { value: String },

            #[error("Multiple fields: {name} = {value} (count: {count})")]
            MultipleFields {
                name: String,
                value: String,
                count: u32,
            },

            #[error("Number formatting: {percentage:.2}%")]
            NumberFormatting { percentage: f64 },

            #[error("Debug formatting: {items:?}")]
            DebugFormatting { items: Vec<String> },
        }
    }

    // Test simple error
    let simple = DisplayTestError::SimpleError;
    assert_eq!(simple.to_string(), "Simple error message");

    // Test field interpolation
    let with_field = DisplayTestError::WithField {
        value: "test_value".to_string(),
    };
    assert_eq!(with_field.to_string(), "Error with field: test_value");

    // Test multiple fields
    let multiple = DisplayTestError::MultipleFields {
        name: "config_key".to_string(),
        value: "invalid_value".to_string(),
        count: 42,
    };
    assert_eq!(
        multiple.to_string(),
        "Multiple fields: config_key = invalid_value (count: 42)"
    );

    // Test number formatting
    let number_fmt = DisplayTestError::NumberFormatting {
        percentage: 75.456789,
    };
    assert_eq!(number_fmt.to_string(), "Number formatting: 75.46%");

    // Test debug formatting
    let debug_fmt = DisplayTestError::DebugFormatting {
        items: vec!["item1".to_string(), "item2".to_string()],
    };
    assert_eq!(
        debug_fmt.to_string(),
        r#"Debug formatting: ["item1", "item2"]"#
    );
}

#[test]
fn thiserror_debug_trait() {
    define_errors! {
        pub enum DebugTestError {
            #[error("Debug test error")]
            TestError { field: String },
        }
    }

    let error = DebugTestError::TestError {
        field: "test".to_string(),
    };

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TestError"));
    assert!(debug_str.contains("field"));
    assert!(debug_str.contains("test"));
}

#[test]
fn thiserror_error_trait() {
    define_errors! {
        pub enum ErrorTraitTest {
            #[error("Primary error")]
            Primary,

            #[error("Error with source")]
            WithSource {
                #[source]
                source: io::Error,
            },

            #[error("Nested source chain")]
            NestedChain {
                details: String,
                #[source]
                source: Box<dyn Error + Send + Sync>,
            },
        }
    }

    // Test Error trait implementation exists
    let primary = ErrorTraitTest::Primary;
    let _: &dyn Error = &primary; // Compile-time check

    // Test source() method returns None for errors without source
    assert!(primary.source().is_none());

    // Test source() method with direct source
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let with_source = ErrorTraitTest::WithSource { source: io_err };

    assert!(with_source.source().is_some());
    let source = with_source.source().unwrap();
    assert_eq!(source.to_string(), "File not found");

    // Test nested error chain
    let inner_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let nested = ErrorTraitTest::NestedChain {
        details: "Operation failed".to_string(),
        source: Box::new(inner_err),
    };

    assert!(nested.source().is_some());
    assert_eq!(nested.source().unwrap().to_string(), "Access denied");

    // Test error chain walking
    let mut current: &dyn Error = &nested;
    let mut chain_count = 0;

    while let Some(source) = current.source() {
        chain_count += 1;
        current = source;
    }

    assert_eq!(chain_count, 1); // One level of source
}

#[test]
fn thiserror_source_chaining_complex() {
    // Create a custom error type for testing
    #[derive(Debug)]
    struct CustomError {
        message: String,
    }

    impl std::fmt::Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Custom: {}", self.message)
        }
    }

    impl Error for CustomError {}

    define_errors! {
        pub enum ComplexChainError {
            #[error("Application error: {context}")]
            AppError {
                context: String,
                #[source]
                source: io::Error,
            },

            #[error("Service error")]
            ServiceError {
                #[source]
                source: Box<dyn Error + Send + Sync>,
            },
        }
    }

    // Create a chain: ComplexChainError -> CustomError -> io::Error
    let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    let custom_err = CustomError {
        message: "Database connection failed".to_string(),
    };

    // First level: io::Error directly
    let app_error = ComplexChainError::AppError {
        context: "During startup".to_string(),
        source: io_err,
    };

    // Test the full error message
    assert_eq!(app_error.to_string(), "Application error: During startup");

    // Test source access
    let source = app_error.source().unwrap();
    assert_eq!(source.to_string(), "Connection refused");

    // Second level: Boxed custom error
    let service_error = ComplexChainError::ServiceError {
        source: Box::new(custom_err),
    };

    assert_eq!(service_error.to_string(), "Service error");
    assert_eq!(
        service_error.source().unwrap().to_string(),
        "Custom: Database connection failed"
    );
}

#[test]
fn thiserror_field_types_comprehensive() {
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;

    define_errors! {
        pub enum FieldTypeTest {
            #[error("String field: '{text}'")]
            StringField { text: String },

            #[error("Number fields: i32={int}, u64={uint}, f64={float}")]
            NumberFields {
                int: i32,
                uint: u64,
                float: f64,
            },

            #[error("Bool field: {flag}")]
            BoolField { flag: bool },

            #[error("Collection field: {items:?}")]
            CollectionField { items: Vec<i32> },

            #[error("Map field: {data:?}")]
            MapField { data: HashMap<String, String> },

            #[error("JSON field: {json}")]
            JsonField { json: JsonValue },

            #[error("Optional field: {maybe:?}")]
            OptionalField { maybe: Option<String> },
        }
    }

    // Test string field
    let str_test = FieldTypeTest::StringField {
        text: "hello world".to_string(),
    };
    assert_eq!(str_test.to_string(), "String field: 'hello world'");

    // Test number fields
    let num_test = FieldTypeTest::NumberFields {
        int: -42,
        uint: 1234567890,
        float: 3.14159,
    };
    assert_eq!(
        num_test.to_string(),
        "Number fields: i32=-42, u64=1234567890, f64=3.14159"
    );

    // Test bool field
    let bool_test = FieldTypeTest::BoolField { flag: true };
    assert_eq!(bool_test.to_string(), "Bool field: true");

    // Test collection field
    let collection_test = FieldTypeTest::CollectionField {
        items: vec![1, 2, 3],
    };
    assert_eq!(collection_test.to_string(), "Collection field: [1, 2, 3]");

    // Test map field
    let mut map = HashMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());
    let map_test = FieldTypeTest::MapField { data: map };
    let map_str = map_test.to_string();
    assert!(map_str.starts_with("Map field: {"));
    assert!(map_str.contains("key1"));
    assert!(map_str.contains("value1"));

    // Test JSON field
    let json_test = FieldTypeTest::JsonField {
        json: serde_json::json!({"name": "test", "count": 42}),
    };
    let json_str = json_test.to_string();
    assert!(json_str.contains("JSON field:"));
    assert!(json_str.contains("name"));
    assert!(json_str.contains("test"));

    // Test optional field - Some case
    let opt_some = FieldTypeTest::OptionalField {
        maybe: Some("present".to_string()),
    };
    assert_eq!(opt_some.to_string(), r#"Optional field: Some("present")"#);

    // Test optional field - None case
    let opt_none = FieldTypeTest::OptionalField { maybe: None };
    assert_eq!(opt_none.to_string(), "Optional field: None");
}

#[test]
fn thiserror_with_logging_integration() {
    define_errors! {
        pub enum LogIntegrationTest {
            #[error("Field interpolation: {value}")]
            FieldInterpolation { value: String },
        }
    }

    let error = LogIntegrationTest::FieldInterpolation {
        value: "interpolated_value".to_string(),
    };

    // Test that our .code() method works
    assert_eq!(error.code(), "FieldInterpolation");

    // Test that .to_string() gives interpolated message
    assert_eq!(error.to_string(), "Field interpolation: interpolated_value");

    // Test that .log() works (should not panic)
    error.log();

    // Test that Error trait methods work
    assert!(error.source().is_none());
}
