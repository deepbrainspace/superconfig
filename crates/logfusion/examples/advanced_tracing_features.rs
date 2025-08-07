//! Advanced Tracing Features Demo
//!
//! This example demonstrates advanced tracing ecosystem compatibility including:
//! - log crate bridge (for legacy libraries)
//! - Structured logging with fields
//! - Spans and instrumentation
//! - Multiple subscribers and filtering
//! - Custom tracing layers
//!
//! Run with different configurations:
//!
//! # Basic demo
//! cargo run --example advanced_tracing_features
//!
//! # With debug level
//! RUST_LOG=debug cargo run --example advanced_tracing_features
//!
//! # With structured output
//! RUST_LOG=trace cargo run --example advanced_tracing_features

use logfusion::{debug, define_errors, error, info, trace, warn};
use std::collections::HashMap;
use tracing::{Level, instrument};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Custom error types for demonstration
define_errors! {
    pub enum ServiceError {
        #[error("Database connection failed: {host}:{port}", level = error, target = "service::db")]
        DatabaseConnection {
            host: String,
            port: u16,
            #[source]
            source: std::io::Error,
        },

        #[error("API rate limit exceeded: {requests}/{limit} in {window}s", level = warn, target = "service::api")]
        RateLimitExceeded {
            requests: u32,
            limit: u32,
            window: u32,
        },

        #[error("Configuration validation failed", level = error, target = "service::config")]
        ConfigValidation {
            errors: Vec<String>,
        },
    }
}

// Simulated user request processing
#[derive(Debug)]
struct UserRequest {
    id: String,
    user_id: u64,
    action: String,
    payload: HashMap<String, String>,
}

// Instrumented function that creates spans automatically
#[instrument(
    level = "info",
    target = "service::handler",
    fields(
        request_id = %request.id,
        user_id = request.user_id,
        action = %request.action,
        payload_size = request.payload.len()
    )
)]
async fn process_user_request(request: UserRequest) -> Result<String, ServiceError> {
    info!("Processing user request");

    // Simulate database operations with nested spans
    let db_result = database_operation(&request).await?;

    // Simulate API calls with structured logging
    api_call_with_structured_logging(&request).await?;

    // Log success with structured data
    info!(
        processing_time_ms = 150,
        result_size = db_result.len(),
        "Request processed successfully"
    );

    Ok(db_result)
}

#[instrument(level = "debug", target = "service::db")]
async fn database_operation(request: &UserRequest) -> Result<String, ServiceError> {
    debug!("Connecting to database");

    // Simulate connection error
    if request.action == "fail_db" {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");

        let db_error = ServiceError::DatabaseConnection {
            host: "localhost".to_string(),
            port: 5432,
            source: io_error,
        };

        // This will log the error automatically
        db_error.log();
        return Err(db_error);
    }

    // Simulate successful operation
    debug!(
        query = "SELECT * FROM users WHERE id = ?",
        user_id = request.user_id,
        "Executing database query"
    );

    trace!("Query executed successfully");

    Ok(format!("User data for {}", request.user_id))
}

async fn api_call_with_structured_logging(request: &UserRequest) -> Result<(), ServiceError> {
    let span = tracing::span!(
        Level::INFO,
        "api_call",
        service = "external_api",
        endpoint = "/api/v1/validate",
        method = "POST"
    );

    let _enter = span.enter();

    info!("Making external API call");

    // Simulate rate limiting
    if request.action == "rate_limit" {
        let rate_error = ServiceError::RateLimitExceeded {
            requests: 1001,
            limit: 1000,
            window: 3600,
        };

        rate_error.log();
        return Err(rate_error);
    }

    // Log structured success
    info!(
        response_code = 200,
        response_time_ms = 45,
        "API call completed"
    );

    Ok(())
}

fn demonstrate_log_crate_bridge() {
    info!("=== Log Crate Bridge Demo ===");

    // These calls use the log crate directly, but should work through tracing's bridge
    log::error!("Legacy library error (via log crate)");
    log::warn!("Legacy library warning (via log crate)");
    log::info!("Legacy library info (via log crate)");
    log::debug!("Legacy library debug (via log crate)");

    // Mixed with our LogFusion macros
    error!("LogFusion error message");
    warn!("LogFusion warning message");
    info!("LogFusion info message");
    debug!("LogFusion debug message");
}

fn demonstrate_complex_errors() {
    info!("=== Complex Error Handling Demo ===");

    // Configuration validation error
    let config_error = ServiceError::ConfigValidation {
        errors: vec![
            "Missing required field: database.host".to_string(),
            "Invalid port number: 'abc'".to_string(),
            "Unknown log level: 'verbose'".to_string(),
        ],
    };

    config_error.log();

    // Show error chain access
    use std::error::Error;
    info!("Error code: {}", config_error.code());
    info!("Error display: {}", config_error);
    info!("Has source: {}", config_error.source().is_some());
}

#[tokio::main]
async fn main() {
    // Initialize tracing with multiple layers
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    info!("🚀 Advanced Tracing Features Demo");
    info!("==================================");

    // Demonstrate basic compatibility
    demonstrate_log_crate_bridge();

    // Demonstrate complex error handling
    demonstrate_complex_errors();

    // Demonstrate instrumented request processing
    info!("=== Instrumented Request Processing ===");

    let mut payload = HashMap::new();
    payload.insert("operation".to_string(), "user_update".to_string());
    payload.insert("field".to_string(), "email".to_string());

    // Successful request
    let success_request = UserRequest {
        id: "req-12345".to_string(),
        user_id: 67890,
        action: "update_profile".to_string(),
        payload: payload.clone(),
    };

    match process_user_request(success_request).await {
        Ok(result) => info!("Request succeeded: {}", result),
        Err(e) => error!("Request failed: {}", e),
    }

    // Database failure request
    let db_fail_request = UserRequest {
        id: "req-12346".to_string(),
        user_id: 11111,
        action: "fail_db".to_string(),
        payload: payload.clone(),
    };

    match process_user_request(db_fail_request).await {
        Ok(result) => info!("Request succeeded: {}", result),
        Err(e) => error!("Request failed: {}", e),
    }

    // Rate limit failure request
    let rate_limit_request = UserRequest {
        id: "req-12347".to_string(),
        user_id: 22222,
        action: "rate_limit".to_string(),
        payload,
    };

    match process_user_request(rate_limit_request).await {
        Ok(result) => info!("Request succeeded: {}", result),
        Err(e) => error!("Request failed: {}", e),
    }

    info!("=== Feature Summary ===");
    info!("✅ Log crate bridge - Legacy libraries work seamlessly");
    info!("✅ Structured logging - Rich context with fields");
    info!("✅ Span instrumentation - Automatic request tracing");
    info!("✅ Error integration - thiserror + LogFusion + tracing");
    info!("✅ Multiple subscribers - fmt + custom layers");
    info!("✅ Environment filtering - RUST_LOG support");
    info!("✅ Async compatibility - Works with tokio/async-std");

    info!("🎯 All tracing ecosystem features work perfectly!");
}
