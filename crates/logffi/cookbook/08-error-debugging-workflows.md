# Error Debugging Workflows

This cookbook entry demonstrates how to use LogFFI's `error_info()` method for debugging production issues, building error aggregation tools, and analyzing error patterns for code quality improvements.

## Overview

Effective error debugging requires systematic approaches to:

- 🔍 Analyzing error patterns and trends
- 🛠️ Building development debugging tools
- 📊 Creating error aggregation systems
- 🎯 Identifying root causes quickly

## Production Error Analysis

### Error Pattern Analysis

Use `error_info()` to identify recurring patterns:

```rust
use logffi::define_errors;
use std::collections::BTreeMap;

define_errors! {
    ProductionError {
        DatabaseTimeout { query: String, duration_ms: u64 } : "Database timeout: {query} ({duration_ms}ms)" [level = error, target = "prod::db"],
        ApiRateLimit { endpoint: String } : "Rate limit: {endpoint}" [level = warn, target = "prod::api"],
        AuthFailure { user_id: u64, reason: String } : "Auth failed for {user_id}: {reason}" [level = warn, target = "prod::auth"],
        ServiceDown { service: String, status: u16 } : "Service {service} down (HTTP {status})" [level = error, target = "prod::service"]
    }
}

/// Analyze error patterns across multiple errors
pub fn analyze_error_patterns(errors: &[ProductionError]) {
    let mut patterns = BTreeMap::new();
    
    for error in errors {
        let (code, level, target) = error.error_info();
        let pattern_key = format!("{}::{}", target, level);
        *patterns.entry(pattern_key).or_insert(0) += 1;
    }
    
    println!("📊 Error Pattern Analysis:");
    for (pattern, count) in patterns {
        println!("  {} -> {} occurrences", pattern, count);
    }
}
```

### Root Cause Investigation

Build debugging insights from error metadata:

```rust
pub fn investigate_root_causes(errors: &[ProductionError]) {
    println!("🔍 Root Cause Investigation:");
    
    let mut investigations = Vec::new();
    
    // Database performance analysis
    let db_timeouts: Vec<_> = errors.iter()
        .filter(|e| matches!(e, ProductionError::DatabaseTimeout { .. }))
        .collect();
    
    if !db_timeouts.is_empty() {
        investigations.push(format!(
            "🗄️  DATABASE PERFORMANCE: {} timeout(s) detected",
            db_timeouts.len()
        ));
        investigations.push("   → Check database connection pool settings".to_string());
        investigations.push("   → Analyze slow query logs".to_string());
        investigations.push("   → Verify database server health metrics".to_string());
    }
    
    // Authentication security analysis
    let auth_failures: Vec<_> = errors.iter()
        .filter(|e| matches!(e, ProductionError::AuthFailure { .. }))
        .collect();
        
    if auth_failures.len() > 3 {
        investigations.push(format!(
            "🔒 SECURITY CONCERN: {} authentication failures",
            auth_failures.len()
        ));
        investigations.push("   → Review authentication logs for patterns".to_string());
        investigations.push("   → Check for brute force attacks".to_string());
        investigations.push("   → Verify JWT token expiration settings".to_string());
    }
    
    // Service availability analysis  
    let service_downs: Vec<_> = errors.iter()
        .filter(|e| matches!(e, ProductionError::ServiceDown { .. }))
        .collect();
        
    if !service_downs.is_empty() {
        investigations.push(format!(
            "🔧 SERVICE RELIABILITY: {} service outages",
            service_downs.len()
        ));
        investigations.push("   → Check dependent service health".to_string());
        investigations.push("   → Verify network connectivity".to_string());
        investigations.push("   → Review circuit breaker configurations".to_string());
    }
    
    for investigation in investigations {
        println!("  {}", investigation);
    }
}
```

## Development Debugging Tools

### Interactive Error Explorer

Build tools to explore errors interactively:

```rust
use std::io::{self, Write};

pub struct ErrorExplorer {
    errors: Vec<ProductionError>,
    current_filter: Option<String>,
}

impl ErrorExplorer {
    pub fn new(errors: Vec<ProductionError>) -> Self {
        Self {
            errors,
            current_filter: None,
        }
    }
    
    pub fn run_interactive_session(&mut self) {
        loop {
            self.print_menu();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            match input.trim() {
                "1" => self.show_error_summary(),
                "2" => self.filter_by_level(),
                "3" => self.filter_by_target(), 
                "4" => self.show_timeline(),
                "5" => self.generate_debug_report(),
                "6" => self.clear_filters(),
                "q" => break,
                _ => println!("Invalid option"),
            }
        }
    }
    
    fn print_menu(&self) {
        println!("\n🔧 Error Explorer - {} errors loaded", self.errors.len());
        if let Some(ref filter) = self.current_filter {
            println!("   Filter active: {}", filter);
        }
        println!("1. Show error summary");
        println!("2. Filter by level");
        println!("3. Filter by target");
        println!("4. Show timeline");
        println!("5. Generate debug report");
        println!("6. Clear filters");
        println!("q. Quit");
        print!("Choose option: ");
        io::stdout().flush().unwrap();
    }
    
    fn show_error_summary(&self) {
        let filtered_errors = self.get_filtered_errors();
        
        println!("\n📊 Error Summary ({} errors):", filtered_errors.len());
        
        let mut by_code = BTreeMap::new();
        let mut by_level = BTreeMap::new();
        let mut by_target = BTreeMap::new();
        
        for error in filtered_errors {
            let (code, level, target) = error.error_info();
            *by_code.entry(code).or_insert(0) += 1;
            *by_level.entry(level).or_insert(0) += 1;
            *by_target.entry(target).or_insert(0) += 1;
        }
        
        println!("\nBy Error Type:");
        for (code, count) in by_code {
            println!("  {} -> {}", code, count);
        }
        
        println!("\nBy Level:");
        for (level, count) in by_level {
            println!("  {} -> {}", level, count);
        }
        
        println!("\nBy Target:");
        for (target, count) in by_target {
            println!("  {} -> {}", target, count);
        }
    }
    
    fn get_filtered_errors(&self) -> Vec<&ProductionError> {
        if let Some(ref filter) = self.current_filter {
            self.errors.iter()
                .filter(|e| {
                    let (code, level, target) = e.error_info();
                    code.contains(filter) || level.contains(filter) || target.contains(filter)
                })
                .collect()
        } else {
            self.errors.iter().collect()
        }
    }
    
    fn filter_by_level(&mut self) {
        print!("Enter level (error/warn/info): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let level = input.trim();
        
        self.current_filter = Some(level.to_string());
        println!("Filter set to level: {}", level);
    }
    
    fn generate_debug_report(&self) {
        println!("\n📋 Debug Report:");
        println!("================");
        
        let filtered_errors = self.get_filtered_errors();
        
        // Generate actionable debugging checklist
        let mut checklist = Vec::new();
        
        for error in &filtered_errors {
            let (_code, _level, target) = error.error_info();
            
            match target {
                t if t.contains("db") => {
                    checklist.push("□ Check database connection settings");
                    checklist.push("□ Review slow query logs");
                }
                t if t.contains("api") => {
                    checklist.push("□ Verify API rate limiting config");
                    checklist.push("□ Check endpoint response times");
                }
                t if t.contains("auth") => {
                    checklist.push("□ Review authentication service logs");
                    checklist.push("□ Check for security incidents");
                }
                _ => {}
            }
        }
        
        // Remove duplicates
        checklist.sort();
        checklist.dedup();
        
        for item in checklist {
            println!("  {}", item);
        }
    }
    
    // Additional methods omitted for brevity...
}
```

### Error Correlation Engine

Correlate errors across different systems:

```rust
#[derive(Debug)]
pub struct ErrorCorrelationEngine {
    correlations: BTreeMap<String, Vec<CorrelationRule>>,
}

#[derive(Debug)]
struct CorrelationRule {
    condition: String,
    related_patterns: Vec<String>,
    impact_level: String,
    suggested_actions: Vec<String>,
}

impl ErrorCorrelationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            correlations: BTreeMap::new(),
        };
        
        engine.setup_default_correlations();
        engine
    }
    
    fn setup_default_correlations(&mut self) {
        // Database timeout correlations
        self.correlations.insert(
            "DatabaseTimeout".to_string(),
            vec![
                CorrelationRule {
                    condition: "Multiple database timeouts".to_string(),
                    related_patterns: vec![
                        "ServiceDown".to_string(),
                        "ApiRateLimit".to_string(),
                    ],
                    impact_level: "High".to_string(),
                    suggested_actions: vec![
                        "Check database server load".to_string(),
                        "Review connection pool settings".to_string(),
                        "Scale database resources".to_string(),
                    ],
                },
            ],
        );
        
        // Authentication failure correlations
        self.correlations.insert(
            "AuthFailure".to_string(), 
            vec![
                CorrelationRule {
                    condition: "Spike in auth failures".to_string(),
                    related_patterns: vec![
                        "ServiceDown".to_string(),
                    ],
                    impact_level: "Medium".to_string(),
                    suggested_actions: vec![
                        "Check for security attacks".to_string(),
                        "Verify auth service health".to_string(),
                        "Review rate limiting rules".to_string(),
                    ],
                },
            ],
        );
    }
    
    pub fn analyze_correlations(&self, errors: &[ProductionError]) -> CorrelationReport {
        let mut report = CorrelationReport::new();
        
        // Count error types
        let mut error_counts = BTreeMap::new();
        for error in errors {
            let (code, _level, _target) = error.error_info();
            *error_counts.entry(code.to_string()).or_insert(0) += 1;
        }
        
        // Check for correlations
        for (error_type, rules) in &self.correlations {
            if let Some(&count) = error_counts.get(error_type) {
                if count >= 2 {  // Threshold for correlation
                    for rule in rules {
                        // Check if related patterns exist
                        let related_found = rule.related_patterns.iter()
                            .any(|pattern| error_counts.contains_key(pattern));
                        
                        if related_found || count >= 3 {
                            report.add_correlation(
                                error_type.clone(),
                                rule.condition.clone(),
                                rule.impact_level.clone(),
                                rule.suggested_actions.clone(),
                            );
                        }
                    }
                }
            }
        }
        
        report
    }
}

#[derive(Debug)]
pub struct CorrelationReport {
    correlations: Vec<CorrelationMatch>,
}

#[derive(Debug)]
struct CorrelationMatch {
    error_type: String,
    condition: String,
    impact_level: String,
    actions: Vec<String>,
}

impl CorrelationReport {
    fn new() -> Self {
        Self {
            correlations: Vec::new(),
        }
    }
    
    fn add_correlation(&mut self, error_type: String, condition: String, impact_level: String, actions: Vec<String>) {
        self.correlations.push(CorrelationMatch {
            error_type,
            condition,
            impact_level,
            actions,
        });
    }
    
    pub fn print_report(&self) {
        println!("🔗 Error Correlation Analysis:");
        println!("==============================");
        
        if self.correlations.is_empty() {
            println!("  No significant correlations detected");
            return;
        }
        
        for correlation in &self.correlations {
            println!("📊 {} - {} (Impact: {})", 
                    correlation.error_type, 
                    correlation.condition,
                    correlation.impact_level);
            
            println!("   Suggested Actions:");
            for action in &correlation.actions {
                println!("   → {}", action);
            }
            println!();
        }
    }
}
```

## Error Aggregation Systems

### Real-time Error Stream Processing

Process error streams for real-time insights:

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct ErrorStreamProcessor {
    error_sender: Sender<ProductionError>,
    aggregation_window: Duration,
}

impl ErrorStreamProcessor {
    pub fn new(aggregation_window: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        // Spawn aggregation worker
        thread::spawn(move || {
            Self::run_aggregation_worker(receiver, aggregation_window);
        });
        
        Self {
            error_sender: sender,
            aggregation_window,
        }
    }
    
    pub fn process_error(&self, error: ProductionError) {
        // Send to aggregation worker
        let _ = self.error_sender.send(error);
    }
    
    fn run_aggregation_worker(receiver: Receiver<ProductionError>, window: Duration) {
        let mut error_buffer = Vec::new();
        let mut last_aggregation = std::time::Instant::now();
        
        loop {
            // Try to receive errors with timeout
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(error) => {
                    error_buffer.push(error);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if aggregation window has passed
                    if last_aggregation.elapsed() >= window {
                        if !error_buffer.is_empty() {
                            Self::perform_aggregation(&error_buffer);
                            error_buffer.clear();
                            last_aggregation = std::time::Instant::now();
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    
    fn perform_aggregation(errors: &[ProductionError]) {
        println!("\n⚡ Real-time Error Aggregation ({} errors in window):", errors.len());
        
        // Quick pattern analysis
        analyze_error_patterns(errors);
        
        // Generate immediate insights
        let critical_count = errors.iter()
            .filter(|e| e.error_info().1 == "error")
            .count();
        
        if critical_count > 0 {
            println!("🚨 ALERT: {} critical errors in aggregation window", critical_count);
        }
        
        // Check for error spikes
        if errors.len() > 10 {
            println!("📈 ERROR SPIKE: Unusual error volume detected");
        }
    }
}
```

### Historical Error Analysis

Analyze historical error data for trends:

```rust
pub struct HistoricalAnalyzer {
    error_history: Vec<(u64, ProductionError)>, // (timestamp, error)
}

impl HistoricalAnalyzer {
    pub fn analyze_trends(&self, days: u32) -> TrendAnalysis {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let cutoff = now - (days as u64 * 24 * 3600);
        
        let recent_errors: Vec<_> = self.error_history.iter()
            .filter(|(timestamp, _)| *timestamp >= cutoff)
            .map(|(_, error)| error)
            .collect();
        
        let mut trend_analysis = TrendAnalysis::new();
        
        // Analyze daily trends
        for day in 0..days {
            let day_start = cutoff + (day as u64 * 24 * 3600);
            let day_end = day_start + (24 * 3600);
            
            let day_errors: Vec<_> = self.error_history.iter()
                .filter(|(timestamp, _)| *timestamp >= day_start && *timestamp < day_end)
                .map(|(_, error)| error)
                .collect();
            
            trend_analysis.add_day_data(day, day_errors.len());
        }
        
        // Identify trending error types
        let mut error_type_trends = BTreeMap::new();
        for error in &recent_errors {
            let (code, _level, _target) = error.error_info();
            *error_type_trends.entry(code.to_string()).or_insert(0) += 1;
        }
        
        trend_analysis.set_error_type_trends(error_type_trends);
        trend_analysis
    }
}

#[derive(Debug)]
pub struct TrendAnalysis {
    daily_counts: Vec<(u32, usize)>, // (day, count)
    error_type_trends: BTreeMap<String, usize>,
}

impl TrendAnalysis {
    fn new() -> Self {
        Self {
            daily_counts: Vec::new(),
            error_type_trends: BTreeMap::new(),
        }
    }
    
    fn add_day_data(&mut self, day: u32, count: usize) {
        self.daily_counts.push((day, count));
    }
    
    fn set_error_type_trends(&mut self, trends: BTreeMap<String, usize>) {
        self.error_type_trends = trends;
    }
    
    pub fn print_trend_report(&self) {
        println!("📈 Historical Trend Analysis:");
        println!("=============================");
        
        println!("\nDaily Error Counts:");
        for (day, count) in &self.daily_counts {
            println!("  Day {} -> {} errors", day, count);
        }
        
        println!("\nTop Error Types:");
        let mut sorted_trends: Vec<_> = self.error_type_trends.iter().collect();
        sorted_trends.sort_by(|a, b| b.1.cmp(a.1));
        
        for (error_type, count) in sorted_trends.iter().take(5) {
            println!("  {} -> {} occurrences", error_type, count);
        }
        
        // Identify trends
        if self.daily_counts.len() >= 2 {
            let recent_avg = self.daily_counts.iter()
                .rev()
                .take(3)
                .map(|(_, count)| *count)
                .sum::<usize>() as f64 / 3.0;
                
            let older_avg = self.daily_counts.iter()
                .take(self.daily_counts.len().saturating_sub(3))
                .map(|(_, count)| *count)
                .sum::<usize>() as f64 / (self.daily_counts.len() - 3).max(1) as f64;
            
            let trend_change = (recent_avg - older_avg) / older_avg * 100.0;
            
            if trend_change.abs() > 20.0 {
                if trend_change > 0.0 {
                    println!("\n📊 TREND: Error rate increased by {:.1}% recently", trend_change);
                } else {
                    println!("\n📊 TREND: Error rate decreased by {:.1}% recently", -trend_change);
                }
            }
        }
    }
}
```

## Code Quality Analysis

### Error Hotspot Detection

Identify code areas with frequent errors:

```rust
pub fn detect_error_hotspots(errors: &[ProductionError]) {
    println!("🔥 Error Hotspot Analysis:");
    println!("==========================");
    
    let mut target_analysis = BTreeMap::new();
    
    for error in errors {
        let (_code, level, target) = error.error_info();
        
        let entry = target_analysis.entry(target.to_string())
            .or_insert(HotspotMetrics::new());
            
        entry.total_errors += 1;
        
        match level {
            "error" => entry.critical_errors += 1,
            "warn" => entry.warning_errors += 1,
            _ => entry.info_errors += 1,
        }
    }
    
    // Sort by total errors
    let mut hotspots: Vec<_> = target_analysis.into_iter().collect();
    hotspots.sort_by(|a, b| b.1.total_errors.cmp(&a.1.total_errors));
    
    println!("\nTop Error Hotspots:");
    for (i, (target, metrics)) in hotspots.iter().take(5).enumerate() {
        println!("  {}. {} ({} total errors)", i + 1, target, metrics.total_errors);
        println!("     Critical: {}, Warnings: {}, Info: {}", 
                metrics.critical_errors, 
                metrics.warning_errors, 
                metrics.info_errors);
        
        // Calculate error severity score
        let severity_score = metrics.critical_errors * 3 + metrics.warning_errors * 2 + metrics.info_errors;
        
        if severity_score > 10 {
            println!("     🔥 HIGH PRIORITY: Severity score {}", severity_score);
        }
        
        println!();
    }
}

#[derive(Debug)]
struct HotspotMetrics {
    total_errors: usize,
    critical_errors: usize,
    warning_errors: usize,
    info_errors: usize,
}

impl HotspotMetrics {
    fn new() -> Self {
        Self {
            total_errors: 0,
            critical_errors: 0,
            warning_errors: 0,
            info_errors: 0,
        }
    }
}
```

## Best Practices

### 1. Systematic Debugging Approach

Follow a structured debugging methodology:

```rust
pub fn systematic_debug_workflow(errors: &[ProductionError]) {
    println!("🔬 Systematic Debug Workflow:");
    println!("=============================");
    
    // Step 1: Initial assessment
    println!("\n1️⃣ Initial Assessment:");
    analyze_error_patterns(errors);
    
    // Step 2: Prioritization
    println!("\n2️⃣ Prioritization:");
    detect_error_hotspots(errors);
    
    // Step 3: Root cause analysis
    println!("\n3️⃣ Root Cause Analysis:");
    investigate_root_causes(errors);
    
    // Step 4: Correlation analysis
    println!("\n4️⃣ Correlation Analysis:");
    let engine = ErrorCorrelationEngine::new();
    let report = engine.analyze_correlations(errors);
    report.print_report();
    
    // Step 5: Action plan
    println!("\n5️⃣ Recommended Action Plan:");
    generate_action_plan(errors);
}

fn generate_action_plan(errors: &[ProductionError]) {
    let mut actions = Vec::new();
    
    // Analyze error patterns for specific actions
    let error_counts: BTreeMap<String, usize> = errors.iter()
        .fold(BTreeMap::new(), |mut acc, error| {
            let (code, _level, _target) = error.error_info();
            *acc.entry(code.to_string()).or_insert(0) += 1;
            acc
        });
    
    for (error_type, count) in error_counts {
        match error_type.as_str() {
            "DatabaseTimeout" if count >= 3 => {
                actions.push("🗄️  IMMEDIATE: Investigate database performance");
                actions.push("   → Check slow query logs");
                actions.push("   → Review connection pool configuration");
            }
            "AuthFailure" if count >= 5 => {
                actions.push("🔒 URGENT: Review authentication security");
                actions.push("   → Check for brute force attacks");
                actions.push("   → Verify rate limiting effectiveness");
            }
            "ServiceDown" => {
                actions.push("🔧 HIGH: Address service reliability");
                actions.push("   → Implement circuit breakers");
                actions.push("   → Add health check monitoring");
            }
            _ => {}
        }
    }
    
    if actions.is_empty() {
        actions.push("✅ No immediate critical actions required");
        actions.push("   → Continue monitoring error patterns");
    }
    
    for action in actions {
        println!("  {}", action);
    }
}
```

### 2. Debugging Checklist Template

Create reusable debugging checklists:

```rust
pub fn generate_debugging_checklist(errors: &[ProductionError]) -> Vec<String> {
    let mut checklist = Vec::new();
    
    // Universal checks
    checklist.push("□ Check system resource utilization (CPU, memory, disk)".to_string());
    checklist.push("□ Verify network connectivity between services".to_string());
    checklist.push("□ Review recent deployments and configuration changes".to_string());
    
    // Error-specific checks
    let has_db_errors = errors.iter().any(|e| e.error_info().2.contains("db"));
    let has_api_errors = errors.iter().any(|e| e.error_info().2.contains("api"));
    let has_auth_errors = errors.iter().any(|e| e.error_info().0.contains("Auth"));
    
    if has_db_errors {
        checklist.push("□ Check database connection pool status".to_string());
        checklist.push("□ Review database performance metrics".to_string());
        checklist.push("□ Verify database disk space and memory".to_string());
    }
    
    if has_api_errors {
        checklist.push("□ Check API gateway and load balancer status".to_string());
        checklist.push("□ Review API rate limiting configuration".to_string());
        checklist.push("□ Verify API endpoint response times".to_string());
    }
    
    if has_auth_errors {
        checklist.push("□ Review authentication service logs".to_string());
        checklist.push("□ Check for unusual login patterns".to_string());
        checklist.push("□ Verify token validation and expiration".to_string());
    }
    
    checklist
}
```

## See Also

- [Error Monitoring and Alerting](07-error-monitoring-and-alerting.md) - Set up comprehensive monitoring systems
- [Structured Logging](02-structured-logging.md) - Integrate errors with structured logging
- [Error Handling](03-error-handling.md) - Best practices for error handling in applications
