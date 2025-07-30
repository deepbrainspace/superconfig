#!/usr/bin/env node
/**
 * Test script to demonstrate using SuperConfig WASM bindings from Node.js
 */

import { SuperConfig } from "./pkg/superconfig_ffi.js";

async function testWasm() {
  try {
    console.log("✅ Successfully imported SuperConfig from WASM module!");

    // Create a new SuperConfig instance
    const config = new SuperConfig();
    console.log("✅ Created SuperConfig instance");

    // Test getting verbosity
    const verbosity = config.get_verbosity();
    console.log(`✅ Default verbosity: ${verbosity}`);

    // Test with debug verbosity
    const debugConfig = config.with_debug_verbosity();
    const debugVerbosity = debugConfig.get_verbosity();
    console.log(`✅ Debug verbosity: ${debugVerbosity}`);

    // Test with trace verbosity
    const traceConfig = config.with_trace_verbosity();
    const traceVerbosity = traceConfig.get_verbosity();
    console.log(`✅ Trace verbosity: ${traceVerbosity}`);

    console.log("\n🎉 All WASM bindings working correctly!");

    // Clean up WASM memory
    config.free();
    debugConfig.free();
    traceConfig.free();
    console.log("✅ Memory cleaned up");
  } catch (error) {
    console.error("❌ Error:", error.message);
  }
}

testWasm();
