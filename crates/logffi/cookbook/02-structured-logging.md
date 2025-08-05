# Structured Logging with Tracing via LogFFI

This guide shows how to use tracing's structured logging capabilities through LogFFI to add rich, machine-readable metadata to your log messages. Structured logging is a core tracing feature that's essential for modern observability platforms and makes your logs searchable, filterable, and analyzable.

**What LogFFI provides:** Direct access to tracing's structured logging syntax through LogFFI macros, plus convenient re-exports of tracing-subscriber components for configuration.

**What tracing provides:** The structured logging implementation, field handling, JSON formatting, and integration with observability platforms.

## Table of Contents

- [What is Structured Logging?](#what-is-structured-logging)
- [Basic Structured Syntax](#basic-structured-syntax)
- [Field Types and Values](#field-types-and-values)
- [Real-World Examples](#real-world-examples)
- [JSON Output Configuration](#json-output-configuration)
- [Best Practices](#best-practices)
- [Integration with Monitoring Platforms](#integration-with-monitoring-platforms)
- [Performance Considerations](#performance-considerations)

## What is Structured Logging?

Structured logging adds key-value pairs (fields) to your log messages, making them machine-readable and searchable:

```rust
use logffi::info;

fn main() {
    // Traditional logging - hard to parse
    info!("User alice logged in from 192.168.1.1 with session sess_123");
    
    // Structured logging - machine-readable fields
    info!(
        user_id = "alice",
        ip_address = "192.168.1.1", 
        session_id = "sess_123",
        "User login successful"
    );
}
```

**Output comparison:**

```
# Traditional
2024-01-15T10:30:00Z INFO User alice logged in from 192.168.1.1 with session sess_123

# Structured  
2024-01-15T10:30:00Z INFO User login successful user_id="alice" ip_address="192.168.1.1" session_id="sess_123"
```

## Basic Structured Syntax

LogFFI provides access to tracing's structured logging syntax in all logging macros:

```rust
use logffi::{error, warn, info, debug, trace};

fn main() {
    // Basic structure: fields followed by message
    info!(
        field_name = field_value,
        another_field = another_value,
        "Your log message here"
    );
    
    // All log levels support structured syntax
    error!(
        error_code = 500,
        component = "database",
        "Database connection failed"
    );
    
    warn!(
        memory_usage = 85.7,
        threshold = 80.0,
        "Memory usage approaching limit"
    );
    
    debug!(
        query_time_ms = 45,
        table = "users",
        rows_returned = 150,
        "Database query completed"
    );
    
    trace!(
        function = "process_request",
        line = 42,
        "Entering function"
    );
}
```

## Field Types and Values

Tracing's structured logging (accessed through LogFFI) supports various data types:

```rust
use logffi::info;

fn main() {
    info!(
        // String fields
        username = "alice",
        operation = "login",
        
        // Numeric fields
        user_id = 12345,
        attempt_count = 3,
        success_rate = 98.5,
        
        // Boolean fields
        is_admin = true,
        mfa_enabled = false,
        
        // Complex expressions (evaluated once)
        timestamp = chrono::Utc::now().timestamp(),
        request_size = calculate_request_size(),
        
        "User authentication completed"
    );
}

fn calculate_request_size() -> usize {
    // Some computation
    1024
}
```

## Real-World Examples

### User Authentication

```rust
use logffi::{info, warn, error};

fn handle_login(username: &str, ip: &str, user_agent: &str) -> Result<(), AuthError> {
    info!(
        username = username,
        ip_address = ip,
        user_agent = user_agent,
        auth_method = "password",
        "Login attempt started"
    );
    
    match authenticate_user(username) {
        Ok(user) => {
            info!(
                user_id = user.id,
                username = username,
                ip_address = ip,
                session_duration_hours = 24,
                mfa_enabled = user.mfa_enabled,
                "User authentication successful"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                username = username,
                ip_address = ip,
                failure_reason = "invalid_credentials",
                attempt_number = get_attempt_count(username),
                "Authentication failed"
            );
            Err(e)
        }
    }
}
```

### API Request Handling

```rust
use logffi::{info, error, debug};
use std::time::Instant;

fn handle_api_request(request: ApiRequest) -> ApiResponse {
    let start_time = Instant::now();
    let request_id = generate_request_id();
    
    info!(
        request_id = request_id,
        method = request.method,
        path = request.path,
        user_id = request.user_id,
        client_ip = request.client_ip,
        "API request received"
    );
    
    // Process request
    let result = process_request(&request);
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            info!(
                request_id = request_id,
                status_code = response.status,
                response_size_bytes = response.body.len(),
                duration_ms = duration.as_millis() as u64,
                cache_hit = response.from_cache,
                "API request completed successfully"
            );
            response
        }
        Err(e) => {
            error!(
                request_id = request_id,
                error_type = e.error_type(),
                error_code = e.code(),
                duration_ms = duration.as_millis() as u64,
                "API request failed"
            );
            ApiResponse::error(e)
        }
    }
}
```

### Payment Processing

```rust
use logffi::{info, error, warn};

fn process_payment(payment: PaymentRequest) -> PaymentResult {
    let transaction_id = generate_transaction_id();
    
    info!(
        transaction_id = transaction_id,
        customer_id = payment.customer_id,
        amount_cents = payment.amount_cents,
        currency = payment.currency,
        payment_method = payment.method.to_string(),
        "Payment processing initiated"
    );
    
    // Fraud detection
    let fraud_score = calculate_fraud_score(&payment);
    if fraud_score > 0.8 {
        warn!(
            transaction_id = transaction_id,
            customer_id = payment.customer_id,
            fraud_score = fraud_score,
            risk_factors = format!("{:?}", get_risk_factors(&payment)),
            "High fraud risk detected"
        );
    }
    
    // Process payment
    match charge_payment(&payment) {
        Ok(charge) => {
            info!(
                transaction_id = transaction_id,
                charge_id = charge.id,
                amount_cents = payment.amount_cents,
                processing_time_ms = charge.processing_time.as_millis() as u64,
                gateway_response = "approved",
                "Payment processed successfully"
            );
            PaymentResult::Success(charge)
        }
        Err(e) => {
            error!(
                transaction_id = transaction_id,
                customer_id = payment.customer_id,
                amount_cents = payment.amount_cents,
                decline_code = e.decline_code(),
                decline_reason = e.reason(),
                gateway_response = "declined",
                "Payment processing failed"
            );
            PaymentResult::Failed(e)
        }
    }
}
```

### Database Operations

```rust
use logffi::{info, warn, debug};

fn execute_query(query: &str, params: &[&str]) -> QueryResult {
    let query_id = generate_query_id();
    let start_time = Instant::now();
    
    debug!(
        query_id = query_id,
        query = query,
        param_count = params.len(),
        "Executing database query"
    );
    
    let result = database.execute(query, params);
    let duration = start_time.elapsed();
    
    match result {
        Ok(rows) => {
            info!(
                query_id = query_id,
                execution_time_ms = duration.as_millis() as u64,
                rows_affected = rows.len(),
                query_type = detect_query_type(query),
                "Query executed successfully"
            );
            
            // Warn about slow queries
            if duration.as_millis() > 1000 {
                warn!(
                    query_id = query_id,
                    execution_time_ms = duration.as_millis() as u64,
                    slow_query_threshold_ms = 1000,
                    query = query,
                    "Slow query detected"
                );
            }
            
            QueryResult::Success(rows)
        }
        Err(e) => {
            error!(
                query_id = query_id,
                execution_time_ms = duration.as_millis() as u64,
                error_code = e.code(),
                error_message = e.message(),
                query = query,
                "Query execution failed"
            );
            QueryResult::Error(e)
        }
    }
}
```

## JSON Output Configuration

For production systems, configure JSON output using tracing-subscriber components (re-exported by LogFFI for convenience):

```rust
use logffi::{info, registry, fmt, SubscriberExt, SubscriberInitExt};

fn main() {
    // Configure JSON output using tracing-subscriber (via LogFFI's re-exports)
    registry()
        .with(fmt::layer().json())
        .init();
    
    // Tracing's structured logs will now be output as JSON
    info!(
        service = "user-api",
        version = "1.2.3",
        environment = "production",
        "Service started"
    );
    
    info!(
        user_id = 12345,
        action = "profile_update",
        fields_updated = ["email", "phone"],
        "User profile updated"
    );
}
```

**JSON Output:**

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "level": "INFO",
  "message": "Service started",
  "target": "myapp",
  "service": "user-api",
  "version": "1.2.3",
  "environment": "production"
}
{
  "timestamp": "2024-01-15T10:30:01.456Z",
  "level": "INFO", 
  "message": "User profile updated",
  "target": "myapp",
  "user_id": 12345,
  "action": "profile_update",
  "fields_updated": ["email", "phone"]
}
```

## Best Practices

### Field Naming Conventions

Use consistent, searchable field names across your application:

```rust
use logffi::info;

fn good_field_naming() {
    // ✅ Good: Consistent snake_case, clear semantics
    info!(
        user_id = 12345,
        session_id = "sess_abc123",
        request_id = "req_456789",
        ip_address = "192.168.1.1",
        user_agent = "Mozilla/5.0...",
        "User session started"
    );
    
    // ✅ Good: Consistent units and formats
    info!(
        duration_ms = 145,
        memory_mb = 512,
        cpu_percent = 85.2,
        timestamp_unix = 1674123456,
        "Performance metrics"
    );
}

fn avoid_these_patterns() {
    // ❌ Avoid: Inconsistent naming
    info!(
        userId = 123,        // camelCase mixed with snake_case
        session_ID = "sess", // Mixed case styles
        ip = "192.168.1.1",  // Abbreviated vs explicit
        "Inconsistent naming"
    );
    
    // ❌ Avoid: Values that change constantly (high cardinality)
    info!(
        exact_timestamp_nanoseconds = std::time::SystemTime::now(),
        random_uuid = uuid::Uuid::new_v4(),
        "High cardinality fields can overwhelm monitoring systems"
    );
}
```

### Structured vs Simple Messages

Choose the right approach for each situation:

```rust
use logffi::{info, debug};

fn when_to_use_structured() {
    // ✅ Use structured logging for business events
    info!(
        user_id = 12345,
        order_id = "order_789",
        amount_cents = 2999,
        payment_method = "credit_card",
        "Order completed successfully"
    );
    
    // ✅ Use structured logging for metrics and measurements
    info!(
        endpoint = "/api/users",
        response_time_ms = 45,
        status_code = 200,
        cache_hit = true,
        "API request processed"
    );
    
    // ✅ Use simple logging for development/debugging
    debug!("Entering user validation function");
    debug!("Cache miss for key: user_12345");
    
    // ✅ Use simple logging for straightforward messages
    info!("Application startup completed");
    info!("Database migration finished");
}
```

### Field Value Types

Choose appropriate types for your field values:

```rust
use logffi::info;

fn appropriate_field_types() {
    // ✅ Good: Use appropriate numeric types
    info!(
        user_id = 12345_u64,           // User IDs as integers
        amount_cents = 2999_u32,       // Money in smallest unit
        success_rate = 98.5_f64,       // Percentages as floats
        retry_count = 3_u8,            // Small counters as small ints
        "Transaction processed"
    );
    
    // ✅ Good: Use strings for identifiers and enums
    info!(
        transaction_id = "txn_abc123",  // External IDs as strings
        status = "completed",           // Status/enum values as strings
        payment_method = "visa",        // Categories as strings
        "Payment status update"
    );
    
    // ✅ Good: Use booleans for flags
    info!(
        is_premium_user = true,
        mfa_enabled = false,
        terms_accepted = true,
        "User profile loaded"
    );
}
```

## Integration with Monitoring Platforms

### ELK Stack (Elasticsearch, Logstash, Kibana)

```rust
use logffi::{info, registry, fmt, SubscriberExt, SubscriberInitExt};

fn setup_for_elk() {
    registry()
        .with(fmt::layer().json())
        .init();
    
    // These structured logs work perfectly with ELK
    info!(
        "@timestamp" = chrono::Utc::now().to_rfc3339(),
        service = "user-service",
        environment = "production",
        user_id = 12345,
        action = "login",
        ip_address = "192.168.1.1",
        "User login event"
    );
}
```

### Grafana Loki

```rust
use logffi::{info, registry, fmt, SubscriberExt, SubscriberInitExt};

fn setup_for_loki() {
    registry()
        .with(fmt::layer().json())
        .init();
    
    // Loki labels come from consistent field names
    info!(
        service = "api-gateway",
        environment = "prod",
        region = "us-east-1",
        status_code = 200,
        method = "GET",
        "HTTP request processed"
    );
}
```

### Datadog

```rust
use logffi::info;

fn datadog_integration() {
    // Datadog automatically parses structured logs
    info!(
        service = "checkout-service",
        env = "production",
        version = "1.0.0",
        user_id = 12345,
        order_total = 29.99,
        currency = "USD",
        dd_trace_id = "1234567890",  // Datadog trace correlation
        "Order processed successfully"
    );
}
```

## Performance Considerations

### Field Evaluation

Fields are only evaluated if the log level is enabled:

```rust
use logffi::{debug, info};

fn performance_considerations() {
    // ✅ Efficient: Simple values are very fast
    info!(
        user_id = 12345,
        action = "login",
        "Fast structured logging"
    );
    
    // ✅ Efficient: Expressions only evaluated if log level enabled
    debug!(
        expensive_calculation = expensive_computation(),
        cache_stats = get_cache_statistics(),
        "Debug information"
    );
    // If debug logging is disabled, expensive functions aren't called
}

fn expensive_computation() -> String {
    // This only runs if debug logging is enabled
    format!("Expensive result: {}", complex_calculation())
}

fn complex_calculation() -> i32 {
    // Some expensive operation
    42
}
```

### High-Volume Logging

For high-throughput applications, be mindful of field cardinality:

```rust
use logffi::{info, debug};

fn high_volume_logging() {
    // ✅ Good: Low cardinality fields
    info!(
        service = "api-gateway",      // ~1 value
        environment = "production",   // ~3-4 values  
        method = "GET",              // ~7 values
        status_code = 200,           // ~20 values
        endpoint_category = "users", // ~10 values
        "Request processed"
    );
    
    // ⚠️ Be careful: High cardinality fields in high volume
    debug!(
        request_id = generate_uuid(),    // Unique every time
        user_specific_data = user_data,  // Varies per user
        timestamp_nanos = precise_time(), // Changes constantly
        "Detailed request info"
    );
    // Use these sparingly or at debug/trace levels
}
```

## Common Patterns

### Error Context

```rust
use logffi::error;

fn handle_database_error(err: DatabaseError, context: &QueryContext) {
    error!(
        error_type = "database_error",
        error_code = err.code(),
        error_message = err.message(),
        query_id = context.query_id,
        table_name = context.table,
        operation = context.operation,
        retry_count = context.retries,
        "Database operation failed"
    );
}
```

### Request Tracing

```rust
use logffi::{info, debug};

fn trace_request_lifecycle(request: &Request) {
    let request_id = request.id();
    
    info!(
        request_id = request_id,
        method = request.method(),
        path = request.path(),
        user_id = request.user_id(),
        "Request started"
    );
    
    debug!(
        request_id = request_id,
        headers_count = request.headers().len(),
        body_size_bytes = request.body().len(),
        "Request details"
    );
    
    // ... process request ...
    
    info!(
        request_id = request_id,
        status_code = 200,
        response_size_bytes = 1024,
        duration_ms = 45,
        "Request completed"
    );
}
```

## Next Steps

Now that you understand structured logging, explore these related topics:

- **[Error Handling](03-error-handling.md)** - Combine structured logging with `define_errors!`
- **[Spans and Instrumentation](04-spans-instrumentation.md)** - Add structured context to spans
- **[Advanced Tracing Integration](05-tracing-integration.md)** - OpenTelemetry and distributed tracing

## Troubleshooting

### Common Issues

**Q: My structured fields aren't appearing in the output**

```rust
// Make sure you're using the correct syntax - fields come BEFORE the message
info!(field = value, "message");  // ✅ Correct
info!("message", field = value);  // ❌ Wrong - this won't work
```

**Q: Fields have unexpected values**

```rust
// Check field types - strings need quotes in output
info!(status = "active");     // Output: status="active"  
info!(user_id = 12345);       // Output: user_id=12345
info!(is_valid = true);       // Output: is_valid=true
```

**Q: JSON output is not working**

```rust
// Make sure to set up JSON formatting before logging
use logffi::{registry, fmt, SubscriberExt, SubscriberInitExt};

registry()
    .with(fmt::layer().json())
    .init();
    
// Now structured logs will be JSON formatted
```
