# Error Monitoring and Alerting

This cookbook entry demonstrates how to build comprehensive error monitoring and alerting systems using LogFFI's `error_info()` method for structured error introspection.

## Overview

The `error_info()` method provides structured access to error metadata, making it perfect for:

- 📊 Building error dashboards
- 🚨 Setting up monitoring alerts
- 📈 Analyzing error patterns
- 🔍 Creating SLA monitoring from error rates by level

## Basic Error Monitoring

### Setting Up Error Metrics Collection

```rust
use logffi::define_errors;
use std::collections::HashMap;

define_errors! {
    ApiError {
        DatabaseTimeout { query: String, duration_ms: u64 } : "Database timeout: {query} ({duration_ms}ms)" [level = error, target = "api::db"],
        RateLimit { endpoint: String, limit: u32 } : "Rate limit on {endpoint}: {limit}/min" [level = warn, target = "api::rate"],
        ValidationFailed { field: String } : "Validation failed: {field}" [level = info, target = "api::validation"]
    }
}

struct ErrorMetrics {
    error_counts: HashMap<String, u64>,
    level_counts: HashMap<String, u64>,
    target_counts: HashMap<String, u64>,
}

impl ErrorMetrics {
    fn record_error(&mut self, error: &ApiError) {
        let (code, level, target) = error.error_info();
        
        *self.error_counts.entry(code.to_string()).or_insert(0) += 1;
        *self.level_counts.entry(level.to_string()).or_insert(0) += 1;
        *self.target_counts.entry(target.to_string()).or_insert(0) += 1;
    }
}
```

### Prometheus Integration

Export metrics in Prometheus format for monitoring dashboards:

```rust
impl ErrorMetrics {
    fn generate_prometheus_metrics(&self) -> String {
        let mut output = String::new();
        
        // Error counts by type
        output.push_str("# HELP logffi_errors_total Total errors by type\n");
        output.push_str("# TYPE logffi_errors_total counter\n");
        for (error_code, count) in &self.error_counts {
            output.push_str(&format!(
                "logffi_errors_total{{error_code=\"{}\"}} {}\n", 
                error_code, count
            ));
        }
        
        // Error counts by level
        output.push_str("# HELP logffi_errors_by_level_total Total errors by severity\n");
        output.push_str("# TYPE logffi_errors_by_level_total counter\n");
        for (level, count) in &self.level_counts {
            output.push_str(&format!(
                "logffi_errors_by_level_total{{level=\"{}\"}} {}\n",
                level, count
            ));
        }
        
        output
    }
}
```

## Advanced Monitoring Patterns

### Time-Series Error Tracking

Track error rates over time for trend analysis:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct TimeSeriesMetrics {
    windows: HashMap<u64, HashMap<String, u64>>, // timestamp -> error counts
    window_size: u64, // seconds
}

impl TimeSeriesMetrics {
    fn new(window_size: u64) -> Self {
        Self {
            windows: HashMap::new(),
            window_size,
        }
    }
    
    fn record_error<E>(&mut self, error: &E) 
    where E: /* has error_info method */
    {
        let (code, _level, _target) = error.error_info();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let window = (now / self.window_size) * self.window_size;
        
        *self.windows
            .entry(window)
            .or_insert_with(HashMap::new)
            .entry(code.to_string())
            .or_insert(0) += 1;
    }
    
    fn get_error_rate(&self, error_code: &str, windows: usize) -> f64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let current_window = (now / self.window_size) * self.window_size;
        
        let mut total = 0u64;
        for i in 0..windows {
            let window = current_window - (i as u64 * self.window_size);
            if let Some(window_data) = self.windows.get(&window) {
                total += window_data.get(error_code).unwrap_or(&0);
            }
        }
        
        total as f64 / windows as f64
    }
}
```

### Smart Alert Conditions

Create intelligent alerting based on error patterns:

```rust
#[derive(Debug)]
struct AlertManager {
    thresholds: HashMap<String, AlertThreshold>,
    alert_history: Vec<Alert>,
}

#[derive(Debug)]
struct AlertThreshold {
    level: String,
    max_count: u64,
    time_window: u64,
}

#[derive(Debug)]
struct Alert {
    condition: String,
    message: String,
    timestamp: u64,
    severity: AlertSeverity,
}

#[derive(Debug)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertManager {
    fn check_alert_conditions<E>(&mut self, errors: &[E]) -> Vec<Alert>
    where E: /* has error_info method */
    {
        let mut alerts = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Count errors by level
        let mut level_counts = HashMap::new();
        for error in errors {
            let (_code, level, _target) = error.error_info();
            *level_counts.entry(level).or_insert(0) += 1;
        }
        
        // Check critical error threshold
        if let Some(&error_count) = level_counts.get("error") {
            if error_count >= 5 {
                alerts.push(Alert {
                    condition: "high_critical_errors".to_string(),
                    message: format!("High critical error rate: {} errors", error_count),
                    timestamp: now,
                    severity: AlertSeverity::Critical,
                });
            }
        }
        
        // Check database performance
        let db_errors: usize = errors.iter()
            .filter(|e| e.error_info().2.contains("db"))
            .count();
        
        if db_errors >= 3 {
            alerts.push(Alert {
                condition: "database_issues".to_string(),
                message: format!("Database performance degraded: {} database errors", db_errors),
                timestamp: now,
                severity: AlertSeverity::Warning,
            });
        }
        
        alerts
    }
}
```

## Grafana Dashboard Integration

### Creating Dashboard Panels

Set up Grafana panels using Prometheus metrics:

```json
{
  "dashboard": {
    "title": "LogFFI Error Monitoring",
    "panels": [
      {
        "title": "Error Rate by Level",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(logffi_errors_by_level_total[5m])",
            "legendFormat": "{{level}} errors/sec"
          }
        ]
      },
      {
        "title": "Top Error Types",
        "type": "barchart",
        "targets": [
          {
            "expr": "topk(10, logffi_errors_total)",
            "legendFormat": "{{error_code}}"
          }
        ]
      },
      {
        "title": "Error Timeline",
        "type": "timeseries",
        "targets": [
          {
            "expr": "increase(logffi_errors_total[1m])",
            "legendFormat": "{{error_code}}"
          }
        ]
      }
    ]
  }
}
```

### Alert Rules Configuration

Configure Grafana alert rules:

```yaml
# grafana-alerts.yml
groups:
  - name: logffi-errors
    rules:
      - alert: HighErrorRate
        expr: rate(logffi_errors_by_level_total{level="error"}[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/second"
          
      - alert: DatabasePerformance
        expr: rate(logffi_errors_total{error_code="DatabaseTimeout"}[5m]) > 0.05
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Database performance issues"
          description: "Database timeout rate is {{ $value }} timeouts/second"
```

## ELK Stack Integration

### Elasticsearch Index Mapping

Set up proper field mapping for error data:

```json
{
  "mappings": {
    "properties": {
      "@timestamp": { "type": "date" },
      "error_code": { "type": "keyword" },
      "level": { "type": "keyword" },
      "target": { "type": "keyword" },
      "message": { "type": "text" },
      "context": { "type": "keyword" },
      "metadata": {
        "properties": {
          "service": { "type": "keyword" },
          "environment": { "type": "keyword" },
          "version": { "type": "keyword" }
        }
      }
    }
  }
}
```

### Kibana Dashboard Setup

Create visualizations for error analysis:

```json
{
  "version": "7.15.0",
  "objects": [
    {
      "attributes": {
        "title": "Error Rate by Level",
        "type": "line",
        "params": {
          "grid": { "categoryLines": false, "style": { "color": "#eee" } },
          "categoryAxes": [
            {
              "id": "CategoryAxis-1",
              "type": "category",
              "position": "bottom",
              "show": true,
              "style": {},
              "scale": { "type": "linear" },
              "labels": { "show": true, "truncate": 100 },
              "title": {}
            }
          ],
          "valueAxes": [
            {
              "id": "ValueAxis-1",
              "name": "LeftAxis-1",
              "type": "value",
              "position": "left",
              "show": true,
              "style": {},
              "scale": { "type": "linear", "mode": "normal" },
              "labels": { "show": true, "rotate": 0, "filter": false, "truncate": 100 },
              "title": { "text": "Count" }
            }
          ],
          "seriesParams": [
            {
              "show": "true",
              "type": "line",
              "mode": "normal",
              "data": { "label": "Count", "id": "1" },
              "valueAxis": "ValueAxis-1",
              "drawLinesBetweenPoints": true,
              "showCircles": true
            }
          ],
          "addTooltip": true,
          "addLegend": true,
          "legendPosition": "right",
          "times": [],
          "addTimeMarker": false
        }
      }
    }
  ]
}
```

## SLA Monitoring

### Error Budget Tracking

Monitor error budgets for SLA compliance:

```rust
#[derive(Debug)]
struct SLAMonitor {
    error_budget: f64,        // percentage (e.g., 0.1 for 99.9% uptime)
    budget_window: u64,       // time window in seconds
    total_requests: u64,
    error_requests: u64,
}

impl SLAMonitor {
    fn new(error_budget: f64, budget_window: u64) -> Self {
        Self {
            error_budget,
            budget_window,
            total_requests: 0,
            error_requests: 0,
        }
    }
    
    fn record_request<E>(&mut self, result: Result<(), E>) 
    where E: /* has error_info method */
    {
        self.total_requests += 1;
        
        if let Err(error) = result {
            let (_code, level, _target) = error.error_info();
            
            // Only count error-level as SLA violations
            if level == "error" {
                self.error_requests += 1;
            }
        }
    }
    
    fn current_error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.error_requests as f64 / self.total_requests as f64
        }
    }
    
    fn is_budget_exceeded(&self) -> bool {
        self.current_error_rate() > self.error_budget
    }
    
    fn budget_remaining(&self) -> f64 {
        (self.error_budget - self.current_error_rate()).max(0.0)
    }
}
```

## Best Practices

### 1. Error Classification

Classify errors appropriately for monitoring:

```rust
// Use appropriate log levels
define_errors! {
    ServiceError {
        // Critical errors that break functionality
        SystemFailure {} : "System failure" [level = error, target = "service::system"],
        
        // Degraded performance but still functional  
        SlowResponse { duration_ms: u64 } : "Slow response: {duration_ms}ms" [level = warn, target = "service::performance"],
        
        // Expected operational events
        UserNotFound { user_id: u64 } : "User not found: {user_id}" [level = info, target = "service::user"]
    }
}
```

### 2. Meaningful Targets

Use consistent and descriptive target patterns:

```rust
define_errors! {
    AppError {
        // Format: service::component::subcomponent
        DatabaseError {} : "Database error" [level = error, target = "app::db::connection"],
        ApiError {} : "API error" [level = error, target = "app::api::request"],
        CacheError {} : "Cache error" [level = warn, target = "app::cache::redis"]
    }
}
```

### 3. Context-Rich Monitoring

Include relevant context in error messages:

```rust
// Good: Includes actionable context
ValidationError { field: String, constraint: String, value: String } : 
    "Validation failed on {field}: expected {constraint}, got '{value}'" 
    [level = warn, target = "api::validation"]

// Better: Includes request context for debugging
RequestValidationError { 
    field: String, 
    constraint: String, 
    value: String, 
    request_id: String 
} : "Request {request_id} validation failed on {field}: expected {constraint}, got '{value}'" 
  [level = warn, target = "api::validation"]
```

## See Also

- [Error Debugging Workflows](08-error-debugging-workflows.md) - Debug production issues using error patterns
- [Structured Logging](02-structured-logging.md) - Integrate with structured logging systems
- [Advanced Tracing](05-advanced-tracing.md) - Correlate errors with distributed traces
