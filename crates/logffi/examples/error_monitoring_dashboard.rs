use logffi::define_errors;
use std::collections::HashMap;

// Example: Building an error monitoring dashboard using error_info()

define_errors! {
    // API-related errors
    ApiError {
        DatabaseTimeout { query: String, duration_ms: u64 } : "Database query timed out: {query} ({duration_ms}ms)" [level = error, target = "api::db"],
        RateLimited { user_id: u64, limit: u32 } : "Rate limit exceeded for user {user_id}: {limit} requests/min" [level = warn, target = "api::rate"],
        ValidationFailed { field: String, value: String } : "Validation failed for {field}: '{value}'" [level = info, target = "api::validation"],
        Unauthorized { token: String } : "Invalid auth token: {token}" [level = warn, target = "api::auth"]
    }

    // System-level errors
    SystemError {
        DiskFull { path: String, available_mb: u64 } : "Disk full at {path}: {available_mb}MB available" [level = error, target = "system::storage"],
        MemoryPressure { used_mb: u64, total_mb: u64 } : "High memory usage: {used_mb}MB/{total_mb}MB" [level = warn, target = "system::memory"],
        ServiceDown { service: String } : "Service unavailable: {service}" [level = error, target = "system::health"]
    }
}

/// Metrics collector for monitoring dashboard
pub struct ErrorMetricsCollector {
    error_counts: HashMap<String, u64>,
    level_counts: HashMap<String, u64>,
    target_counts: HashMap<String, u64>,
}

impl ErrorMetricsCollector {
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            level_counts: HashMap::new(),
            target_counts: HashMap::new(),
        }
    }

    /// Record error metrics using error_info()
    pub fn record_api_error(&mut self, error: &ApiError) {
        let (code, level, target) = error.error_info();
        self.record_metrics(code, level, target);

        // Log the error with tracing integration
        error.log();
    }

    pub fn record_system_error(&mut self, error: &SystemError) {
        let (code, level, target) = error.error_info();
        self.record_metrics(code, level, target);
        error.log();
    }

    fn record_metrics(&mut self, code: &str, level: &str, target: &str) {
        *self.error_counts.entry(code.to_string()).or_insert(0) += 1;
        *self.level_counts.entry(level.to_string()).or_insert(0) += 1;
        *self.target_counts.entry(target.to_string()).or_insert(0) += 1;
    }

    /// Generate Prometheus-style metrics
    pub fn generate_prometheus_metrics(&self) -> String {
        let mut output = String::new();

        // Error counts by type
        output.push_str("# HELP logffi_errors_total Total number of errors by type\n");
        output.push_str("# TYPE logffi_errors_total counter\n");
        for (error_code, count) in &self.error_counts {
            output.push_str(&format!(
                "logffi_errors_total{{error_code=\"{}\"}} {}\n",
                error_code, count
            ));
        }

        // Error counts by level
        output.push_str("\n# HELP logffi_errors_by_level_total Total number of errors by level\n");
        output.push_str("# TYPE logffi_errors_by_level_total counter\n");
        for (level, count) in &self.level_counts {
            output.push_str(&format!(
                "logffi_errors_by_level_total{{level=\"{}\"}} {}\n",
                level, count
            ));
        }

        // Error counts by target
        output
            .push_str("\n# HELP logffi_errors_by_target_total Total number of errors by target\n");
        output.push_str("# TYPE logffi_errors_by_target_total counter\n");
        for (target, count) in &self.target_counts {
            output.push_str(&format!(
                "logffi_errors_by_target_total{{target=\"{}\"}} {}\n",
                target, count
            ));
        }

        output
    }

    /// Generate alert conditions based on error patterns
    pub fn check_alert_conditions(&self) -> Vec<String> {
        let mut alerts = Vec::new();

        // Check for critical error thresholds
        if let Some(&error_count) = self.level_counts.get("error") {
            if error_count >= 10 {
                alerts.push(format!(
                    "HIGH_ERROR_RATE: {} critical errors detected",
                    error_count
                ));
            }
        }

        // Check for specific error patterns
        if let Some(&db_timeouts) = self.error_counts.get("DatabaseTimeout") {
            if db_timeouts >= 5 {
                alerts.push(format!(
                    "DATABASE_PERFORMANCE: {} timeouts in monitoring window",
                    db_timeouts
                ));
            }
        }

        // Check for system health issues
        if self.target_counts.get("system::storage").unwrap_or(&0) > &0 {
            alerts.push("STORAGE_WARNING: Disk space issues detected".to_string());
        }

        alerts
    }

    /// Print human-readable dashboard
    pub fn print_dashboard(&self) {
        println!("🚨 Error Monitoring Dashboard");
        println!("============================");

        println!("\n📊 Error Counts by Type:");
        for (error_code, count) in &self.error_counts {
            println!("  {} → {} occurrences", error_code, count);
        }

        println!("\n🎯 Error Counts by Severity:");
        for (level, count) in &self.level_counts {
            let emoji = match level.as_str() {
                "error" => "🔴",
                "warn" => "🟡",
                "info" => "🔵",
                _ => "⚪",
            };
            println!("  {} {} → {} occurrences", emoji, level, count);
        }

        println!("\n🏷️ Error Counts by Component:");
        for (target, count) in &self.target_counts {
            println!("  {} → {} occurrences", target, count);
        }

        // Show alerts
        let alerts = self.check_alert_conditions();
        if !alerts.is_empty() {
            println!("\n🚨 Active Alerts:");
            for alert in alerts {
                println!("  ⚠️  {}", alert);
            }
        } else {
            println!("\n✅ No active alerts");
        }
    }
}

fn main() {
    // Initialize LogFFI for automatic error logging
    logffi::info!("Starting error monitoring dashboard demo");

    let mut metrics = ErrorMetricsCollector::new();

    // Simulate various errors occurring in the system
    println!("🔄 Simulating application errors...\n");

    // API errors
    let api_errors = vec![
        ApiError::DatabaseTimeout {
            query: "SELECT * FROM users WHERE active = true".to_string(),
            duration_ms: 5000,
        },
        ApiError::DatabaseTimeout {
            query: "UPDATE orders SET status = 'completed'".to_string(),
            duration_ms: 3000,
        },
        ApiError::RateLimited {
            user_id: 12345,
            limit: 100,
        },
        ApiError::ValidationFailed {
            field: "email".to_string(),
            value: "not-an-email".to_string(),
        },
        ApiError::Unauthorized {
            token: "invalid-jwt-token".to_string(),
        },
    ];

    // System errors
    let system_errors = vec![
        SystemError::DiskFull {
            path: "/var/log".to_string(),
            available_mb: 50,
        },
        SystemError::MemoryPressure {
            used_mb: 7500,
            total_mb: 8192,
        },
        SystemError::ServiceDown {
            service: "redis".to_string(),
        },
    ];

    // Record all errors
    for error in &api_errors {
        metrics.record_api_error(error);
    }

    for error in &system_errors {
        metrics.record_system_error(error);
    }

    println!("\n");

    // Display the dashboard
    metrics.print_dashboard();

    // Generate Prometheus metrics for external monitoring
    println!("\n📈 Prometheus Metrics Export:");
    println!("{}", metrics.generate_prometheus_metrics());

    logffi::info!("Error monitoring dashboard demo completed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_info_extraction() {
        let error = ApiError::DatabaseTimeout {
            query: "SELECT 1".to_string(),
            duration_ms: 1000,
        };

        let (code, level, target) = error.error_info();
        assert_eq!(code, "DatabaseTimeout");
        assert_eq!(level, "error");
        assert_eq!(target, "api::db");
    }

    #[test]
    fn test_metrics_collection() {
        let mut collector = ErrorMetricsCollector::new();

        let error = SystemError::ServiceDown {
            service: "database".to_string(),
        };
        collector.record_system_error(&error);

        assert_eq!(collector.error_counts.get("ServiceDown"), Some(&1));
        assert_eq!(collector.level_counts.get("error"), Some(&1));
        assert_eq!(collector.target_counts.get("system::health"), Some(&1));
    }

    #[test]
    fn test_alert_conditions() {
        let mut collector = ErrorMetricsCollector::new();

        // Simulate multiple database timeouts
        for _ in 0..6 {
            let error = ApiError::DatabaseTimeout {
                query: "test query".to_string(),
                duration_ms: 2000,
            };
            collector.record_api_error(&error);
        }

        let alerts = collector.check_alert_conditions();
        assert!(
            alerts
                .iter()
                .any(|alert| alert.contains("DATABASE_PERFORMANCE"))
        );
    }
}
