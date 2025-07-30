#!/usr/bin/env node
/**
 * Test script to demonstrate using SuperConfig from Node.js
 */

try {
  const { SuperConfig } = require("./superconfig_ffi.node");

  console.log("✅ Successfully imported SuperConfig from Node.js module!");

  // Create a new SuperConfig instance
  const config = new SuperConfig();
  console.log("✅ Created SuperConfig instance:", config);

  // Test getting verbosity
  const verbosity = config.getVerbosity();
  console.log("✅ Default verbosity:", verbosity);

  // Test with debug verbosity
  const debugConfig = config.withDebugVerbosity();
  const debugVerbosity = debugConfig.getVerbosity();
  console.log("✅ Debug verbosity:", debugVerbosity);

  // Test with trace verbosity
  const traceConfig = config.withTraceVerbosity();
  const traceVerbosity = traceConfig.getVerbosity();
  console.log("✅ Trace verbosity:", traceVerbosity);

  console.log("\n🎉 All Node.js bindings working correctly!");
} catch (error) {
  console.error("❌ Error:", error.message);
  console.error(
    'Make sure to build with: cargo build --features "nodejs" --release',
  );
}
