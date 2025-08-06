use logffi::define_errors;
use std::collections::BTreeMap;
use std::time::SystemTime;

// Example: Error analysis and debugging tools using error_info()

define_errors! {
    // Production errors from different systems
    ProductionError {
        DatabaseTimeout { query: String, duration_ms: u64 } : "Database query timed out: {query} ({duration_ms}ms)" [level = error, target = "prod::db"],
        ApiRateLimit { endpoint: String, limit: u32 } : "Rate limit exceeded on {endpoint}: {limit}/min" [level = warn, target = "prod::api"],
        AuthFailure { user_id: u64, reason: String } : "Authentication failed for user {user_id}: {reason}" [level = warn, target = "prod::auth"],
        CacheExpired { key: String } : "Cache expired for key: {key}" [level = info, target = "prod::cache"],
        ServiceUnavailable { service: String, status_code: u16 } : "Service {service} unavailable (HTTP {status_code})" [level = error, target = "prod::service"],
        ValidationFailed { field: String, constraint: String } : "Validation failed on {field}: {constraint}" [level = warn, target = "prod::validation"],
        ConfigError { key: String } : "Missing configuration: {key}" [level = error, target = "prod::config"]
    }
}

/// Analyze error patterns across multiple errors
pub fn analyze_error_patterns(errors: &[ProductionError]) {
    use std::collections::BTreeMap;

    let mut analysis = BTreeMap::new();
    for error in errors {
        let (code, level, target) = error.error_info();
        let key = format!("{}::{}::{}", target, level, code);
        *analysis.entry(key).or_insert(0) += 1;
    }

    println!("📊 Error Analysis Report:");
    println!("========================");
    for (pattern, count) in analysis {
        println!("  {} -> {} occurrences", pattern, count);
    }
}

/// Advanced error debugging with detailed breakdown
pub fn debug_error_breakdown(errors: &[ProductionError]) {
    println!("\n🔍 Detailed Error Breakdown:");
    println!("=============================");

    // Group by error type
    let mut by_type = BTreeMap::new();
    let mut by_level = BTreeMap::new();
    let mut by_target = BTreeMap::new();

    for error in errors {
        let (code, level, target) = error.error_info();

        *by_type.entry(code).or_insert(0) += 1;
        *by_level.entry(level).or_insert(0) += 1;
        *by_target.entry(target).or_insert(0) += 1;
    }

    println!("\n📋 By Error Type:");
    for (error_type, count) in by_type {
        let percentage = (count as f64 / errors.len() as f64) * 100.0;
        println!("  {} -> {} ({:.1}%)", error_type, count, percentage);
    }

    println!("\n🎯 By Severity Level:");
    for (level, count) in by_level {
        let icon = match level {
            "error" => "🔴",
            "warn" => "🟡",
            "info" => "🔵",
            _ => "⚪",
        };
        let percentage = (count as f64 / errors.len() as f64) * 100.0;
        println!("  {} {} -> {} ({:.1}%)", icon, level, count, percentage);
    }

    println!("\n🏷️ By System Component:");
    for (target, count) in by_target {
        let percentage = (count as f64 / errors.len() as f64) * 100.0;
        println!("  {} -> {} ({:.1}%)", target, count, percentage);
    }
}

/// Generate debugging insights and recommendations
pub fn generate_debug_insights(errors: &[ProductionError]) {
    println!("\n💡 Debug Insights & Recommendations:");
    println!("====================================");

    let error_counts: BTreeMap<&str, usize> =
        errors
            .iter()
            .map(|e| e.error_info().0)
            .fold(BTreeMap::new(), |mut acc, code| {
                *acc.entry(code).or_insert(0) += 1;
                acc
            });

    let level_counts: BTreeMap<&str, usize> =
        errors
            .iter()
            .map(|e| e.error_info().1)
            .fold(BTreeMap::new(), |mut acc, level| {
                *acc.entry(level).or_insert(0) += 1;
                acc
            });

    let target_counts: BTreeMap<&str, usize> =
        errors
            .iter()
            .map(|e| e.error_info().2)
            .fold(BTreeMap::new(), |mut acc, target| {
                *acc.entry(target).or_insert(0) += 1;
                acc
            });

    println!("📍 Errors by Target:");
    for (target, count) in &target_counts {
        println!("  {} → {} occurrences", target, count);
    }

    // Critical error analysis
    if let Some(&critical_count) = level_counts.get("error") {
        if critical_count > errors.len() / 4 {
            println!(
                "⚠️  HIGH PRIORITY: {}% of errors are critical-level",
                (critical_count * 100) / errors.len()
            );
            println!("   Recommendation: Investigate critical errors immediately");
        }
    }

    // Database performance analysis
    let db_timeouts = error_counts.get("DatabaseTimeout").unwrap_or(&0);
    if *db_timeouts > 0 {
        println!(
            "🗄️  DATABASE PERFORMANCE: {} timeout(s) detected",
            db_timeouts
        );
        println!("   Recommendation: Check database performance and optimize slow queries");
    }

    // Authentication security analysis
    let auth_failures = error_counts.get("AuthFailure").unwrap_or(&0);
    if *auth_failures > 2 {
        println!(
            "🔒 SECURITY CONCERN: {} authentication failure(s)",
            auth_failures
        );
        println!("   Recommendation: Review authentication logs for potential security issues");
    }

    // Rate limiting analysis
    let rate_limits = error_counts.get("ApiRateLimit").unwrap_or(&0);
    if *rate_limits > 1 {
        println!("🚦 RATE LIMITING: {} rate limit hit(s)", rate_limits);
        println!(
            "   Recommendation: Consider increasing limits or implementing better backoff strategies"
        );
    }

    // Service availability analysis
    let service_issues = error_counts.get("ServiceUnavailable").unwrap_or(&0);
    if *service_issues > 0 {
        println!(
            "🔧 SERVICE RELIABILITY: {} service unavailable error(s)",
            service_issues
        );
        println!("   Recommendation: Check service health and implement circuit breakers");
    }
}

/// Create a debugging timeline (simulated)
pub fn create_debug_timeline(errors: &[ProductionError]) {
    println!("\n⏰ Error Timeline Analysis:");
    println!("===========================");

    // Simulate timestamps for demonstration
    let base_time = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (index, error) in errors.iter().enumerate() {
        let (code, level, target) = error.error_info();
        let timestamp = base_time - (errors.len() - index) as u64 * 300; // 5 minutes apart

        let level_icon = match level {
            "error" => "🔴",
            "warn" => "🟡",
            "info" => "🔵",
            _ => "⚪",
        };

        println!(
            "  {} [{}] {} :: {} -> {}",
            level_icon, timestamp, target, code, error
        );
    }
}

/// Generate actionable debugging checklist
pub fn generate_debug_checklist(errors: &[ProductionError]) {
    println!("\n✅ Debugging Checklist:");
    println!("=======================");

    let mut checklist = Vec::new();

    // Check for database issues
    if errors.iter().any(|e| e.error_info().2.contains("db")) {
        checklist.push("□ Check database connection pool settings");
        checklist.push("□ Analyze slow query logs");
        checklist.push("□ Verify database server health metrics");
    }

    // Check for API issues
    if errors.iter().any(|e| e.error_info().2.contains("api")) {
        checklist.push("□ Review API rate limiting configuration");
        checklist.push("□ Check API endpoint response times");
        checklist.push("□ Verify load balancer health");
    }

    // Check for authentication issues
    if errors.iter().any(|e| e.error_info().0 == "AuthFailure") {
        checklist.push("□ Review authentication service logs");
        checklist.push("□ Check for suspicious login patterns");
        checklist.push("□ Verify JWT token expiration settings");
    }

    // Check for configuration issues
    if errors.iter().any(|e| e.error_info().0 == "ConfigError") {
        checklist.push("□ Validate configuration file completeness");
        checklist.push("□ Check environment variable availability");
        checklist.push("□ Verify configuration reload mechanisms");
    }

    // Check for service availability issues
    if errors
        .iter()
        .any(|e| e.error_info().0 == "ServiceUnavailable")
    {
        checklist.push("□ Check dependent service status");
        checklist.push("□ Verify network connectivity");
        checklist.push("□ Review circuit breaker configurations");
    }

    for item in checklist {
        println!("  {}", item);
    }

    if errors.len() > 10 {
        println!("  □ Consider implementing error aggregation");
        println!("  □ Set up automated alerting for error spikes");
    }
}

fn main() {
    println!("🔧 LogFFI Error Debugging Tools Demo");
    println!("====================================\n");

    // Initialize LogFFI for automatic error logging
    logffi::info!("Starting error debugging tools demo");

    // Simulate a collection of production errors
    let production_errors = vec![
        ProductionError::DatabaseTimeout {
            query: "SELECT * FROM users WHERE status = 'active'".to_string(),
            duration_ms: 5000,
        },
        ProductionError::DatabaseTimeout {
            query: "UPDATE sessions SET last_activity = NOW()".to_string(),
            duration_ms: 3000,
        },
        ProductionError::ApiRateLimit {
            endpoint: "/api/v1/users".to_string(),
            limit: 100,
        },
        ProductionError::AuthFailure {
            user_id: 12345,
            reason: "Invalid password".to_string(),
        },
        ProductionError::AuthFailure {
            user_id: 67890,
            reason: "Account locked".to_string(),
        },
        ProductionError::CacheExpired {
            key: "user_session_12345".to_string(),
        },
        ProductionError::ServiceUnavailable {
            service: "payment-processor".to_string(),
            status_code: 503,
        },
        ProductionError::ValidationFailed {
            field: "email".to_string(),
            constraint: "must be unique".to_string(),
        },
        ProductionError::ConfigError {
            key: "REDIS_URL".to_string(),
        },
        ProductionError::ApiRateLimit {
            endpoint: "/api/v1/orders".to_string(),
            limit: 50,
        },
    ];

    println!(
        "🔄 Analyzing {} production errors...\n",
        production_errors.len()
    );

    // Run all debugging tools
    analyze_error_patterns(&production_errors);
    debug_error_breakdown(&production_errors);
    generate_debug_insights(&production_errors);
    create_debug_timeline(&production_errors);
    generate_debug_checklist(&production_errors);

    logffi::info!("Error debugging tools demo completed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_error_info_extraction() {
        let error = ProductionError::DatabaseTimeout {
            query: "SELECT 1".to_string(),
            duration_ms: 1000,
        };

        let (code, level, target) = error.error_info();
        assert_eq!(code, "DatabaseTimeout");
        assert_eq!(level, "error");
        assert_eq!(target, "prod::db");
    }

    #[test]
    fn test_error_pattern_analysis() {
        let errors = vec![
            ProductionError::DatabaseTimeout {
                query: "test".to_string(),
                duration_ms: 1000,
            },
            ProductionError::DatabaseTimeout {
                query: "test2".to_string(),
                duration_ms: 2000,
            },
            ProductionError::ApiRateLimit {
                endpoint: "test".to_string(),
                limit: 100,
            },
        ];

        let mut patterns = BTreeMap::new();
        for error in &errors {
            let (code, level, target) = error.error_info();
            let key = format!("{}::{}", target, level);
            *patterns.entry(key).or_insert(0) += 1;
        }

        assert_eq!(patterns.get("prod::db::error"), Some(&2));
        assert_eq!(patterns.get("prod::api::warn"), Some(&1));
    }

    #[test]
    fn test_error_categorization() {
        let errors = vec![
            ProductionError::AuthFailure {
                user_id: 123,
                reason: "test".to_string(),
            },
            ProductionError::ConfigError {
                key: "test".to_string(),
            },
            ProductionError::CacheExpired {
                key: "test".to_string(),
            },
        ];

        let error_levels: Vec<_> = errors.iter().map(|e| e.error_info().1).collect();
        assert!(error_levels.contains(&"warn")); // AuthFailure
        assert!(error_levels.contains(&"error")); // ConfigError  
        assert!(error_levels.contains(&"info")); // CacheExpired
    }

    #[test]
    fn test_target_grouping() {
        let errors = vec![
            ProductionError::DatabaseTimeout {
                query: "test".to_string(),
                duration_ms: 1000,
            },
            ProductionError::ApiRateLimit {
                endpoint: "test".to_string(),
                limit: 100,
            },
            ProductionError::AuthFailure {
                user_id: 123,
                reason: "test".to_string(),
            },
        ];

        let targets: Vec<_> = errors.iter().map(|e| e.error_info().2).collect();
        assert!(targets.contains(&"prod::db"));
        assert!(targets.contains(&"prod::api"));
        assert!(targets.contains(&"prod::auth"));
    }
}
