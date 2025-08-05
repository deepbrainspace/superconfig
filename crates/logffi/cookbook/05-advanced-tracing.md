# Advanced Tracing Integration

This guide covers advanced tracing ecosystem integration using LogFFI, including OpenTelemetry, custom subscribers, and integration with monitoring platforms. LogFFI provides convenient access to the full tracing ecosystem while tracing handles the underlying observability infrastructure.

**What LogFFI provides:** Convenient re-exports of tracing-subscriber components, simplified initialization, and direct access to tracing's advanced features.

**What tracing provides:** The underlying observability framework, OpenTelemetry integration, custom subscriber support, and ecosystem compatibility.

## Table of Contents

- [OpenTelemetry Integration](#opentelemetry-integration)
- [Custom Subscribers](#custom-subscribers)
- [Multiple Subscriber Layers](#multiple-subscriber-layers)
- [Filtering and Sampling](#filtering-and-sampling)
- [Monitoring Platform Integration](#monitoring-platform-integration)
- [Performance Optimization](#performance-optimization)
- [Testing and Development](#testing-and-development)
- [Production Configuration](#production-configuration)

## OpenTelemetry Integration

### Basic OpenTelemetry Setup

```rust
use logffi::{info, instrument, registry, SubscriberExt, SubscriberInitExt};
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;

async fn setup_opentelemetry() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "my-service"),
                    opentelemetry::KeyValue::new("service.version", "1.0.0"),
                    opentelemetry::KeyValue::new("deployment.environment", "production"),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Set up subscriber with OpenTelemetry layer
    registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(logffi::fmt::layer().with_target(false))
        .with(logffi::EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[instrument]
async fn traced_operation(user_id: u64) -> Result<String, Box<dyn std::error::Error>> {
    info!(user_id = user_id, "Starting traced operation");
    
    // This will be sent to OpenTelemetry collector
    let result = process_user_data(user_id).await?;
    
    info!(result_length = result.len(), "Operation completed");
    Ok(result)
}

#[instrument]
async fn process_user_data(user_id: u64) -> Result<String, Box<dyn std::error::Error>> {
    // Simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(format!("processed_data_for_user_{}", user_id))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_opentelemetry().await?;
    
    traced_operation(12345).await?;
    
    // Shutdown OpenTelemetry to flush remaining spans
    global::shutdown_tracer_provider();
    
    Ok(())
}
```

### Advanced OpenTelemetry Configuration

```rust
use logffi::{info, instrument, registry, SubscriberExt, SubscriberInitExt};
use opentelemetry::{global, KeyValue, Context};
use opentelemetry::propagation::Injector;
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use std::collections::HashMap;

struct OtelConfig {
    service_name: String,
    service_version: String,
    environment: String,
    otlp_endpoint: String,
    sampling_ratio: f64,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "logffi-service".to_string(),
            service_version: "1.0.0".to_string(),
            environment: "development".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_ratio: 1.0, // Sample all traces in development
        }
    }
}

async fn setup_advanced_opentelemetry(config: OtelConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create resource with comprehensive metadata
    let resource = opentelemetry::sdk::Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version),
        KeyValue::new("deployment.environment", config.environment),
        KeyValue::new("host.name", hostname::get().unwrap_or_default().to_string_lossy().to_string()),
        KeyValue::new("process.pid", std::process::id().to_string()),
    ]);

    // Configure sampling
    let sampler = opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(config.sampling_ratio);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(config.otlp_endpoint)
                .with_timeout(std::time::Duration::from_secs(10))
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(resource)
                .with_sampler(sampler)
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(128)
        )
        .with_batch_config(
            opentelemetry::sdk::trace::BatchConfig::default()
                .with_max_queue_size(2048)
                .with_max_export_batch_size(512)
                .with_scheduled_delay(std::time::Duration::from_millis(500))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Set up subscriber with multiple layers
    registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(
            logffi::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_span_events(logffi::fmt::format::FmtSpan::ACTIVE)
        )
        .with(logffi::EnvFilter::from_default_env())
        .init();

    Ok(())
}

// Example of distributed tracing with context propagation
#[instrument]
async fn http_handler(headers: HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Processing HTTP request");

    // Extract OpenTelemetry context from headers
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor::new(&headers))
    });

    // Create a span with the extracted context as parent
    let span = tracing::Span::current();
    span.set_parent(parent_context);

    // Make downstream HTTP call with context propagation
    let downstream_response = make_downstream_call().await?;

    info!(response_size = downstream_response.len(), "Request processed");
    Ok(downstream_response)
}

#[instrument]
async fn make_downstream_call() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HashMap::new();
    
    // Inject current span context into headers
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&Context::current(), &mut HeaderInjector::new(&mut headers))
    });

    info!("Making downstream service call");
    
    // Here you would make actual HTTP call with propagated headers
    // For demo, we simulate the call
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    Ok("downstream_response".to_string())
}

// Helper types for context propagation
struct HeaderExtractor<'a> {
    headers: &'a HashMap<String, String>,
}

impl<'a> HeaderExtractor<'a> {
    fn new(headers: &'a HashMap<String, String>) -> Self {
        Self { headers }
    }
}

impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|k| k.as_str()).collect()
    }
}

struct HeaderInjector<'a> {
    headers: &'a mut HashMap<String, String>,
}

impl<'a> HeaderInjector<'a> {
    fn new(headers: &'a mut HashMap<String, String>) -> Self {
        Self { headers }
    }
}

impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(key.to_string(), value);
    }
}
```

## Custom Subscribers

### Creating Custom Layers

```rust
use logffi::{registry, SubscriberExt, SubscriberInitExt, info};
use tracing_subscriber::Layer;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Custom layer that collects metrics
#[derive(Debug, Clone)]
pub struct MetricsLayer {
    metrics: Arc<Mutex<HashMap<String, MetricData>>>,
}

#[derive(Debug, Clone)]
struct MetricData {
    count: u64,
    total_duration_ns: u64,
    min_duration_ns: u64,
    max_duration_ns: u64,
}

impl MetricsLayer {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_metrics(&self) -> HashMap<String, MetricData> {
        self.metrics.lock().unwrap().clone()
    }
}

impl<S> Layer<S> for MetricsLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, id: &tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            extensions.insert(std::time::Instant::now());
        }
    }

    fn on_close(&self, id: tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let extensions = span.extensions();
            if let Some(start_time) = extensions.get::<std::time::Instant>() {
                let duration = start_time.elapsed();
                let span_name = span.metadata().name().to_string();
                
                let mut metrics = self.metrics.lock().unwrap();
                let entry = metrics.entry(span_name).or_insert(MetricData {
                    count: 0,
                    total_duration_ns: 0,
                    min_duration_ns: u64::MAX,
                    max_duration_ns: 0,
                });
                
                entry.count += 1;
                entry.total_duration_ns += duration.as_nanos() as u64;
                entry.min_duration_ns = entry.min_duration_ns.min(duration.as_nanos() as u64);
                entry.max_duration_ns = entry.max_duration_ns.max(duration.as_nanos() as u64);
            }
        }
    }
}

// Custom layer for audit logging
#[derive(Debug)]
pub struct AuditLayer {
    audit_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
}

impl AuditLayer {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self {
            audit_writer: Arc::new(Mutex::new(writer)),
        }
    }
}

impl<S> Layer<S> for AuditLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Only log events from specific targets as audit events
        if event.metadata().target().starts_with("audit") {
            let mut visitor = AuditVisitor::new();
            event.record(&mut visitor);
            
            let audit_entry = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "level": format!("{}", event.metadata().level()),
                "target": event.metadata().target(),
                "message": visitor.message,
                "fields": visitor.fields,
            });
            
            if let Ok(mut writer) = self.audit_writer.lock() {
                writeln!(writer, "{}", audit_entry).ok();
                writer.flush().ok();
            }
        }
    }
}

struct AuditVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl AuditVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: serde_json::Map::new(),
        }
    }
}

impl tracing::field::Visit for AuditVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value).trim_matches('"').to_string();
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(format!("{:?}", value))
            );
        }
    }
}

// Setting up multiple custom layers
fn setup_custom_subscribers() -> Result<(MetricsLayer, AuditLayer), Box<dyn std::error::Error>> {
    let metrics_layer = MetricsLayer::new();
    let audit_file = std::fs::File::create("audit.log")?;
    let audit_layer = AuditLayer::new(Box::new(audit_file));

    registry()
        .with(logffi::fmt::layer())
        .with(metrics_layer.clone())
        .with(audit_layer.clone())
        .with(logffi::EnvFilter::from_default_env())
        .init();

    Ok((metrics_layer, audit_layer))
}

// Example usage of custom subscribers
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (metrics_layer, _audit_layer) = setup_custom_subscribers()?;

    // Regular application logging
    info!("Application starting");

    // Audit logging (goes to audit.log)
    tracing::info!(target: "audit", user_id = 12345, action = "login", "User login event");

    // Some traced operations to generate metrics
    traced_operation_1().await;
    traced_operation_2().await;

    // Print collected metrics
    println!("Collected metrics: {:#?}", metrics_layer.get_metrics());

    Ok(())
}

#[logffi::instrument]
async fn traced_operation_1() {
    info!("Executing operation 1");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[logffi::instrument]
async fn traced_operation_2() {
    info!("Executing operation 2");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}
```

## Multiple Subscriber Layers

### Layered Architecture

```rust
use logffi::{info, registry, fmt, SubscriberExt, SubscriberInitExt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt as _;

fn setup_multi_layer_subscriber() -> Result<(), Box<dyn std::error::Error>> {
    // Layer 1: Console output with formatting
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_span_events(fmt::format::FmtSpan::CLOSE);

    // Layer 2: JSON file output
    let file = std::fs::File::create("application.json")?;
    let json_layer = fmt::layer()
        .json()
        .with_writer(file)
        .with_current_span(true)
        .with_span_list(true);

    // Layer 3: Structured metrics collection
    let metrics_layer = MetricsLayer::new();

    // Layer 4: Error tracking
    let error_layer = ErrorTrackingLayer::new();

    // Combine all layers with filtering
    registry()
        .with(
            console_layer
                .with_filter(EnvFilter::new("info"))
        )
        .with(
            json_layer
                .with_filter(EnvFilter::new("debug"))
        )
        .with(
            metrics_layer
                .with_filter(EnvFilter::new("trace"))
        )
        .with(
            error_layer
                .with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
        )
        .init();

    Ok(())
}

// Error tracking layer
#[derive(Debug)]
pub struct ErrorTrackingLayer {
    error_count: Arc<Mutex<HashMap<String, u64>>>,
}

impl ErrorTrackingLayer {
    pub fn new() -> Self {
        Self {
            error_count: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_error_stats(&self) -> HashMap<String, u64> {
        self.error_count.lock().unwrap().clone()
    }
}

impl<S> tracing_subscriber::Layer<S> for ErrorTrackingLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if *event.metadata().level() == tracing::Level::ERROR {
            let target = event.metadata().target().to_string();
            let mut counts = self.error_count.lock().unwrap();
            *counts.entry(target).or_insert(0) += 1;
        }
    }
}
```

## Filtering and Sampling

### Advanced Filtering

```rust
use logffi::{registry, fmt, SubscriberExt, SubscriberInitExt, EnvFilter};
use tracing_subscriber::filter::{FilterFn, LevelFilter};

fn setup_advanced_filtering() {
    // Complex environment-based filtering
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("sqlx=debug".parse().unwrap())
        .add_directive("myapp::sensitive=error".parse().unwrap());

    // Custom filtering function
    let custom_filter = FilterFn::new(|metadata| {
        // Skip noisy third-party crates in production
        if cfg!(not(debug_assertions)) {
            if metadata.target().starts_with("h2::") 
                || metadata.target().starts_with("hyper::")
                || metadata.target().starts_with("tokio::") {
                return false;
            }
        }

        // Always include error-level events
        if *metadata.level() == tracing::Level::ERROR {
            return true;
        }

        // Skip trace-level events from database layer in production
        if *metadata.level() == tracing::Level::TRACE 
            && metadata.target().contains("database") 
            && cfg!(not(debug_assertions)) {
            return false;
        }

        true
    });

    // Performance-based sampling
    let sampling_filter = FilterFn::new(|metadata| {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Sample only every 10th trace-level event in high-throughput scenarios
        if *metadata.level() == tracing::Level::TRACE {
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            return count % 10 == 0;
        }

        true
    });

    registry()
        .with(
            fmt::layer()
                .with_filter(env_filter)
                .with_filter(custom_filter)
                .with_filter(sampling_filter)
        )
        .init();
}

// Dynamic filtering that can be changed at runtime
use std::sync::Arc;
use tracing_subscriber::reload;

type ReloadHandle = reload::Handle<EnvFilter, registry>;

fn setup_dynamic_filtering() -> ReloadHandle {
    let (env_filter, reload_handle) = reload::Layer::new(EnvFilter::from_default_env());

    registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    reload_handle
}

// Function to change log level at runtime
fn change_log_level(handle: &ReloadHandle, new_filter: &str) -> Result<(), Box<dyn std::error::Error>> {
    let new_env_filter = EnvFilter::try_new(new_filter)?;
    handle.reload(new_env_filter)?;
    info!("Log level changed to: {}", new_filter);
    Ok(())
}
```

## Monitoring Platform Integration

### Datadog Integration

```rust
use logffi::{info, instrument, registry, fmt, SubscriberExt, SubscriberInitExt};

fn setup_datadog_integration() {
    // Configure for Datadog Log Management
    let datadog_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .map_event_format(|_| {
            fmt::format()
                .json()
                .with_current_span(true)
                .with_span_events(fmt::format::FmtSpan::NONE)
        });

    registry()
        .with(datadog_layer)
        .with(logffi::EnvFilter::from_default_env())
        .init();
}

#[instrument(fields(
    dd.service = "my-rust-service",
    dd.version = "1.0.0",
    dd.env = "production"
))]
async fn datadog_traced_function(user_id: u64) {
    info!(
        user_id = user_id,
        dd.trace_id = %generate_trace_id(),
        dd.span_id = %generate_span_id(),
        "Processing user request"
    );
    
    // Business logic here
}

fn generate_trace_id() -> String {
    // In real implementation, extract from Datadog tracing context
    "1234567890123456".to_string()
}

fn generate_span_id() -> String {
    // In real implementation, extract from Datadog tracing context  
    "0987654321098765".to_string()
}
```

### Prometheus Metrics Integration

```rust
use logffi::{info, instrument, registry, SubscriberExt, SubscriberInitExt};
use prometheus::{Counter, Histogram, Registry as PrometheusRegistry};
use std::sync::Arc;

#[derive(Clone)]
pub struct PrometheusLayer {
    request_counter: Counter,
    request_duration: Histogram,
    error_counter: Counter,
}

impl PrometheusLayer {
    pub fn new(registry: &PrometheusRegistry) -> Result<Self, prometheus::Error> {
        let request_counter = Counter::new(
            "http_requests_total",
            "Total number of HTTP requests"
        )?;
        
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        )?;
        
        let error_counter = Counter::new(
            "application_errors_total",
            "Total number of application errors"
        )?;

        registry.register(Box::new(request_counter.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(error_counter.clone()))?;

        Ok(Self {
            request_counter,
            request_duration,
            error_counter,
        })
    }
}

impl<S> tracing_subscriber::Layer<S> for PrometheusLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(&self, _attrs: &tracing::span::Attributes<'_>, id: &tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            if span.metadata().name() == "http_request" {
                self.request_counter.inc();
                let mut extensions = span.extensions_mut();
                extensions.insert(std::time::Instant::now());
            }
        }
    }

    fn on_close(&self, id: tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            if span.metadata().name() == "http_request" {
                let extensions = span.extensions();
                if let Some(start_time) = extensions.get::<std::time::Instant>() {
                    let duration = start_time.elapsed();
                    self.request_duration.observe(duration.as_secs_f64());
                }
            }
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if *event.metadata().level() == tracing::Level::ERROR {
            self.error_counter.inc();
        }
    }
}

fn setup_prometheus_integration() -> Result<(), Box<dyn std::error::Error>> {
    let prometheus_registry = PrometheusRegistry::new();
    let prometheus_layer = PrometheusLayer::new(&prometheus_registry)?;

    registry()
        .with(logffi::fmt::layer())
        .with(prometheus_layer)
        .with(logffi::EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[instrument(name = "http_request")]
async fn handle_http_request(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!(http.path = path, "Handling HTTP request");
    
    // Simulate request processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok("response".to_string())
}
```

## Performance Optimization

### Zero-Cost Abstractions

```rust
use logffi::{debug, trace, info};

// Compile-time log level filtering
macro_rules! expensive_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            if tracing::enabled!(tracing::Level::DEBUG) {
                debug!($($arg)*);
            }
        }
    };
}

// Conditional expensive operations
fn optimized_logging_example(data: &[u8]) {
    info!("Processing data batch");

    // ✅ Good: Only compute expensive debug info when needed
    expensive_debug!(
        data_checksum = %compute_checksum(data),
        data_preview = %format!("{:?}", &data[..10.min(data.len())]),
        "Detailed data information"
    );

    // ✅ Good: Use feature flags for detailed tracing
    #[cfg(feature = "detailed-tracing")]
    trace!(
        data_length = data.len(),
        data_hash = %compute_hash(data),
        "Trace-level data analysis"
    );
}

fn compute_checksum(data: &[u8]) -> u32 {
    // Expensive checksum calculation
    data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}

#[cfg(feature = "detailed-tracing")]
fn compute_hash(data: &[u8]) -> String {
    // Very expensive hash calculation
    format!("hash_{}", data.len())
}
```

### High-Performance Async Tracing

```rust
use logffi::{instrument, info};
use std::pin::Pin;
use std::future::Future;

// Custom instrumentation for hot paths
trait InstrumentedFuture: Future {
    fn instrument_hot_path(self, span_name: &'static str) -> Pin<Box<dyn Future<Output = Self::Output> + Send>>
    where
        Self: Send + Sized + 'static,
        Self::Output: Send,
    {
        Box::pin(async move {
            let span = tracing::trace_span!(span_name);
            let _guard = span.enter();
            self.await
        })
    }
}

impl<F: Future> InstrumentedFuture for F {}

// High-performance batch processing with minimal tracing overhead
#[instrument(skip(items), fields(batch_size = items.len()))]
async fn high_performance_batch_processing<T>(items: Vec<T>) -> Result<Vec<ProcessedItem>, ProcessingError>
where
    T: Send + Sync + 'static,
{
    info!("Starting high-performance batch processing");

    let batch_size = items.len();
    let chunk_size = 1000;
    let mut results = Vec::with_capacity(batch_size);

    for (chunk_index, chunk) in items.chunks(chunk_size).enumerate() {
        // Only instrument every 10th chunk to reduce overhead
        let chunk_result = if chunk_index % 10 == 0 {
            process_chunk_with_tracing(chunk, chunk_index).await?
        } else {
            process_chunk_minimal_tracing(chunk).await?
        };
        
        results.extend(chunk_result);
    }

    info!(
        total_processed = results.len(),
        "Batch processing completed"
    );

    Ok(results)
}

#[instrument(skip(chunk), fields(chunk_index = chunk_index, chunk_size = chunk.len()))]
async fn process_chunk_with_tracing<T>(chunk: &[T], chunk_index: usize) -> Result<Vec<ProcessedItem>, ProcessingError> {
    info!("Processing chunk with detailed tracing");
    // Process chunk with full instrumentation
    Ok(vec![ProcessedItem::default(); chunk.len()])
}

async fn process_chunk_minimal_tracing<T>(chunk: &[T]) -> Result<Vec<ProcessedItem>, ProcessingError> {
    // Minimal tracing for hot path
    Ok(vec![ProcessedItem::default(); chunk.len()])
}

#[derive(Default)]
struct ProcessedItem;

struct ProcessingError;
```

## Testing and Development

### Test-Specific Tracing Configuration

```rust
#[cfg(test)]
mod test_tracing {
    use logffi::{registry, fmt, SubscriberExt, SubscriberInitExt, info};
    use std::sync::Once;
    use tracing_test::traced_test;

    static INIT: Once = Once::new();

    pub fn init_test_tracing() {
        INIT.call_once(|| {
            registry()
                .with(
                    fmt::layer()
                        .with_test_writer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_span_events(fmt::format::FmtSpan::CLOSE)
                )
                .with(logffi::EnvFilter::new("debug"))
                .init();
        });
    }

    #[traced_test]
    #[tokio::test]
    async fn test_with_tracing() {
        info!("Starting test with tracing");
        
        let result = traced_function().await;
        
        assert_eq!(result, "success");
        info!("Test completed successfully");
    }

    #[logffi::instrument]
    async fn traced_function() -> &'static str {
        info!("Executing traced function in test");
        "success"
    }
}
```

## Production Configuration

### Production-Ready Setup

```rust
use logffi::{registry, fmt, SubscriberExt, SubscriberInitExt, EnvFilter};
use std::env;

pub struct TracingConfig {
    pub environment: String,
    pub service_name: String,
    pub service_version: String,
    pub json_output: bool,
    pub otlp_endpoint: Option<String>,
    pub sample_rate: f64,
}

impl TracingConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "rust-service".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            json_output: env::var("JSON_LOGS").unwrap_or_else(|_| "false".to_string()).parse()?,
            otlp_endpoint: env::var("OTLP_ENDPOINT").ok(),
            sample_rate: env::var("TRACE_SAMPLE_RATE")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()?,
        })
    }
}

pub async fn setup_production_tracing(config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut layers = Vec::new();

    // Console/file output layer
    if config.json_output {
        let json_layer = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false)
            .with_target(false)
            .with_timer(fmt::time::UtcTime::rfc_3339());
        layers.push(json_layer.boxed());
    } else {
        let pretty_layer = fmt::layer()
            .with_target(true)
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .with_span_events(fmt::format::FmtSpan::CLOSE);
        layers.push(pretty_layer.boxed());
    }

    // OpenTelemetry layer (if configured)
    if let Some(otlp_endpoint) = &config.otlp_endpoint {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint.clone())
            )
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(config.sample_rate))
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
                        opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
                        opentelemetry::KeyValue::new("deployment.environment", config.environment.clone()),
                    ]))
            )
            .install_batch(opentelemetry::runtime::Tokio)?;

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        layers.push(otel_layer.boxed());
    }

    // Environment-based filtering
    let filter = EnvFilter::builder()
        .with_default_directive(
            if config.environment == "production" {
                tracing_subscriber::filter::LevelFilter::INFO.into()
            } else {
                tracing_subscriber::filter::LevelFilter::DEBUG.into()
            }
        )
        .from_env_lossy();

    // Combine all layers
    let subscriber = layers
        .into_iter()
        .fold(registry().with(filter), |acc, layer| acc.with(layer));

    subscriber.init();

    Ok(())
}

// Graceful shutdown for production
pub async fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}

// Health check endpoint that includes tracing status
#[logffi::instrument]
pub async fn health_check() -> serde_json::Value {
    info!("Health check requested");
    
    serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "tracing": "active"
    })
}
```

## Next Steps

You now have comprehensive knowledge of advanced tracing integration. Explore the final cookbook:

- **[FFI Integration](06-ffi-integration.md)** - Cross-language tracing and callback instrumentation

## Troubleshooting

### Common Issues

**Q: OpenTelemetry spans aren't appearing in my collector**

```rust
// Make sure to flush spans before shutdown
use opentelemetry::global;

// At application shutdown
global::shutdown_tracer_provider();
```

**Q: Too much tracing overhead in production**

```rust
// Use sampling and appropriate log levels
let sampler = opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(0.1); // 10% sampling

// Use conditional compilation for expensive traces
#[cfg(debug_assertions)]
trace!("Expensive debug information: {:?}", expensive_computation());
```

**Q: Custom layers not receiving events**

```rust
// Make sure layer is properly registered
registry()
    .with(my_custom_layer)  // ✅ Correct order
    .with(fmt::layer())
    .init();
```
