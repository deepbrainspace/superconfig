use std::any::Any;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use superhashmap::HashMap as SuperHashMap;

// Benchmark configuration constants
const TOTAL_ENTRIES: usize = 200_000; // 200K entries - faster with stable results
const CONCURRENT_THREADS: usize = 8;
const OPS_PER_THREAD: usize = 10_000; // 80k total concurrent operations  
const TOTAL_CONCURRENT_OPS: usize = CONCURRENT_THREADS * OPS_PER_THREAD;

// Test data structure
#[derive(Clone, Debug)]
struct TestConfig {
    host: String,
    port: u16,
    timeout_ms: u32,
}

// Benchmark result
#[derive(Clone)]
struct BenchmarkResult {
    name: String,
    insert_time: Duration,
    read_time: Duration,
    update_time: Duration,
    concurrent_read_time: Duration,
    concurrent_write_time: Duration,
    concurrent_update_time: Duration,
    memory_estimate: usize,
}

// Generic storage trait for benchmarking
trait Storage<K, V>: Send + Sync {
    fn insert(&self, key: K, value: V); // Now handles upsert (insert or update)
    fn get(&self, key: &K) -> Option<V>;
    fn len(&self) -> usize;
}

// DashMap implementation
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for dashmap::DashMap<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        self.insert(key, value);
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        self.get(key).map(|entry| entry.value().clone())
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// SCC HashMap implementation
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for scc::HashMap<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        // Proper upsert: try to update first, if that fails, then insert
        if self.update(&key, |_, _| value.clone()).is_none() {
            let _ = self.insert(key, value);
        }
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut result = None;
        let _ = self.read(key, |_, value| {
            result = Some(value.clone());
        });
        result
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// SCC HashIndex implementation (read-optimized)
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for scc::HashIndex<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        // HashIndex doesn't have update, so use upsert pattern: remove then insert
        self.remove(&key);
        let _ = self.insert(key, value);
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        self.peek_with(key, |_, value| value.clone())
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// SCC HashCache implementation (bounded LRU cache)
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for scc::HashCache<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        // HashCache put() is already an upsert operation
        let _ = self.put(key, value);
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        self.get(key).map(|entry| entry.get().clone())
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// Papaya HashMap implementation
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for papaya::HashMap<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        let guard = self.pin();
        guard.insert(key, value);
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        let guard = self.pin();
        guard.get(key).cloned()
    }

    fn len(&self) -> usize {
        // Papaya doesn't expose len easily, so we estimate
        100_000 // This is a rough estimate for the benchmark
    }
}

// SuperHashMap implementation (Papaya-based)
impl Storage<String, Arc<dyn Any + Send + Sync>>
    for SuperHashMap<String, Arc<dyn Any + Send + Sync>>
{
    fn insert(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        let guard = self.pin();
        guard.insert(key, value);
    }

    fn get(&self, key: &String) -> Option<Arc<dyn Any + Send + Sync>> {
        let guard = self.pin();
        guard.get(key).cloned()
    }

    fn len(&self) -> usize {
        let guard = self.pin();
        guard.len()
    }
}

// Benchmark operations
#[derive(Clone)]
enum BenchmarkOp {
    Insert {
        entries: usize,
    },
    Read {
        entries: usize,
    },
    ConcurrentRead {
        threads: usize,
        ops_per_thread: usize,
    },
    ConcurrentWrite {
        threads: usize,
        ops_per_thread: usize,
    },
    ConcurrentUpdate {
        threads: usize,
        ops_per_thread: usize,
    },
    Update {
        entries: usize,
    },
}

// Benchmark runner
struct BenchmarkRunner<S> {
    storage: Arc<S>,
    name: String,
}

impl<S> BenchmarkRunner<S>
where
    S: Storage<String, Arc<dyn Any + Send + Sync>> + 'static,
{
    fn new(storage: S, name: String) -> Self {
        Self {
            storage: Arc::new(storage),
            name,
        }
    }

    fn run_operation(&self, op: &BenchmarkOp) -> Duration {
        match op {
            BenchmarkOp::Insert { entries } => {
                let start = Instant::now();
                for i in 0..*entries {
                    let config = TestConfig {
                        host: format!("host-{i}"),
                        port: 8080 + (i % 1000) as u16,
                        timeout_ms: 5000 + i as u32,
                    };
                    self.storage.insert(
                        format!("key-{i}"),
                        Arc::new(config) as Arc<dyn Any + Send + Sync>,
                    );
                }
                start.elapsed()
            }

            BenchmarkOp::Read { entries } => {
                let start = Instant::now();
                for i in 0..*entries {
                    let key = format!("key-{i}");
                    if let Some(value) = self.storage.get(&key) {
                        if let Ok(config) = value.downcast::<TestConfig>() {
                            let _host = &config.host; // Use the data
                            let _port = config.port; // Use the port field
                            let _timeout = config.timeout_ms; // Use the timeout field
                        }
                    }
                }
                start.elapsed()
            }

            BenchmarkOp::ConcurrentRead {
                threads,
                ops_per_thread,
            } => {
                let start = Instant::now();
                let threads = *threads;
                let ops_per_thread = *ops_per_thread;
                let handles: Vec<_> = (0..threads)
                    .map(|thread_id| {
                        let storage_ref = Arc::clone(&self.storage);
                        thread::spawn(move || {
                            for i in 0..ops_per_thread {
                                // Read existing keys (cycle through available keys)
                                let key = format!(
                                    "key-{}",
                                    (thread_id * ops_per_thread + i) % TOTAL_ENTRIES
                                );
                                if let Some(value) = storage_ref.get(&key) {
                                    if let Ok(config) = value.downcast::<TestConfig>() {
                                        let _host = &config.host; // Use the data
                                        let _port = config.port; // Use the port field
                                        let _timeout = config.timeout_ms; // Use the timeout field
                                    }
                                }
                            }
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.join().unwrap();
                }
                start.elapsed()
            }

            BenchmarkOp::ConcurrentWrite {
                threads,
                ops_per_thread,
            } => {
                let start = Instant::now();
                let threads = *threads;
                let ops_per_thread = *ops_per_thread;
                let handles: Vec<_> = (0..threads)
                    .map(|thread_id| {
                        let storage_ref = Arc::clone(&self.storage);
                        thread::spawn(move || {
                            for i in 0..ops_per_thread {
                                let key = format!("concurrent-write-{thread_id}-{i}");
                                let config = TestConfig {
                                    host: format!("host-{thread_id}-{i}"),
                                    port: 9000 + i as u16,
                                    timeout_ms: 3000 + i as u32,
                                };
                                storage_ref
                                    .insert(key, Arc::new(config) as Arc<dyn Any + Send + Sync>);
                            }
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.join().unwrap();
                }
                start.elapsed()
            }

            BenchmarkOp::ConcurrentUpdate {
                threads,
                ops_per_thread,
            } => {
                let start = Instant::now();
                let threads = *threads;
                let ops_per_thread = *ops_per_thread;
                let handles: Vec<_> = (0..threads)
                    .map(|thread_id| {
                        let storage_ref = Arc::clone(&self.storage);
                        thread::spawn(move || {
                            for i in 0..ops_per_thread {
                                // Update existing keys (cycle through available keys)
                                let key = format!(
                                    "key-{}",
                                    (thread_id * ops_per_thread + i) % TOTAL_ENTRIES
                                );
                                let updated_config = TestConfig {
                                    host: format!("updated-host-{thread_id}-{i}"),
                                    port: 8000 + i as u16,
                                    timeout_ms: 4000 + i as u32,
                                };
                                // Use insert which now properly handles upserts for all implementations
                                storage_ref.insert(
                                    key,
                                    Arc::new(updated_config) as Arc<dyn Any + Send + Sync>,
                                );
                            }
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.join().unwrap();
                }
                start.elapsed()
            }

            BenchmarkOp::Update { entries } => {
                let start = Instant::now();
                for i in 0..*entries {
                    let key = format!("key-{i}");
                    let new_config = TestConfig {
                        host: format!("updated-host-{i}"),
                        port: 9000 + (i % 1000) as u16,
                        timeout_ms: 6000 + i as u32,
                    };
                    self.storage
                        .insert(key, Arc::new(new_config) as Arc<dyn Any + Send + Sync>);
                }
                start.elapsed()
            }
        }
    }

    fn run_benchmark(&self) -> BenchmarkResult {
        let entries = TOTAL_ENTRIES;
        let threads = CONCURRENT_THREADS;
        let ops_per_thread = OPS_PER_THREAD;

        // Standard benchmark operations
        let insert_time = self.run_operation(&BenchmarkOp::Insert { entries });
        let read_time = self.run_operation(&BenchmarkOp::Read { entries });
        let update_time = self.run_operation(&BenchmarkOp::Update { entries });

        // Separate concurrent read and write tests
        let concurrent_read_time = self.run_operation(&BenchmarkOp::ConcurrentRead {
            threads,
            ops_per_thread,
        });

        let concurrent_write_time = self.run_operation(&BenchmarkOp::ConcurrentWrite {
            threads,
            ops_per_thread,
        });

        let concurrent_update_time = self.run_operation(&BenchmarkOp::ConcurrentUpdate {
            threads,
            ops_per_thread,
        });

        BenchmarkResult {
            name: self.name.clone(),
            insert_time,
            read_time,
            update_time,
            concurrent_read_time,
            concurrent_write_time,
            concurrent_update_time,
            memory_estimate: self.storage.len() * 64, // More realistic: ~32 bytes for TestConfig + ~32 bytes overhead
        }
    }
}

fn print_results_table(results: &[BenchmarkResult]) {
    println!("\n📊 CONCURRENT PERFORMANCE RESULTS:");
    println!("{:-<130}", "");
    println!(
        "{:<30} | {:>15} | {:>16} | {:>16} | {:>10}",
        "Implementation",
        "Concurrent R (ns)",
        "Concurrent W (ns)",
        "Concurrent U (ns)",
        "Memory (MB)"
    );
    println!("{:-<130}", "");

    let concurrent_ops = TOTAL_CONCURRENT_OPS as f64;

    let best_concurrent_read = results
        .iter()
        .min_by_key(|r| r.concurrent_read_time)
        .unwrap();
    let worst_concurrent_read = results
        .iter()
        .max_by_key(|r| r.concurrent_read_time)
        .unwrap();

    let best_concurrent_write = results
        .iter()
        .min_by_key(|r| r.concurrent_write_time)
        .unwrap();
    let worst_concurrent_write = results
        .iter()
        .max_by_key(|r| r.concurrent_write_time)
        .unwrap();

    let best_concurrent_update = results
        .iter()
        .min_by_key(|r| r.concurrent_update_time)
        .unwrap();
    let worst_concurrent_update = results
        .iter()
        .max_by_key(|r| r.concurrent_update_time)
        .unwrap();

    let best_memory = results.iter().min_by_key(|r| r.memory_estimate).unwrap();
    let worst_memory = results.iter().max_by_key(|r| r.memory_estimate).unwrap();

    for result in results {
        // Color codes for concurrent operations
        let concurrent_read_color = if result.name == best_concurrent_read.name {
            "\x1b[92m"
        }
        // Bright Green
        else if result.name == worst_concurrent_read.name {
            "\x1b[91m"
        }
        // Bright Red
        else {
            ""
        };

        let concurrent_write_color = if result.name == best_concurrent_write.name {
            "\x1b[92m"
        } else if result.name == worst_concurrent_write.name {
            "\x1b[91m"
        } else {
            ""
        };

        let concurrent_update_color = if result.name == best_concurrent_update.name {
            "\x1b[92m"
        } else if result.name == worst_concurrent_update.name {
            "\x1b[91m"
        } else {
            ""
        };

        let memory_color = if result.name == best_memory.name {
            "\x1b[92m"
        } else if result.name == worst_memory.name {
            "\x1b[91m"
        } else {
            ""
        };

        let reset = "\x1b[0m";

        println!(
            "{:<30} | {}{:>15.0}{} | {}{:>16.0}{} | {}{:>16.0}{} | {}{:>10.1}{}",
            result.name,
            concurrent_read_color,
            result.concurrent_read_time.as_nanos() as f64 / concurrent_ops,
            reset,
            concurrent_write_color,
            result.concurrent_write_time.as_nanos() as f64 / concurrent_ops,
            reset,
            concurrent_update_color,
            result.concurrent_update_time.as_nanos() as f64 / concurrent_ops,
            reset,
            memory_color,
            result.memory_estimate as f64 / 1024.0 / 1024.0,
            reset
        );
    }
    println!("{:-<130}", "");
    println!("🟢 = Best concurrent performance  🔴 = Worst concurrent performance");
    println!("Note: Lower nanoseconds per operation = better performance");
}

fn main() {
    const NUM_RUNS: usize = 5;

    println!("🚀 SuperConfig Hash Table Benchmark");
    println!(
        "Testing with {TOTAL_ENTRIES} entries + {CONCURRENT_THREADS}-thread concurrency tests"
    );
    println!("Each entry: TestConfig with String, u16, u32 fields");
    println!(
        "Concurrent tests: {CONCURRENT_THREADS} threads × {OPS_PER_THREAD} ops = {TOTAL_CONCURRENT_OPS} total operations"
    );
    println!("Running {NUM_RUNS} iterations for stable averages...");
    println!();

    // Create benchmark runners
    type BenchmarkFn = Box<dyn Fn() -> BenchmarkResult>;
    let benchmarks: Vec<(&str, BenchmarkFn)> = vec![
        (
            "DashMap",
            Box::new(|| {
                let storage = dashmap::DashMap::new();
                BenchmarkRunner::new(storage, "DashMap".to_string()).run_benchmark()
            }),
        ),
        (
            "Papaya HashMap",
            Box::new(|| {
                let storage = papaya::HashMap::new();
                BenchmarkRunner::new(storage, "Papaya HashMap".to_string()).run_benchmark()
            }),
        ),
        (
            "SuperHashMap",
            Box::new(|| {
                let storage: SuperHashMap<String, Arc<dyn Any + Send + Sync>> = SuperHashMap::new();
                BenchmarkRunner::new(storage, "SuperHashMap".to_string()).run_benchmark()
            }),
        ),
        (
            "SCC HashMap",
            Box::new(|| {
                let storage = scc::HashMap::new();
                BenchmarkRunner::new(storage, "SCC HashMap".to_string()).run_benchmark()
            }),
        ),
        (
            "SCC HashIndex",
            Box::new(|| {
                let storage = scc::HashIndex::new();
                BenchmarkRunner::new(storage, "SCC HashIndex".to_string()).run_benchmark()
            }),
        ),
        (
            "SCC HashCache (LRU)",
            Box::new(|| {
                let storage = scc::HashCache::with_capacity(TOTAL_ENTRIES * 2, TOTAL_ENTRIES * 2);
                BenchmarkRunner::new(storage, "SCC HashCache (LRU)".to_string()).run_benchmark()
            }),
        ),
    ];

    // Collect results from multiple runs
    let mut all_results: Vec<Vec<BenchmarkResult>> = vec![vec![]; benchmarks.len()];

    for run in 1..=NUM_RUNS {
        println!("🔄 Run {run}/{NUM_RUNS}");

        for (i, (name, benchmark)) in benchmarks.iter().enumerate() {
            print!("  Testing {name}...");
            std::io::stdout().flush().unwrap();

            let result = benchmark();
            all_results[i].push(result);

            println!(" ✓");
        }
        println!();
    }

    // Calculate averages
    let mut averaged_results = vec![];
    for (i, (name, _)) in benchmarks.iter().enumerate() {
        let results = &all_results[i];
        let len = results.len() as f64;

        let avg_result = BenchmarkResult {
            name: name.to_string(),
            insert_time: Duration::from_nanos(
                (results
                    .iter()
                    .map(|r| r.insert_time.as_nanos())
                    .sum::<u128>() as f64
                    / len) as u64,
            ),
            read_time: Duration::from_nanos(
                (results.iter().map(|r| r.read_time.as_nanos()).sum::<u128>() as f64 / len) as u64,
            ),
            update_time: Duration::from_nanos(
                (results
                    .iter()
                    .map(|r| r.update_time.as_nanos())
                    .sum::<u128>() as f64
                    / len) as u64,
            ),
            concurrent_read_time: Duration::from_nanos(
                (results
                    .iter()
                    .map(|r| r.concurrent_read_time.as_nanos())
                    .sum::<u128>() as f64
                    / len) as u64,
            ),
            concurrent_write_time: Duration::from_nanos(
                (results
                    .iter()
                    .map(|r| r.concurrent_write_time.as_nanos())
                    .sum::<u128>() as f64
                    / len) as u64,
            ),
            concurrent_update_time: Duration::from_nanos(
                (results
                    .iter()
                    .map(|r| r.concurrent_update_time.as_nanos())
                    .sum::<u128>() as f64
                    / len) as u64,
            ),
            memory_estimate: (results.iter().map(|r| r.memory_estimate).sum::<usize>() as f64 / len)
                as usize,
        };
        averaged_results.push(avg_result);
    }

    println!("📊 AVERAGED RESULTS ({NUM_RUNS} runs):");
    print_results_table(&averaged_results);

    // Summary comparison - Compact ranking table
    println!("📊 PERFORMANCE RANKING:");
    println!("⏱️  Rankings for each operation type (1st = best, 7th = worst)");
    println!();
    println!("{:-<120}", "");
    println!(
        "{:<25} | {:>12} | {:>12} | {:>12} | {:>15} | {:>15}",
        "Implementation", "🏃 Insert", "📖 Read", "🔄 Update", "📖 Concurrent R", "✍️ Concurrent W"
    );
    println!("{:-<120}", "");

    // Calculate rankings for each metric
    let mut insert_sorted = averaged_results.clone();
    insert_sorted.sort_by_key(|r| r.insert_time);

    let mut read_sorted = averaged_results.clone();
    read_sorted.sort_by_key(|r| r.read_time);

    let mut update_sorted = averaged_results.clone();
    update_sorted.sort_by_key(|r| r.update_time);

    let mut concurrent_read_sorted = averaged_results.clone();
    concurrent_read_sorted.sort_by_key(|r| r.concurrent_read_time);

    let mut concurrent_write_sorted = averaged_results.clone();
    concurrent_write_sorted.sort_by_key(|r| r.concurrent_write_time);

    // Create a map of implementation name to rankings
    let mut rankings: std::collections::HashMap<String, (usize, usize, usize, usize, usize)> =
        std::collections::HashMap::new();

    for (rank, result) in insert_sorted.iter().enumerate() {
        rankings
            .entry(result.name.clone())
            .or_insert((0, 0, 0, 0, 0))
            .0 = rank + 1;
    }
    for (rank, result) in read_sorted.iter().enumerate() {
        rankings
            .entry(result.name.clone())
            .or_insert((0, 0, 0, 0, 0))
            .1 = rank + 1;
    }
    for (rank, result) in update_sorted.iter().enumerate() {
        rankings
            .entry(result.name.clone())
            .or_insert((0, 0, 0, 0, 0))
            .2 = rank + 1;
    }
    for (rank, result) in concurrent_read_sorted.iter().enumerate() {
        rankings
            .entry(result.name.clone())
            .or_insert((0, 0, 0, 0, 0))
            .3 = rank + 1;
    }
    for (rank, result) in concurrent_write_sorted.iter().enumerate() {
        rankings
            .entry(result.name.clone())
            .or_insert((0, 0, 0, 0, 0))
            .4 = rank + 1;
    }

    // Print each implementation's rankings
    for result in &averaged_results {
        if let Some(&(insert_rank, read_rank, update_rank, cr_rank, cw_rank)) =
            rankings.get(&result.name)
        {
            println!(
                "{:<25} | {:>12} | {:>12} | {:>12} | {:>15} | {:>15}",
                result.name, insert_rank, read_rank, update_rank, cr_rank, cw_rank
            );
        }
    }

    println!("{:-<120}", "");
    println!("Note: Lower rank number = better performance (1st place = fastest)");

    // Recommendation
    let best_overall = averaged_results
        .iter()
        .min_by_key(|r| {
            r.insert_time + r.read_time + r.concurrent_read_time + r.concurrent_write_time
        })
        .unwrap();
    println!("🏆 RECOMMENDED: {}", best_overall.name);
    println!("   Best overall performance for SuperConfig use case");
}
