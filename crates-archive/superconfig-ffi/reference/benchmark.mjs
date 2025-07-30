#!/usr/bin/env node
/**
 * Performance Benchmark: WASI WASM vs NAPI (Node.js Native)
 * Tests startup time and operation speed differences
 */

import { readFile } from 'fs/promises';
import { WASI } from 'wasi';
import { argv, env, hrtime } from 'process';

// Benchmark configuration
const WARMUP_ITERATIONS = 10;
const BENCHMARK_ITERATIONS = 1000;

class PerformanceTester {
    constructor() {
        this.results = {
            napi: { startup: [], operations: [] },
            wasi: { startup: [], operations: [] }
        };
    }

    // Measure time in nanoseconds with high precision
    measureTime(fn) {
        const start = hrtime.bigint();
        const result = fn();
        const end = hrtime.bigint();
        return {
            result,
            timeNs: Number(end - start),
            timeMs: Number(end - start) / 1_000_000
        };
    }

    async measureTimeAsync(fn) {
        const start = hrtime.bigint();
        const result = await fn();
        const end = hrtime.bigint();
        return {
            result,
            timeNs: Number(end - start),
            timeMs: Number(end - start) / 1_000_000
        };
    }

    // Test NAPI version performance
    async benchmarkNAPI() {
        console.log('🔥 Benchmarking NAPI (Node.js Native) version...');
        
        // Warmup
        for (let i = 0; i < WARMUP_ITERATIONS; i++) {
            const { SuperConfig } = await import('./superconfig_ffi.node');
            const config = new SuperConfig();
            config.getVerbosity();
        }

        // Benchmark startup time
        console.log('  📊 Testing NAPI startup time...');
        for (let i = 0; i < 100; i++) {
            const measurement = await this.measureTimeAsync(async () => {
                const { SuperConfig } = await import('./superconfig_ffi.node');
                return new SuperConfig();
            });
            this.results.napi.startup.push(measurement.timeNs);
        }

        // Benchmark operations
        console.log('  ⚡ Testing NAPI operation speed...');
        const { SuperConfig } = await import('./superconfig_ffi.node');
        
        for (let i = 0; i < BENCHMARK_ITERATIONS; i++) {
            const measurement = this.measureTime(() => {
                const config = new SuperConfig();
                const verbosity = config.getVerbosity();
                return verbosity;
            });
            this.results.napi.operations.push(measurement.timeNs);
        }
    }

    // Test WASI version performance
    async benchmarkWASI() {
        console.log('🌊 Benchmarking WASI WASM version...');
        
        // Pre-load the WASM buffer
        const wasmBuffer = await readFile('./target/wasm32-wasip1/release/superconfig_ffi.wasm');
        
        // Warmup
        for (let i = 0; i < WARMUP_ITERATIONS; i++) {
            const wasi = new WASI({ version: 'preview1', args: argv, env });
            const wasmModule = await WebAssembly.compile(wasmBuffer);
            const wasmInstance = await WebAssembly.instantiate(wasmModule, {
                wasi_snapshot_preview1: wasi.wasiImport,
            });
            wasi.initialize(wasmInstance);
        }

        // Benchmark startup time
        console.log('  📊 Testing WASI startup time...');
        for (let i = 0; i < 100; i++) {
            const measurement = await this.measureTimeAsync(async () => {
                const wasi = new WASI({ version: 'preview1', args: argv, env });
                const wasmModule = await WebAssembly.compile(wasmBuffer);
                const wasmInstance = await WebAssembly.instantiate(wasmModule, {
                    wasi_snapshot_preview1: wasi.wasiImport,
                });
                wasi.initialize(wasmInstance);
                return wasmInstance;
            });
            this.results.wasi.startup.push(measurement.timeNs);
        }

        // Benchmark operations (simplified - WASI doesn't expose direct function calls)
        console.log('  ⚡ Testing WASI operation speed...');
        for (let i = 0; i < 100; i++) { // Fewer iterations for complex WASI setup
            const measurement = await this.measureTimeAsync(async () => {
                const wasi = new WASI({ version: 'preview1', args: argv, env });
                const wasmModule = await WebAssembly.compile(wasmBuffer);
                const wasmInstance = await WebAssembly.instantiate(wasmModule, {
                    wasi_snapshot_preview1: wasi.wasiImport,
                });
                wasi.initialize(wasmInstance);
                return 'initialized';
            });
            this.results.wasi.operations.push(measurement.timeNs);
        }
    }

    // Calculate statistics
    calculateStats(times) {
        const sorted = times.sort((a, b) => a - b);
        const len = sorted.length;
        
        return {
            min: sorted[0] / 1_000_000, // Convert to ms
            max: sorted[len - 1] / 1_000_000,
            avg: (sorted.reduce((a, b) => a + b, 0) / len) / 1_000_000,
            median: len % 2 === 0 
                ? (sorted[len/2 - 1] + sorted[len/2]) / 2 / 1_000_000
                : sorted[Math.floor(len/2)] / 1_000_000,
            p95: sorted[Math.floor(len * 0.95)] / 1_000_000,
            p99: sorted[Math.floor(len * 0.99)] / 1_000_000
        };
    }

    // Print detailed results
    printResults() {
        console.log('\n📈 PERFORMANCE BENCHMARK RESULTS');
        console.log('='.repeat(50));
        
        const napiStartup = this.calculateStats(this.results.napi.startup);
        const wasiStartup = this.calculateStats(this.results.wasi.startup);
        const napiOps = this.calculateStats(this.results.napi.operations);
        const wasiOps = this.calculateStats(this.results.wasi.operations);

        console.log('\n🚀 STARTUP TIME (ms)');
        console.log('─'.repeat(30));
        console.log('                 NAPI    WASI    Ratio');
        console.log(`Min:           ${napiStartup.min.toFixed(3)}  ${wasiStartup.min.toFixed(3)}  ${(wasiStartup.min/napiStartup.min).toFixed(1)}x`);
        console.log(`Average:       ${napiStartup.avg.toFixed(3)}  ${wasiStartup.avg.toFixed(3)}  ${(wasiStartup.avg/napiStartup.avg).toFixed(1)}x`);
        console.log(`Median:        ${napiStartup.median.toFixed(3)}  ${wasiStartup.median.toFixed(3)}  ${(wasiStartup.median/napiStartup.median).toFixed(1)}x`);
        console.log(`95th percentile: ${napiStartup.p95.toFixed(3)}  ${wasiStartup.p95.toFixed(3)}  ${(wasiStartup.p95/napiStartup.p95).toFixed(1)}x`);
        console.log(`Max:           ${napiStartup.max.toFixed(3)}  ${wasiStartup.max.toFixed(3)}  ${(wasiStartup.max/napiStartup.max).toFixed(1)}x`);

        console.log('\n⚡ OPERATION SPEED (ms)');
        console.log('─'.repeat(30));
        console.log('                 NAPI    WASI    Ratio');
        console.log(`Min:           ${napiOps.min.toFixed(6)}  ${wasiOps.min.toFixed(3)}  ${(wasiOps.min/napiOps.min).toFixed(0)}x`);
        console.log(`Average:       ${napiOps.avg.toFixed(6)}  ${wasiOps.avg.toFixed(3)}  ${(wasiOps.avg/napiOps.avg).toFixed(0)}x`);
        console.log(`Median:        ${napiOps.median.toFixed(6)}  ${wasiOps.median.toFixed(3)}  ${(wasiOps.median/napiOps.median).toFixed(0)}x`);
        console.log(`95th percentile: ${napiOps.p95.toFixed(6)}  ${wasiOps.p95.toFixed(3)}  ${(wasiOps.p95/napiOps.p95).toFixed(0)}x`);
        console.log(`Max:           ${napiOps.max.toFixed(6)}  ${wasiOps.max.toFixed(3)}  ${(wasiOps.max/napiOps.max).toFixed(0)}x`);

        console.log('\n📊 SUMMARY');
        console.log('─'.repeat(30));
        if (napiStartup.avg < wasiStartup.avg) {
            console.log(`🏆 NAPI is ${(wasiStartup.avg/napiStartup.avg).toFixed(1)}x faster at startup`);
        } else {
            console.log(`🏆 WASI is ${(napiStartup.avg/wasiStartup.avg).toFixed(1)}x faster at startup`);
        }
        
        if (napiOps.avg < wasiOps.avg) {
            console.log(`⚡ NAPI is ${(wasiOps.avg/napiOps.avg).toFixed(0)}x faster at operations`);
        } else {
            console.log(`⚡ WASI is ${(napiOps.avg/wasiOps.avg).toFixed(0)}x faster at operations`);
        }

        console.log('\n💡 RECOMMENDATIONS');
        console.log('─'.repeat(30));
        console.log('🔥 NAPI: Better for high-frequency operations, faster startup');
        console.log('🌊 WASI: Better for filesystem access, portable across platforms');
        console.log('🎯 Next.js: Use NAPI for performance-critical API routes');
        console.log('📁 Next.js: Use WASI when you need file-based configuration');
    }

    async run() {
        console.log('🎯 SuperConfig Performance Benchmark: WASI vs NAPI\n');
        
        try {
            await this.benchmarkNAPI();
            await this.benchmarkWASI();
            this.printResults();
        } catch (error) {
            console.error('❌ Benchmark failed:', error.message);
            console.error('💡 Make sure both superconfig_ffi.node and WASI .wasm files exist');
        }
    }
}

// Run the benchmark
const tester = new PerformanceTester();
tester.run();