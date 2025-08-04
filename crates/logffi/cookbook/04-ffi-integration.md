# FFI Integration

LogFFI is designed to bridge Rust logging to other languages through FFI callbacks.

## Basic FFI Callback Setup

```rust
use logffi::{set_callback, error, warn, info};

// Define FFI callback signature
type LogCallback = extern "C" fn(level: *const i8, target: *const i8, message: *const i8);

// Set up FFI callback
pub fn setup_ffi_logging(callback: LogCallback) {
    set_callback(Box::new(move |level, target, message| {
        use std::ffi::CString;
        
        // Convert to C strings
        let c_level = CString::new(level).unwrap();
        let c_target = CString::new(target).unwrap();
        let c_message = CString::new(message).unwrap();
        
        // Call the FFI callback
        callback(
            c_level.as_ptr(),
            c_target.as_ptr(),
            c_message.as_ptr()
        );
    }));
}

// Now all LogFFI logs go through the callback
fn example() {
    error!("This error goes to FFI callback");
    warn!(target: "ffi::demo", "This warning too");
}
```

## Python Integration

```rust
// Rust side - src/lib.rs
use pyo3::prelude::*;
use logffi::{set_callback, info, error};

#[pymodule]
fn rust_module(_py: Python, m: &PyModule) -> PyResult<()> {
    // Initialize Python logging bridge
    #[pyfn(m)]
    fn init_logging(py: Python) -> PyResult<()> {
        set_callback(Box::new(move |level, target, message| {
            Python::with_gil(|py| {
                // Get Python logging module
                let logging = py.import("logging").unwrap();
                let logger = logging.call_method1("getLogger", (target,)).unwrap();
                
                // Map Rust levels to Python levels
                let py_level = match level {
                    "ERROR" => "error",
                    "WARN" => "warning",
                    "INFO" => "info",
                    "DEBUG" => "debug",
                    "TRACE" => "debug",  // Python doesn't have trace
                    _ => "info",
                };
                
                // Log to Python
                logger.call_method1(py_level, (message,)).unwrap();
            });
        }));
        Ok(())
    }
    
    #[pyfn(m)]
    fn process_data(data: &str) -> PyResult<String> {
        info!(target: "rust_module", "Processing data: {}", data);
        
        match validate_data(data) {
            Ok(result) => Ok(result),
            Err(e) => {
                error!(target: "rust_module", "Validation failed: {}", e);
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Invalid data: {}", e)
                ))
            }
        }
    }
    
    Ok(())
}
```

```python
# Python side
import logging
import rust_module

# Configure Python logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# Initialize Rust->Python logging bridge
rust_module.init_logging()

# Now Rust logs appear in Python
try:
    result = rust_module.process_data("test data")
    # Logs: "2024-01-15 10:30:00 - rust_module - INFO - Processing data: test data"
except ValueError as e:
    # Logs: "2024-01-15 10:30:01 - rust_module - ERROR - Validation failed: ..."
    print(f"Error: {e}")
```

## Node.js Integration

```rust
// Rust side using neon
use neon::prelude::*;
use logffi::{set_callback, debug, error};

fn init_logging(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Get the JavaScript callback function
    let callback = cx.argument::<JsFunction>(0)?;
    let callback = callback.root(&mut cx);
    
    // Set up the bridge
    set_callback(Box::new(move |level, target, message| {
        // Run in JavaScript context
        let callback = callback.clone();
        
        std::thread::spawn(move || {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                callback.into_inner(&mut cx)
                    .call_with(&cx)
                    .arg(cx.string(level))
                    .arg(cx.string(target))
                    .arg(cx.string(message))
                    .apply::<JsUndefined, _>(&mut cx)
                    .unwrap();
            });
        });
    }));
    
    Ok(cx.undefined())
}

fn process_request(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);
    
    debug!(target: "rust::api", "Processing request: {}", input);
    
    match handle_request(&input) {
        Ok(result) => Ok(cx.string(result)),
        Err(e) => {
            error!(target: "rust::api", "Request failed: {}", e);
            cx.throw_error(format!("Request failed: {}", e))
        }
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("initLogging", init_logging)?;
    cx.export_function("processRequest", process_request)?;
    Ok(())
}
```

```javascript
// JavaScript side
const rust = require('./native');

// Set up logging bridge to Winston
const winston = require('winston');

const logger = winston.createLogger({
    level: 'debug',
    format: winston.format.json(),
    transports: [
        new winston.transports.Console(),
        new winston.transports.File({ filename: 'app.log' })
    ]
});

// Initialize Rust logging
rust.initLogging((level, target, message) => {
    // Map Rust levels to Winston
    const winstonLevel = {
        'ERROR': 'error',
        'WARN': 'warn',
        'INFO': 'info',
        'DEBUG': 'debug',
        'TRACE': 'silly'
    }[level] || 'info';
    
    logger.log({
        level: winstonLevel,
        message: message,
        target: target,
        source: 'rust'
    });
});

// Use the Rust module
try {
    const result = rust.processRequest('test data');
    // Logs to Winston: {"level":"debug","message":"Processing request: test data","target":"rust::api","source":"rust"}
} catch (error) {
    // Logs to Winston: {"level":"error","message":"Request failed: ...","target":"rust::api","source":"rust"}
    console.error('Error:', error);
}
```

## C/C++ Integration

```rust
// Rust side - expose C API
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use logffi::{set_callback, info, error};

pub type CLogCallback = extern "C" fn(
    level: *const c_char,
    target: *const c_char,
    message: *const c_char,
);

#[no_mangle]
pub extern "C" fn logffi_init(callback: CLogCallback) {
    set_callback(Box::new(move |level, target, message| {
        let c_level = CString::new(level).unwrap();
        let c_target = CString::new(target).unwrap();
        let c_message = CString::new(message).unwrap();
        
        callback(
            c_level.as_ptr(),
            c_target.as_ptr(),
            c_message.as_ptr()
        );
    }));
}

#[no_mangle]
pub extern "C" fn process_data(data: *const c_char) -> i32 {
    let data = unsafe {
        assert!(!data.is_null());
        CStr::from_ptr(data)
    };
    
    let data_str = match data.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!(target: "ffi", "Invalid UTF-8 in input data");
            return -1;
        }
    };
    
    info!(target: "ffi", "Processing data: {}", data_str);
    
    // Process and return status code
    0
}
```

```c
// C side
#include <stdio.h>
#include <string.h>

// Function declarations
extern void logffi_init(void (*callback)(const char*, const char*, const char*));
extern int process_data(const char* data);

// C logging callback
void log_callback(const char* level, const char* target, const char* message) {
    // Map to syslog priorities
    int priority;
    if (strcmp(level, "ERROR") == 0) priority = LOG_ERR;
    else if (strcmp(level, "WARN") == 0) priority = LOG_WARNING;
    else if (strcmp(level, "INFO") == 0) priority = LOG_INFO;
    else if (strcmp(level, "DEBUG") == 0) priority = LOG_DEBUG;
    else priority = LOG_DEBUG;
    
    // Log to syslog
    syslog(priority, "[%s] %s", target, message);
    
    // Also print to console
    printf("[%s] %s: %s\n", level, target, message);
}

int main() {
    // Initialize syslog
    openlog("myapp", LOG_PID | LOG_CONS, LOG_USER);
    
    // Initialize Rust logging
    logffi_init(log_callback);
    
    // Use Rust functions
    int result = process_data("Hello from C!");
    // Outputs: [INFO] ffi: Processing data: Hello from C!
    
    closelog();
    return result;
}
```

## WebAssembly Integration

```rust
// Rust side - compile with wasm-pack
use wasm_bindgen::prelude::*;
use logffi::{set_callback, info, warn, error};

#[wasm_bindgen]
pub fn init_logging() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Bridge to browser console
    set_callback(Box::new(|level, target, message| {
        let console_method = match level {
            "ERROR" => "error",
            "WARN" => "warn",
            "INFO" => "info",
            "DEBUG" => "debug",
            "TRACE" => "debug",
            _ => "log",
        };
        
        // Call appropriate console method
        web_sys::console::log_1(&format!(
            "[{}] {}: {}",
            level, target, message
        ).into());
    }));
}

#[wasm_bindgen]
pub fn process_in_wasm(input: &str) -> Result<String, JsValue> {
    info!(target: "wasm", "Processing input: {}", input);
    
    match validate_input(input) {
        Ok(result) => {
            info!(target: "wasm", "Processing successful");
            Ok(result)
        }
        Err(e) => {
            error!(target: "wasm", "Processing failed: {}", e);
            Err(JsValue::from_str(&format!("Error: {}", e)))
        }
    }
}
```

```javascript
// JavaScript side
import init, { init_logging, process_in_wasm } from './pkg/my_wasm_module.js';

async function main() {
    // Initialize WASM module
    await init();
    
    // Set up logging bridge
    init_logging();
    
    // Now Rust logs appear in browser console
    try {
        const result = process_in_wasm("test input");
        // Console: [INFO] wasm: Processing input: test input
        // Console: [INFO] wasm: Processing successful
        console.log('Result:', result);
    } catch (error) {
        // Console: [ERROR] wasm: Processing failed: ...
        console.error('Failed:', error);
    }
}

main();
```

## Advanced FFI Patterns

### Dual-Mode Logging

```rust
use logffi::{set_callback, FORCE_NATIVE_BACKENDS};
use std::sync::atomic::Ordering;

// Enable both FFI callback AND native Rust logging
pub fn enable_dual_logging() {
    // Set FFI callback
    set_callback(Box::new(|level, target, message| {
        send_to_monitoring_system(level, target, message);
    }));
    
    // Also enable native backends
    FORCE_NATIVE_BACKENDS.store(true, Ordering::Relaxed);
    
    // Now logs go to both FFI callback AND env_logger/tracing/slog
}
```

### Conditional FFI Logging

```rust
use logffi::set_callback;

pub fn setup_conditional_ffi() {
    set_callback(Box::new(|level, target, message| {
        // Only forward certain logs to FFI
        if target.starts_with("critical::") || level == "ERROR" {
            forward_to_ffi(level, target, message);
        }
        
        // High-volume debug logs stay in Rust
        if level == "DEBUG" || level == "TRACE" {
            // Don't forward to FFI to avoid overhead
            return;
        }
    }));
}
```
