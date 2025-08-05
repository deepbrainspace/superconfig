//! Structured Logging Demo
//!
//! This example demonstrates LogFFI's structured logging capabilities, showing how to
//! add structured fields to your log messages for better observability and analysis.
//!
//! Structured logging allows you to attach key-value pairs to log messages, making
//! them machine-readable and perfect for log aggregation systems like ELK, Grafana,
//! or cloud logging platforms.
//!
//! Run with different configurations:
//!
//! # Basic demo
//! cargo run --example structured_logging_demo
//!
//! # With JSON output (if using tracing-subscriber)
//! RUST_LOG=debug cargo run --example structured_logging_demo
//!
//! # Focus on specific operations
//! RUST_LOG=structured_logging_demo::payment=info cargo run --example structured_logging_demo

use logffi::{debug, error, info, info_span, warn};
use serde_json::json;
use std::time::{Duration, Instant};

fn main() {
    println!("🏗️  LogFFI Structured Logging Demo");
    println!("===================================\n");

    // Basic structured logging examples
    demonstrate_basic_structured_logging();

    // Real-world scenarios
    demonstrate_user_authentication();
    demonstrate_payment_processing();
    demonstrate_api_request_handling();
    demonstrate_database_operations();
    demonstrate_spans_with_structured_data();

    println!("\n🎯 Key Benefits of Structured Logging:");
    println!("   • Machine-readable log data");
    println!("   • Easy filtering and searching");
    println!("   • Better integration with monitoring tools");
    println!("   • Consistent field names across your application");
    println!("   • Rich context for debugging and analysis");
}

fn demonstrate_basic_structured_logging() {
    println!("📊 Basic Structured Logging");
    println!("---------------------------");

    // Simple field logging
    info!(
        user_id = 12345,
        action = "login",
        "User authentication attempt"
    );

    // Multiple fields with different types
    info!(
        request_id = "req-abc-123",
        method = "POST",
        path = "/api/users",
        status_code = 201,
        duration_ms = 45,
        "API request completed"
    );

    // Numeric fields
    warn!(
        cpu_percent = 85.7,
        memory_mb = 1024,
        disk_usage_percent = 92.3,
        threshold_exceeded = true,
        "System resources under pressure"
    );

    // Complex data structures
    let user_metadata = json!({
        "role": "admin",
        "department": "engineering",
        "permissions": ["read", "write", "admin"]
    });

    info!(
        user_id = 67890,
        metadata = %user_metadata,
        session_timeout_min = 30,
        "User session created with metadata"
    );

    println!();
}

fn demonstrate_user_authentication() {
    println!("🔐 User Authentication Scenario");
    println!("--------------------------------");

    let user_id = 12345;
    let username = "alice.smith";
    let ip_address = "192.168.1.100";
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

    // Successful login
    info!(
        user_id = user_id,
        username = username,
        ip_address = ip_address,
        user_agent = user_agent,
        login_method = "password",
        mfa_enabled = true,
        "User login successful"
    );

    // Failed login attempt
    warn!(
        username = "bob.hacker",
        ip_address = "203.0.113.42",
        user_agent = "curl/7.68.0",
        failure_reason = "invalid_password",
        attempt_count = 3,
        "Failed login attempt"
    );

    // Account lockout
    error!(
        user_id = 54321,
        username = "charlie.target",
        ip_address = "198.51.100.10",
        lockout_duration_min = 15,
        failed_attempts = 5,
        "Account locked due to multiple failed attempts"
    );

    println!();
}

fn demonstrate_payment_processing() {
    println!("💳 Payment Processing Scenario");
    println!("-------------------------------");

    let transaction_id = "txn-2024-001";
    let order_id = "order-abc-123";
    let customer_id = 78901;

    // Payment initiation
    info!(
        transaction_id = transaction_id,
        order_id = order_id,
        customer_id = customer_id,
        amount_cents = 2999, // $29.99
        currency = "USD",
        payment_method = "credit_card",
        card_last_four = "4242",
        "Payment processing initiated"
    );

    // Payment validation
    debug!(
        transaction_id = transaction_id,
        fraud_score = 0.15,
        risk_level = "low",
        validation_checks_passed = 8,
        validation_checks_total = 8,
        "Payment validation completed"
    );

    // Successful payment
    info!(
        transaction_id = transaction_id,
        order_id = order_id,
        customer_id = customer_id,
        amount_cents = 2999,
        processing_time_ms = 1247,
        gateway_response = "approved",
        authorization_code = "AUTH123456",
        "Payment processed successfully"
    );

    // Payment failure scenario
    error!(
        transaction_id = "txn-2024-002",
        order_id = "order-def-456",
        customer_id = 11111,
        amount_cents = 15000,
        decline_reason = "insufficient_funds",
        gateway_code = "51",
        retry_allowed = true,
        "Payment declined"
    );

    println!();
}

fn demonstrate_api_request_handling() {
    println!("🌐 API Request Handling");
    println!("------------------------");

    let request_id = "req-2024-789";
    let start_time = Instant::now();

    // Request received
    info!(
        request_id = request_id,
        method = "GET",
        path = "/api/v1/users/12345/orders",
        user_agent = "MyApp/1.2.3",
        client_ip = "10.0.0.15",
        "API request received"
    );

    // Simulate some processing time
    std::thread::sleep(Duration::from_millis(50));

    // Database query
    debug!(
        request_id = request_id,
        query_type = "user_orders",
        user_id = 12345,
        query_duration_ms = 23,
        rows_returned = 5,
        "Database query executed"
    );

    // Response sent
    let duration = start_time.elapsed();
    info!(
        request_id = request_id,
        status_code = 200,
        response_size_bytes = 1847,
        duration_ms = duration.as_millis() as u64,
        cache_hit = false,
        "API request completed"
    );

    // Rate limiting example
    warn!(
        client_ip = "203.0.113.50",
        requests_per_minute = 105,
        rate_limit = 100,
        window_remaining_sec = 45,
        "Rate limit exceeded"
    );

    println!();
}

fn demonstrate_database_operations() {
    println!("🗄️  Database Operations");
    println!("------------------------");

    let connection_id = "conn-pool-007";
    let query_id = "query-456";

    // Connection established
    info!(
        connection_id = connection_id,
        database = "app_production",
        host = "db.example.com",
        port = 5432,
        ssl_enabled = true,
        connection_time_ms = 12,
        "Database connection established"
    );

    // Query execution
    debug!(
        connection_id = connection_id,
        query_id = query_id,
        operation = "SELECT",
        table = "users",
        where_conditions = 2,
        query = "SELECT id, name, email FROM users WHERE active = ? AND department = ?",
        "Executing database query"
    );

    // Query completed
    info!(
        connection_id = connection_id,
        query_id = query_id,
        execution_time_ms = 89,
        rows_affected = 0,
        rows_returned = 25,
        index_used = "idx_users_active_dept",
        "Query executed successfully"
    );

    // Connection pool stats
    info!(
        pool_name = "primary",
        active_connections = 8,
        idle_connections = 2,
        max_connections = 20,
        wait_queue_length = 0,
        "Connection pool status"
    );

    // Slow query warning
    warn!(
        connection_id = "conn-pool-012",
        query_id = "query-slow-001",
        execution_time_ms = 2500,
        slow_query_threshold_ms = 1000,
        table = "analytics_events",
        query_type = "aggregation",
        "Slow query detected"
    );

    println!();
}

fn demonstrate_spans_with_structured_data() {
    println!("📏 Spans with Structured Data");
    println!("------------------------------");

    // Create a span with structured fields
    let order_span = info_span!(
        "process_order",
        order_id = "order-xyz-789",
        customer_id = 99999,
        order_total_cents = 4599,
        item_count = 3
    );

    let _enter = order_span.enter();

    info!("Starting order processing");

    // Inventory check with structured data
    debug!(
        sku = "WIDGET-001",
        quantity_requested = 2,
        quantity_available = 15,
        warehouse = "WEST-01",
        "Inventory check completed"
    );

    // Payment processing within span
    info!(
        payment_method = "apple_pay",
        amount_cents = 4599,
        processing_time_ms = 340,
        "Payment processed within order"
    );

    // Shipping calculation
    info!(
        shipping_method = "express",
        shipping_cost_cents = 999,
        estimated_delivery_days = 2,
        carrier = "FastShip",
        tracking_number = "FS123456789",
        "Shipping arranged"
    );

    info!(
        total_processing_time_ms = 580,
        order_status = "confirmed",
        "Order processing completed"
    );

    // Exit span
    drop(_enter);

    println!();
}
