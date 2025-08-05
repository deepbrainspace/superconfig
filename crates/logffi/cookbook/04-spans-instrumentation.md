# Spans and Instrumentation with Tracing via LogFFI

This guide covers tracing's span functionality accessed through LogFFI, including manual span creation and the `#[instrument]` attribute for automatic function tracing. Spans provide hierarchical context for operations and are essential for distributed tracing and performance monitoring.

**What LogFFI provides:** Direct access to tracing's span macros and instrumentation features, plus convenient re-exports of tracing components.

**What tracing provides:** The span implementation, instrumentation framework, context propagation, and distributed tracing support.

## Table of Contents

- [Understanding Spans](#understanding-spans)
- [Manual Span Creation](#manual-span-creation)
- [The #[instrument] Attribute](#the-instrument-attribute)
- [Span Context and Fields](#span-context-and-fields)
- [Nested Spans and Hierarchy](#nested-spans-and-hierarchy)
- [Async Operation Tracing](#async-operation-tracing)
- [Performance Monitoring](#performance-monitoring)
- [Distributed Tracing](#distributed-tracing)
- [Best Practices](#best-practices)

## Understanding Spans

Spans represent units of work in your application and provide hierarchical context for logs and events:

```rust
use logffi::{info, span, Level};

fn process_user_request(user_id: u64, request: &str) {
    // Create a span for this operation
    let request_span = span!(Level::INFO, "process_request", user_id = user_id, request_type = request);
    let _enter = request_span.enter();
    
    info!("Processing user request");
    
    // All logging within this scope is associated with the span
    validate_request(request);
    execute_request(request);
    send_response();
    
    info!("Request processing completed");
    // Span automatically ends when _enter is dropped
}

fn validate_request(request: &str) {
    // This creates a child span
    let validation_span = span!(Level::DEBUG, "validate_request");
    let _enter = validation_span.enter();
    
    info!("Validating request format");
    // Validation logic here
    info!("Request validation passed");
}
```

## Manual Span Creation

### Basic Span Creation

```rust
use logffi::{span, info, Level};

fn basic_span_example() {
    // Create a span with a name
    let span = span!(Level::INFO, "operation_name");
    let _enter = span.enter();
    
    info!("Inside the span");
    // Span context is active here
    
    // Span ends when _enter is dropped
}

// With structured fields
fn span_with_fields() {
    let user_id = 12345;
    let operation = "user_update";
    
    let span = span!(
        Level::INFO,
        "user_operation",
        user_id = user_id,
        operation = operation,
        component = "user_service"
    );
    let _enter = span.enter();
    
    info!("Executing user operation");
    // All events here include span context
}
```

### Span Levels and Targeting

```rust
use logffi::{span, debug, info, Level};

fn span_levels_example() {
    // Different span levels for different granularities
    let service_span = span!(Level::INFO, "user_service");
    let _service_enter = service_span.enter();
    
    info!("Service operation started");
    
    {
        let method_span = span!(Level::DEBUG, "get_user_profile");
        let _method_enter = method_span.enter();
        
        debug!("Fetching user profile from database");
        // Database operations here
    }
    
    {
        let cache_span = span!(Level::TRACE, "cache_lookup");
        let _cache_enter = cache_span.enter();
        
        debug!("Checking cache for user data");
        // Cache operations here
    }
    
    info!("Service operation completed");
}

// Targeted spans for specific modules
fn targeted_spans() {
    let span = span!(
        target: "myapp::database",
        Level::INFO,
        "db_transaction",
        table = "users",
        operation = "select"
    );
    let _enter = span.enter();
    
    info!("Database transaction started");
}
```

## The #[instrument] Attribute

Tracing's `#[instrument]` attribute automatically creates spans for functions:

### Basic Instrumentation

```rust
use logffi::{info, instrument};

#[instrument]
fn process_order(order_id: u64) {
    info!("Processing order");
    // Automatically creates span: process_order{order_id=12345}
    
    validate_order_items(order_id);
    calculate_totals(order_id);
    charge_payment(order_id);
}

#[instrument]
async fn fetch_user_data(user_id: u64) -> Result<UserData, DatabaseError> {
    info!("Fetching user data from database");
    // Span: fetch_user_data{user_id=67890}
    
    let user = database::get_user(user_id).await?;
    let profile = database::get_profile(user_id).await?;
    
    Ok(UserData::new(user, profile))
}
```

### Customizing Instrumentation

```rust
use logffi::{info, instrument, Level};

// Custom span name and level
#[instrument(name = "user_auth", level = "info")]
fn authenticate_user(username: &str, password: &str) -> AuthResult {
    info!("Authenticating user");
    // Span: user_auth{username="alice", password="[redacted]"}
    // ...
}

// Skip sensitive parameters
#[instrument(skip(password, api_key))]
fn secure_operation(user_id: u64, password: &str, api_key: &str) {
    info!("Performing secure operation");
    // Span: secure_operation{user_id=123} (password and api_key not included)
}

// Custom field values
#[instrument(fields(request_id = %generate_request_id(), user_type = "premium"))]
fn handle_premium_request(user_id: u64) {
    info!("Handling premium user request");
    // Span includes generated request_id and user_type fields
}

// Return value instrumentation
#[instrument(ret)]
fn calculate_discount(user_id: u64, order_total: f64) -> f64 {
    let discount = if user_id > 1000 { 0.1 } else { 0.05 };
    order_total * discount
    // Span will include return value: calculate_discount{...} -> 9.5
}

// Error instrumentation
#[instrument(err)]
async fn risky_operation() -> Result<String, MyError> {
    // If this returns an Error, it will be recorded in the span
    Ok("success".to_string())
}
```

### Advanced Instrumentation Patterns

```rust
use logffi::{info, instrument, Level};

// Complex field expressions
#[instrument(
    name = "api_request",
    level = "info",
    fields(
        method = %request.method(),
        path = %request.path(),
        user_id = request.user_id(),
        request_size = request.content_length(),
        user_agent = %request.headers().get("user-agent").unwrap_or_default()
    )
)]
async fn handle_api_request(request: ApiRequest) -> ApiResponse {
    info!("Processing API request");
    
    // Business logic here
    ApiResponse::ok("processed")
}

// Conditional instrumentation
#[instrument(skip_all, fields(operation = "user_lookup"))]
fn lookup_user_conditionally(user_id: u64, include_sensitive: bool) {
    if include_sensitive {
        // Add sensitive fields only when needed
        tracing::Span::current().record("includes_sensitive", &true);
    }
    
    info!("Looking up user data");
}

// Multiple target specification
#[instrument(target = "myapp::service::user", level = "debug")]
fn internal_user_operation(user_id: u64) {
    info!("Internal user operation");
}
```

## Span Context and Fields

### Adding Fields to Existing Spans

```rust
use logffi::{info, span, Level};

fn dynamic_span_fields() {
    let span = span!(Level::INFO, "database_query");
    let _enter = span.enter();
    
    // Add fields dynamically
    span.record("query_type", &"SELECT");
    span.record("table", &"users");
    
    info!("Executing database query");
    
    // Add more fields as operation progresses
    let start_time = std::time::Instant::now();
    
    // ... perform query ...
    
    let duration = start_time.elapsed();
    span.record("duration_ms", &duration.as_millis());
    span.record("rows_returned", &42);
    
    info!("Query completed");
}

#[instrument(fields(query_result))]
async fn instrumented_with_result() -> Result<Vec<User>, DatabaseError> {
    info!("Starting database query");
    
    let result = perform_query().await?;
    
    // Record the result in the span
    tracing::Span::current().record("query_result", &format!("{} users found", result.len()));
    
    Ok(result)
}
```

### Span Events and Lifecycle

```rust
use logffi::{info, warn, error, instrument};

#[instrument]
async fn complex_operation(operation_id: u64) -> Result<OperationResult, OperationError> {
    info!("Operation started");
    
    // Phase 1: Validation
    info!(phase = "validation", "Starting validation phase");
    match validate_input(operation_id).await {
        Ok(_) => info!(phase = "validation", "Validation completed"),
        Err(e) => {
            error!(phase = "validation", error = %e, "Validation failed");
            return Err(e.into());
        }
    }
    
    // Phase 2: Processing
    info!(phase = "processing", "Starting processing phase");
    let processing_result = process_data(operation_id).await?;
    info!(
        phase = "processing",
        items_processed = processing_result.item_count,
        "Processing completed"
    );
    
    // Phase 3: Finalization
    info!(phase = "finalization", "Starting finalization phase");
    let final_result = finalize_operation(processing_result).await?;
    info!(
        phase = "finalization",
        final_status = %final_result.status,
        "Operation completed successfully"
    );
    
    Ok(final_result)
}
```

## Nested Spans and Hierarchy

### Creating Span Hierarchies

```rust
use logffi::{info, span, instrument, Level};

#[instrument]
async fn process_order(order: Order) -> Result<OrderResult, OrderError> {
    info!("Starting order processing");
    
    // Each step creates a child span
    let validation_result = validate_order(&order).await?;
    let payment_result = process_payment(&order).await?;
    let fulfillment_result = fulfill_order(&order).await?;
    
    info!("Order processing completed");
    Ok(OrderResult::new(validation_result, payment_result, fulfillment_result))
}

#[instrument]
async fn validate_order(order: &Order) -> Result<ValidationResult, ValidationError> {
    info!("Validating order");
    
    // These create grandchild spans
    validate_items(&order.items).await?;
    validate_shipping_address(&order.shipping_address).await?;
    validate_payment_method(&order.payment_method).await?;
    
    info!("Order validation completed");
    Ok(ValidationResult::valid())
}

#[instrument]
async fn validate_items(items: &[OrderItem]) -> Result<(), ValidationError> {
    info!(item_count = items.len(), "Validating order items");
    
    for (index, item) in items.iter().enumerate() {
        // Create a span for each item
        let item_span = span!(
            Level::DEBUG,
            "validate_item",
            item_index = index,
            product_id = item.product_id,
            quantity = item.quantity
        );
        let _enter = item_span.enter();
        
        check_inventory(item.product_id, item.quantity).await?;
        validate_item_configuration(item).await?;
    }
    
    info!("All items validated successfully");
    Ok(())
}
```

### Manual Span Hierarchy

```rust
use logffi::{info, span, Level};

fn manual_span_hierarchy() {
    // Parent span
    let parent_span = span!(Level::INFO, "parent_operation", operation_id = 123);
    let _parent_enter = parent_span.enter();
    
    info!("Parent operation started");
    
    {
        // Child span 1
        let child1_span = span!(Level::INFO, "child_operation_1", step = 1);
        let _child1_enter = child1_span.enter();
        
        info!("Child operation 1 executing");
        
        {
            // Grandchild span
            let grandchild_span = span!(Level::DEBUG, "detailed_work", work_type = "data_processing");
            let _grandchild_enter = grandchild_span.enter();
            
            info!("Performing detailed work");
        }
        
        info!("Child operation 1 completed");
    }
    
    {
        // Child span 2
        let child2_span = span!(Level::INFO, "child_operation_2", step = 2);
        let _child2_enter = child2_span.enter();
        
        info!("Child operation 2 executing");
        info!("Child operation 2 completed");
    }
    
    info!("Parent operation completed");
}
```

## Async Operation Tracing

### Async Function Instrumentation

```rust
use logffi::{info, instrument};
use tokio::time::{sleep, Duration};

#[instrument]
async fn async_web_request(url: &str) -> Result<String, reqwest::Error> {
    info!("Starting HTTP request");
    
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    
    info!(response_length = text.len(), "Request completed");
    Ok(text)
}

#[instrument]
async fn concurrent_operations(user_id: u64) -> Result<UserProfile, ServiceError> {
    info!("Starting concurrent user data fetch");
    
    // These operations run concurrently but maintain separate span contexts
    let (basic_info, preferences, activity) = tokio::try_join!(
        fetch_basic_info(user_id),
        fetch_user_preferences(user_id),
        fetch_recent_activity(user_id)
    )?;
    
    info!("All concurrent operations completed");
    Ok(UserProfile::new(basic_info, preferences, activity))
}

#[instrument]
async fn fetch_basic_info(user_id: u64) -> Result<BasicInfo, DatabaseError> {
    info!("Fetching basic user info");
    sleep(Duration::from_millis(100)).await; // Simulate database call
    Ok(BasicInfo::default())
}

#[instrument]
async fn fetch_user_preferences(user_id: u64) -> Result<Preferences, DatabaseError> {
    info!("Fetching user preferences");
    sleep(Duration::from_millis(150)).await; // Simulate database call
    Ok(Preferences::default())
}

#[instrument]
async fn fetch_recent_activity(user_id: u64) -> Result<Activity, DatabaseError> {
    info!("Fetching recent activity");
    sleep(Duration::from_millis(200)).await; // Simulate database call
    Ok(Activity::default())
}
```

### Span Context Across Await Points

```rust
use logffi::{info, span, Level};

async fn span_across_awaits() {
    let operation_span = span!(Level::INFO, "long_operation", operation_id = 456);
    let _enter = operation_span.enter();
    
    info!("Operation phase 1 starting");
    
    // Span context is maintained across await points
    let result1 = async_step_1().await;
    
    info!(phase1_result = ?result1, "Phase 1 completed");
    
    // Still in the same span context
    let result2 = async_step_2().await;
    
    info!(phase2_result = ?result2, "Phase 2 completed");
    
    // Final processing still in span context
    let final_result = combine_results(result1, result2).await;
    
    info!(final_result = ?final_result, "Operation completed");
}

async fn async_step_1() -> String {
    // This can have its own instrumentation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "step1_result".to_string()
}

async fn async_step_2() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    "step2_result".to_string()
}

async fn combine_results(r1: String, r2: String) -> String {
    format!("{}+{}", r1, r2)
}
```

## Performance Monitoring

### Timing and Metrics

```rust
use logffi::{info, instrument};
use std::time::Instant;

#[instrument(fields(duration_ms, items_processed, throughput))]
async fn monitored_batch_processing(items: Vec<Item>) -> Result<ProcessingResult, ProcessingError> {
    let start_time = Instant::now();
    let total_items = items.len();
    
    info!(total_items = total_items, "Starting batch processing");
    
    let mut processed_count = 0;
    let mut results = Vec::new();
    
    for (index, item) in items.iter().enumerate() {
        let result = process_single_item(item).await?;
        results.push(result);
        processed_count += 1;
        
        // Log progress periodically
        if index % 100 == 0 {
            info!(
                processed = processed_count,
                total = total_items,
                progress_percent = (processed_count as f64 / total_items as f64) * 100.0,
                "Batch processing progress"
            );
        }
    }
    
    let duration = start_time.elapsed();
    let throughput = processed_count as f64 / duration.as_secs_f64();
    
    // Record metrics in span
    tracing::Span::current().record("duration_ms", &duration.as_millis());
    tracing::Span::current().record("items_processed", &processed_count);
    tracing::Span::current().record("throughput", &throughput);
    
    info!(
        duration_ms = duration.as_millis(),
        items_processed = processed_count,
        throughput = throughput,
        "Batch processing completed"
    );
    
    Ok(ProcessingResult::new(results))
}

#[instrument]
async fn process_single_item(item: &Item) -> Result<ItemResult, ItemError> {
    // Individual item processing with its own span
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    Ok(ItemResult::success())
}
```

### Resource Monitoring

```rust
use logffi::{info, warn, instrument};

#[instrument(fields(
    memory_usage_mb,
    cpu_usage_percent,
    disk_usage_mb,
    network_bytes
))]
async fn resource_intensive_operation(data: LargeDataSet) -> Result<ProcessedData, ProcessingError> {
    let start_memory = get_memory_usage();
    
    info!(
        data_size_mb = data.size_mb(),
        start_memory_mb = start_memory,
        "Starting resource-intensive operation"
    );
    
    // Phase 1: Loading and validation
    let validated_data = {
        let _phase_span = tracing::info_span!("validation_phase").entered();
        validate_large_dataset(&data).await?
    };
    
    let memory_after_validation = get_memory_usage();
    if memory_after_validation > start_memory * 2.0 {
        warn!(
            memory_increase = memory_after_validation - start_memory,
            "Significant memory increase during validation"
        );
    }
    
    // Phase 2: Processing
    let processed_data = {
        let _phase_span = tracing::info_span!("processing_phase").entered();
        process_large_dataset(validated_data).await?
    };
    
    let final_memory = get_memory_usage();
    let peak_memory = get_peak_memory_usage();
    
    // Record resource usage in span
    tracing::Span::current().record("memory_usage_mb", &final_memory);
    tracing::Span::current().record("peak_memory_mb", &peak_memory);
    
    info!(
        final_memory_mb = final_memory,
        peak_memory_mb = peak_memory,
        memory_efficiency = processed_data.size_mb() / peak_memory,
        "Resource-intensive operation completed"
    );
    
    Ok(processed_data)
}

fn get_memory_usage() -> f64 {
    // Mock implementation - in real code, use a proper memory monitoring library
    42.0
}

fn get_peak_memory_usage() -> f64 {
    // Mock implementation
    55.0
}
```

## Distributed Tracing

### Trace Context Propagation

```rust
use logffi::{info, instrument};

#[instrument(fields(trace_id, span_id))]
async fn api_handler(request: HttpRequest) -> HttpResponse {
    // Extract trace context from headers
    let trace_context = extract_trace_context(&request);
    
    // Record trace IDs in current span
    if let Some(trace_id) = trace_context.trace_id() {
        tracing::Span::current().record("trace_id", &trace_id);
    }
    if let Some(span_id) = trace_context.span_id() {
        tracing::Span::current().record("span_id", &span_id);
    }
    
    info!("Processing API request");
    
    // Call downstream service with trace context
    let result = call_downstream_service(&trace_context).await?;
    
    info!("API request processed successfully");
    Ok(HttpResponse::ok(result))
}

#[instrument]
async fn call_downstream_service(trace_context: &TraceContext) -> Result<ServiceResponse, ServiceError> {
    info!("Calling downstream service");
    
    // Create HTTP client with trace headers
    let client = reqwest::Client::new();
    let mut request = client.post("https://downstream-service.com/api/process");
    
    // Inject trace context into headers
    if let Some(trace_id) = trace_context.trace_id() {
        request = request.header("x-trace-id", trace_id);
    }
    if let Some(span_id) = trace_context.span_id() {
        request = request.header("x-parent-span-id", span_id);
    }
    
    let response = request.send().await?;
    let service_response = response.json::<ServiceResponse>().await?;
    
    info!(
        downstream_status = service_response.status,
        "Downstream service call completed"
    );
    
    Ok(service_response)
}

#[instrument]
async fn database_operation_with_tracing(query: &str) -> Result<QueryResult, DatabaseError> {
    info!("Executing database query");
    
    // In a real implementation, you would propagate trace context to database
    // connection pools and include trace IDs in database logs
    
    let result = execute_query(query).await?;
    
    info!(
        rows_affected = result.rows_affected,
        "Database query completed"
    );
    
    Ok(result)
}

// Mock types for compilation
struct HttpRequest;
struct HttpResponse;
struct TraceContext;
struct ServiceResponse { status: String }
struct ServiceError;
struct QueryResult { rows_affected: u64 }
struct DatabaseError;

impl TraceContext {
    fn trace_id(&self) -> Option<String> { Some("trace123".to_string()) }
    fn span_id(&self) -> Option<String> { Some("span456".to_string()) }
}

fn extract_trace_context(_request: &HttpRequest) -> TraceContext {
    TraceContext
}

impl HttpResponse {
    fn ok<T>(_body: T) -> Self { HttpResponse }
}

async fn execute_query(_query: &str) -> Result<QueryResult, DatabaseError> {
    Ok(QueryResult { rows_affected: 1 })
}
```

## Best Practices

### Span Naming and Organization

```rust
use logffi::{instrument, Level};

// ✅ Good: Clear, hierarchical span names
#[instrument(name = "user_service.create_user")]
async fn create_user(user_data: CreateUserRequest) -> Result<User, UserServiceError> {
    // ...
}

#[instrument(name = "user_service.update_profile")]
async fn update_user_profile(user_id: u64, updates: ProfileUpdates) -> Result<User, UserServiceError> {
    // ...
}

#[instrument(name = "payment_service.process_payment")]
async fn process_payment(payment_request: PaymentRequest) -> Result<PaymentResult, PaymentError> {
    // ...
}

// ✅ Good: Use appropriate span levels
#[instrument(level = "info")]  // Important business operations
async fn place_order(order: Order) -> Result<OrderResult, OrderError> {
    // ...
}

#[instrument(level = "debug")]  // Internal implementation details
async fn validate_order_items(items: &[OrderItem]) -> Result<(), ValidationError> {
    // ...
}

#[instrument(level = "trace")]  // Very detailed tracing
async fn cache_lookup(key: &str) -> Option<CachedValue> {
    // ...
}
```

### Field Selection and Sensitivity

```rust
use logffi::instrument;

// ✅ Good: Skip sensitive data
#[instrument(skip(password, credit_card, api_secret))]
async fn authenticate_and_charge(
    username: &str,
    password: &str,
    credit_card: &CreditCard,
    api_secret: &str,
    amount: f64,
) -> Result<ChargeResult, ChargeError> {
    // Span will include username and amount, but not sensitive fields
}

// ✅ Good: Include relevant business context
#[instrument(fields(
    user_type = %user.user_type(),
    subscription_level = %user.subscription_level(),
    feature_flags = ?user.enabled_features()
))]
async fn personalized_recommendation(user: &User) -> Vec<Recommendation> {
    // ...
}

// ✅ Good: Custom field formatting
#[instrument(fields(
    order_id = %order.id,
    order_total = %format!("${:.2}", order.total),
    item_count = order.items.len(),
    shipping_method = %order.shipping.method
))]
async fn fulfill_order(order: &Order) -> Result<FulfillmentResult, FulfillmentError> {
    // ...
}
```

### Error and Result Tracking

```rust
use logffi::{instrument, error};

// ✅ Good: Track both errors and return values
#[instrument(err, ret)]
async fn critical_calculation(input: f64) -> Result<f64, CalculationError> {
    if input < 0.0 {
        return Err(CalculationError::InvalidInput);
    }
    
    Ok(input * 2.0)
}

// ✅ Good: Custom error handling in spans
#[instrument(fields(error_type, error_details))]
async fn resilient_operation() -> Result<String, Box<dyn std::error::Error>> {
    match risky_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            // Record error details in span
            tracing::Span::current().record("error_type", &format!("{:?}", e));
            tracing::Span::current().record("error_details", &e.to_string());
            
            error!(error = %e, "Operation failed");
            Err(e)
        }
    }
}

async fn risky_operation() -> Result<String, Box<dyn std::error::Error>> {
    Ok("success".to_string())
}
```

## Next Steps

Now that you understand spans and instrumentation, explore these advanced topics:

- **[Advanced Tracing Integration](05-advanced-tracing.md)** - OpenTelemetry, custom subscribers, and monitoring platforms
- **[FFI Integration](06-ffi-integration.md)** - Cross-language tracing and callback instrumentation

## Troubleshooting

### Common Issues

**Q: My spans aren't appearing in the output**

```rust
// Make sure you have proper subscriber initialization
use logffi::{registry, fmt, SubscriberExt, SubscriberInitExt};

registry()
    .with(fmt::layer())
    .init();

// And ensure spans are entered
let span = span!(Level::INFO, "my_span");
let _enter = span.enter(); // Don't forget this!
```

**Q: Async spans are getting mixed up**

```rust
// ✅ Good: Use #[instrument] for async functions
#[instrument]
async fn async_function() {
    // Span context is automatically maintained across await points
}

// ❌ Problematic: Manual span management in async code
async fn manual_async_spans() {
    let span = span!(Level::INFO, "async_op");
    let _enter = span.enter();
    // Context might not be maintained properly across awaits
}
```

**Q: Too much span overhead in hot paths**

```rust
// Use appropriate span levels and conditional compilation
#[instrument(level = "trace")]  // Only active when trace logging enabled
fn hot_path_function() {
    // High-frequency operations
}

// Or use conditional spans
fn conditional_spanning(enable_detailed_tracing: bool) {
    if enable_detailed_tracing {
        let _span = tracing::info_span!("detailed_operation").entered();
        // Detailed operation
    } else {
        // Fast path without tracing overhead
    }
}
```
