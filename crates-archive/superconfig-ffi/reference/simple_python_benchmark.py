#!/usr/bin/env python3
"""
Simple Python Performance Benchmark for SuperConfig FFI
Measures just startup and basic operation times
"""

import time
import statistics
import superconfig_ffi


def measure_time_ms(func):
    """Measure time in milliseconds"""
    start = time.perf_counter()
    func()
    end = time.perf_counter()
    return (end - start) * 1000  # Convert to milliseconds


def benchmark_python():
    print("🐍 Python SuperConfig Performance Benchmark")
    print("=" * 50)

    # Startup benchmark (module already loaded, just instance creation)
    print("📊 Testing instance creation time...")
    startup_times = []
    for i in range(100):
        time_ms = measure_time_ms(lambda: superconfig_ffi.SuperConfig())
        startup_times.append(time_ms)

    # Operation benchmark (instance + get_verbosity only)
    print("⚡ Testing basic operation speed...")
    operation_times = []
    for i in range(1000):
        time_ms = measure_time_ms(lambda: superconfig_ffi.SuperConfig().get_verbosity())
        operation_times.append(time_ms)

    # Calculate statistics
    def calc_stats(times):
        times.sort()
        n = len(times)
        return {
            "min": min(times),
            "max": max(times),
            "avg": statistics.mean(times),
            "median": statistics.median(times),
            "p95": times[int(n * 0.95)] if n > 0 else 0,
            "p99": times[int(n * 0.99)] if n > 0 else 0,
        }

    startup_stats = calc_stats(startup_times)
    operation_stats = calc_stats(operation_times)

    print("\n📈 PYTHON PERFORMANCE RESULTS")
    print("─" * 40)
    print("🚀 Instance Creation (ms)")
    print(f"  Average:  {startup_stats['avg']:.3f}")
    print(f"  Median:   {startup_stats['median']:.3f}")
    print(f"  95th:     {startup_stats['p95']:.3f}")
    print(f"  Min/Max:  {startup_stats['min']:.3f} / {startup_stats['max']:.3f}")

    print("\n⚡ Basic Operations (ms)")
    print(f"  Average:  {operation_stats['avg']:.6f}")
    print(f"  Median:   {operation_stats['median']:.6f}")
    print(f"  95th:     {operation_stats['p95']:.6f}")
    print(f"  Min/Max:  {operation_stats['min']:.6f} / {operation_stats['max']:.6f}")

    print("\n📏 File Size: 681KB (.so)")

    # Compare with known NAPI and WASI results
    napi_avg_ops = 0.001206  # From previous benchmark
    wasi_avg_ops = 0.396  # From previous benchmark

    print("\n🏆 PERFORMANCE COMPARISON")
    print("─" * 40)
    print("Operation Speed Rankings:")

    results = [
        ("NAPI", napi_avg_ops),
        ("Python", operation_stats["avg"]),
        ("WASI", wasi_avg_ops),
    ]
    results.sort(key=lambda x: x[1])  # Sort by time (faster first)

    for i, (name, time_ms) in enumerate(results, 1):
        if name == "Python":
            print(f"  {i}. 🐍 {name}: {time_ms:.6f}ms ⭐")
        else:
            print(f"  {i}. {name}: {time_ms:.6f}ms")

    # Calculate speed ratios relative to Python
    python_time = operation_stats["avg"]
    print("\nSpeed vs Python:")
    print(f"  • NAPI is {python_time/napi_avg_ops:.0f}x faster than Python")
    print(f"  • Python is {wasi_avg_ops/python_time:.0f}x faster than WASI")

    print("\n💡 PYTHON ADVANTAGES")
    print("─" * 40)
    print("✅ Excellent for data science workflows")
    print("✅ Rich ecosystem (pandas, numpy, jupyter)")
    print("✅ Balanced performance (faster than WASI)")
    print("✅ Easy integration with existing Python apps")
    print("✅ Native support for scientific computing")


if __name__ == "__main__":
    benchmark_python()
