//! Performance benchmarks for error_info() method
//!
//! This benchmark suite compares the performance of the error_info() method
//! against manual pattern matching for error metadata extraction.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use logfusion::define_errors;

// Define test errors for benchmarking
define_errors! {
    BenchmarkError {
        DatabaseTimeout { query: String, timeout_ms: u64 } : "Database query timed out: {query} ({timeout_ms}ms)" [level = error, target = "bench::db"],
        ApiRateLimit { endpoint: String, limit: u32 } : "Rate limit exceeded on {endpoint}: {limit}/min" [level = warn, target = "bench::api"],
        CacheExpired { key: String } : "Cache expired for key: {key}" [level = info, target = "bench::cache"],
        ValidationFailed { field: String, constraint: String } : "Validation failed on {field}: {constraint}" [level = warn, target = "bench::validation"],
        ServiceUnavailable { service: String, status_code: u16 } : "Service {service} unavailable (HTTP {status_code})" [level = error, target = "bench::service"]
    }
}

// Manual pattern matching function for comparison
fn extract_error_info_manual(error: &BenchmarkError) -> (&'static str, &'static str, &'static str) {
    match error {
        BenchmarkError::DatabaseTimeout { .. } => ("DatabaseTimeout", "error", "bench::db"),
        BenchmarkError::ApiRateLimit { .. } => ("ApiRateLimit", "warn", "bench::api"),
        BenchmarkError::CacheExpired { .. } => ("CacheExpired", "info", "bench::cache"),
        BenchmarkError::ValidationFailed { .. } => {
            ("ValidationFailed", "warn", "bench::validation")
        }
        BenchmarkError::ServiceUnavailable { .. } => {
            ("ServiceUnavailable", "error", "bench::service")
        }
    }
}

// Create sample errors for testing
fn create_sample_errors() -> Vec<BenchmarkError> {
    vec![
        BenchmarkError::DatabaseTimeout {
            query: "SELECT * FROM users WHERE active = true".to_string(),
            timeout_ms: 5000,
        },
        BenchmarkError::ApiRateLimit {
            endpoint: "/api/v1/users".to_string(),
            limit: 100,
        },
        BenchmarkError::CacheExpired {
            key: "user_session_12345".to_string(),
        },
        BenchmarkError::ValidationFailed {
            field: "email".to_string(),
            constraint: "must be unique".to_string(),
        },
        BenchmarkError::ServiceUnavailable {
            service: "payment-processor".to_string(),
            status_code: 503,
        },
    ]
}

fn bench_error_info_method(c: &mut Criterion) {
    let errors = create_sample_errors();
    let error = &errors[0]; // Use first error for single-call benchmarks

    c.bench_function("error_info_single_call", |b| {
        b.iter(|| black_box(error.error_info()))
    });

    c.bench_function("error_info_batch_processing", |b| {
        b.iter(|| {
            for error in &errors {
                black_box(error.error_info());
            }
        })
    });
}

fn bench_manual_pattern_matching(c: &mut Criterion) {
    let errors = create_sample_errors();
    let error = &errors[0]; // Use first error for single-call benchmarks

    c.bench_function("manual_extraction_single_call", |b| {
        b.iter(|| black_box(extract_error_info_manual(error)))
    });

    c.bench_function("manual_extraction_batch_processing", |b| {
        b.iter(|| {
            for error in &errors {
                black_box(extract_error_info_manual(error));
            }
        })
    });
}

fn bench_error_info_vs_manual(c: &mut Criterion) {
    let errors = create_sample_errors();

    let mut group = c.benchmark_group("error_info_comparison");

    // Benchmark error_info() method
    group.bench_function("error_info_method", |b| {
        b.iter(|| {
            for error in &errors {
                let (_code, _level, _target) = black_box(error.error_info());
            }
        })
    });

    // Benchmark manual pattern matching
    group.bench_function("manual_pattern_matching", |b| {
        b.iter(|| {
            for error in &errors {
                let (_code, _level, _target) = black_box(extract_error_info_manual(error));
            }
        })
    });

    group.finish();
}

fn bench_error_creation_overhead(c: &mut Criterion) {
    c.bench_function("error_creation_with_strings", |b| {
        b.iter(|| {
            black_box(BenchmarkError::DatabaseTimeout {
                query: "SELECT * FROM large_table LIMIT 1000".to_string(),
                timeout_ms: 30000,
            })
        })
    });

    c.bench_function("error_creation_simple", |b| {
        b.iter(|| {
            black_box(BenchmarkError::CacheExpired {
                key: "cache_key_12345".to_string(),
            })
        })
    });
}

fn bench_error_info_with_metrics_collection(c: &mut Criterion) {
    use std::collections::HashMap;

    let errors = create_sample_errors();

    c.bench_function("metrics_collection_with_error_info", |b| {
        b.iter(|| {
            let mut metrics: HashMap<String, u32> = HashMap::new();
            for error in &errors {
                let (code, level, target) = error.error_info();
                let key = format!("{}::{}::{}", target, level, code);
                *metrics.entry(key).or_insert(0) += 1;
            }
            black_box(metrics)
        })
    });

    c.bench_function("metrics_collection_with_manual_matching", |b| {
        b.iter(|| {
            let mut metrics: HashMap<String, u32> = HashMap::new();
            for error in &errors {
                let (code, level, target) = extract_error_info_manual(error);
                let key = format!("{}::{}::{}", target, level, code);
                *metrics.entry(key).or_insert(0) += 1;
            }
            black_box(metrics)
        })
    });
}

fn bench_large_scale_processing(c: &mut Criterion) {
    // Create a large set of errors for stress testing
    let mut large_error_set = Vec::new();
    for i in 0..1000 {
        large_error_set.push(BenchmarkError::ApiRateLimit {
            endpoint: format!("/api/v1/endpoint_{}", i % 10),
            limit: 100 + (i % 50) as u32,
        });
    }

    c.bench_function("large_scale_error_info_processing", |b| {
        b.iter(|| {
            for error in &large_error_set {
                black_box(error.error_info());
            }
        })
    });

    c.bench_function("large_scale_manual_processing", |b| {
        b.iter(|| {
            for error in &large_error_set {
                black_box(extract_error_info_manual(error));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_error_info_method,
    bench_manual_pattern_matching,
    bench_error_info_vs_manual,
    bench_error_creation_overhead,
    bench_error_info_with_metrics_collection,
    bench_large_scale_processing
);

criterion_main!(benches);
