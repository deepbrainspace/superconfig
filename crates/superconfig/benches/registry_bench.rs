use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use superconfig::*;

#[derive(Debug, Clone, PartialEq)]
struct BenchConfig {
    host: String,
    port: u16,
    timeout_ms: u32,
    max_connections: u64,
    ssl_enabled: bool,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
            max_connections: 1000,
            ssl_enabled: true,
        }
    }
}

fn bench_create_operations(c: &mut Criterion) {
    let registry = ConfigRegistry::new();

    c.bench_function("registry_create", |b| {
        let mut counter = 0u32;
        b.iter(|| {
            let config = BenchConfig {
                port: 8080 + (counter % 1000) as u16,
                ..Default::default()
            };
            counter += 1;

            let handle = registry.create(black_box(config)).unwrap();
            black_box(handle)
        })
    });
}

fn bench_read_operations(c: &mut Criterion) {
    let registry = ConfigRegistry::new();
    let handle = registry.create(BenchConfig::default()).unwrap();

    c.bench_function("registry_read", |b| {
        b.iter(|| {
            let config = registry.read(black_box(&handle)).unwrap();
            black_box(config)
        })
    });
}

fn bench_update_operations(c: &mut Criterion) {
    let registry = ConfigRegistry::new();
    let handle = registry.create(BenchConfig::default()).unwrap();

    c.bench_function("registry_update", |b| {
        let mut counter = 0u32;
        b.iter(|| {
            let new_config = BenchConfig {
                port: 9000 + (counter % 1000) as u16,
                ..Default::default()
            };
            counter += 1;

            registry
                .update(black_box(&handle), black_box(new_config))
                .unwrap();
        })
    });
}

fn bench_concurrent_reads(c: &mut Criterion) {
    let registry = Arc::new(ConfigRegistry::new());
    let handle = registry.create(BenchConfig::default()).unwrap();

    c.bench_function("concurrent_reads_10_threads", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let registry = Arc::clone(&registry);
                    let handle = handle.clone();
                    thread::spawn(move || {
                        for _ in 0..100 {
                            let _config = registry.read(&handle).unwrap();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

fn bench_mixed_operations(c: &mut Criterion) {
    c.bench_function("mixed_operations", |b| {
        b.iter(|| {
            let registry = ConfigRegistry::new();

            // Create 10 configs
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let config = BenchConfig {
                        port: 8000 + i as u16,
                        ..Default::default()
                    };
                    registry.create(config).unwrap()
                })
                .collect();

            // Read each config 5 times
            for handle in &handles {
                for _ in 0..5 {
                    let _config = registry.read(handle).unwrap();
                }
            }

            // Update half of them
            for handle in handles.iter().take(5) {
                let new_config = BenchConfig {
                    port: 9000,
                    ..Default::default()
                };
                registry.update(handle, new_config).unwrap();
            }

            // Delete some
            for handle in handles.iter().take(3) {
                let _deleted = registry.delete(handle).unwrap();
            }

            black_box(registry)
        })
    });
}

fn bench_arc_sharing_efficiency(c: &mut Criterion) {
    let registry = ConfigRegistry::new();
    let handle = registry.create(BenchConfig::default()).unwrap();

    c.bench_function("arc_sharing_100_refs", |b| {
        b.iter(|| {
            let mut refs = Vec::with_capacity(100);

            // Create 100 Arc references
            for _ in 0..100 {
                let config_ref = registry.read(black_box(&handle)).unwrap();
                refs.push(config_ref);
            }

            // Access data through each reference
            for config_ref in &refs {
                black_box(&config_ref.host);
                black_box(config_ref.port);
            }

            black_box(refs)
        })
    });
}

fn bench_handle_serialization(c: &mut Criterion) {
    let registry = ConfigRegistry::new();
    let handle = registry.create(BenchConfig::default()).unwrap();

    c.bench_function("handle_serialize", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&handle)).unwrap();
            black_box(serialized)
        })
    });

    let serialized = serde_json::to_string(&handle).unwrap();
    c.bench_function("handle_deserialize", |b| {
        b.iter(|| {
            let handle: ConfigHandle<BenchConfig> =
                serde_json::from_str(black_box(&serialized)).unwrap();
            black_box(handle)
        })
    });
}

fn bench_memory_efficiency(c: &mut Criterion) {
    c.bench_function("memory_usage_1000_configs", |b| {
        b.iter(|| {
            let registry = ConfigRegistry::new();

            // Create 1000 configs
            let handles: Vec<_> = (0..1000)
                .map(|i| {
                    let config = BenchConfig {
                        host: format!("host-{}", i),
                        port: 8000 + (i % 100) as u16,
                        ..Default::default()
                    };
                    registry.create(config).unwrap()
                })
                .collect();

            // Read from random handles
            for i in (0..1000).step_by(10) {
                let _config = registry.read(&handles[i]).unwrap();
            }

            let stats = registry.stats();
            black_box((handles, stats))
        })
    });
}

criterion_group!(
    registry_benches,
    bench_create_operations,
    bench_read_operations,
    bench_update_operations,
    bench_concurrent_reads,
    bench_mixed_operations,
    bench_arc_sharing_efficiency,
    bench_handle_serialization,
    bench_memory_efficiency
);

criterion_main!(registry_benches);
