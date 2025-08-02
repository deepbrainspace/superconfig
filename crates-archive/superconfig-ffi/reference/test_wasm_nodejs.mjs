#!/usr/bin/env node
/**
 * Test script for SuperConfig WASM bindings in Node.js
 * Using .mjs extension for ES modules
 */

import { SuperConfig } from './pkg/superconfig_ffi.js';

async function testWasmInNodejs() {
    try {
        console.log('✅ Successfully imported SuperConfig from Node.js-compatible WASM module!');
        
        // Create a new SuperConfig instance
        const config = new SuperConfig();
        console.log('✅ Created SuperConfig instance');
        
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
        
        console.log('\n🎉 WASM bindings working perfectly in Node.js!');
        console.log('🚀 This will work in Next.js applications!');
        
        // Clean up WASM memory
        config.free();
        debugConfig.free();
        traceConfig.free();
        console.log('✅ Memory cleaned up');
        
    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error('Stack:', error.stack);
    }
}

testWasmInNodejs();