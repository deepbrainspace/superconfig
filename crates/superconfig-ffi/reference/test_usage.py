#!/usr/bin/env python3
"""Test script to demonstrate using SuperConfig from Python"""

try:
    import superconfig_ffi
    
    print("✅ Successfully imported superconfig_ffi module!")
    
    # Create a new SuperConfig instance
    config = superconfig_ffi.SuperConfig()
    print(f"✅ Created SuperConfig instance: {config}")
    
    # Test getting verbosity
    verbosity = config.get_verbosity()
    print(f"✅ Default verbosity: {verbosity}")
    
    # Test with debug verbosity
    debug_config = config.with_debug_verbosity()
    debug_verbosity = debug_config.get_verbosity()
    print(f"✅ Debug verbosity: {debug_verbosity}")
    
    # Test with trace verbosity
    trace_config = config.with_trace_verbosity()
    trace_verbosity = trace_config.get_verbosity()
    print(f"✅ Trace verbosity: {trace_verbosity}")
    
    print("\n🎉 All Python bindings working correctly!")
    
except ImportError as e:
    print(f"❌ Failed to import superconfig_ffi: {e}")
except Exception as e:
    print(f"❌ Error using SuperConfig: {e}")