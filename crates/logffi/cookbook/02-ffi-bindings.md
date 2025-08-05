# LogFFI Cookbook: FFI Bindings for Python and Node.js

This guide shows how to create lightweight FFI bindings for Python and Node.js that bridge Rust logs to native logging systems.

## Table of Contents

- [Python Bindings with PyO3](#python-bindings-with-pyo3)
- [Node.js Bindings with NAPI](#nodejs-bindings-with-napi)
- [WASM Bindings for Browser](#wasm-bindings-for-browser)
- [C/C++ Bindings](#cc-bindings)

## Python Bindings with PyO3

### Rust Library Setup

Create a new crate for Python bindings:

```toml
# Cargo.toml
[package]
name = "logffi-python"
version = "0.1.0"
edition = "2021"

[lib]
name = "logffi_python"
crate-type = ["cdylib"]

[dependencies]
logffi = { version = "0.2", default-features = false, features = ["callback"] }
pyo3 = { version = "0.20", features = ["extension-module"] }
```

```rust
// src/lib.rs
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Mutex;

static PYTHON_HANDLER: Mutex<Option<PyObject>> = Mutex::new(None);

/// Configure Rust logging to bridge to Python's logging system
#[pyfunction]
fn setup_rust_logging(py: Python, handler: PyObject) -> PyResult<()> {
    // Store the Python handler
    *PYTHON_HANDLER.lock().unwrap() = Some(handler.clone());
    
    // Set up LogFFI callback
    logffi::set_callback(Box::new(move |level, target, message| {
        Python::with_gil(|py| {
            if let Some(handler) = PYTHON_HANDLER.lock().unwrap().as_ref() {
                let kwargs = PyDict::new(py);
                kwargs.set_item("level", level).ok();
                kwargs.set_item("target", target).ok();
                kwargs.set_item("message", message).ok();
                
                let _ = handler.call_method(py, "handle_rust_log", (), Some(kwargs));
            }
        });
    }));
    
    Ok(())
}

/// Get current Rust log level
#[pyfunction]
fn get_rust_log_level() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
}

/// Set Rust log level
#[pyfunction]
fn set_rust_log_level(level: &str) -> PyResult<()> {
    std::env::set_var("RUST_LOG", level);
    Ok(())
}

#[pymodule]
fn logffi_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup_rust_logging, m)?)?;
    m.add_function(wrap_pyfunction!(get_rust_log_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_rust_log_level, m)?)?;
    Ok(())
}
```

### Python Usage

```python
# logffi_bridge.py
import logging
import logffi_python
from typing import Optional, Dict, Any

class RustLogHandler:
    """Bridge Rust logs to Python's logging system"""
    
    def __init__(self, logger_name_prefix: str = "rust"):
        self.logger_name_prefix = logger_name_prefix
        self.level_map = {
            'error': logging.ERROR,
            'warn': logging.WARNING,
            'info': logging.INFO,
            'debug': logging.DEBUG,
            'trace': logging.DEBUG - 5,  # Python doesn't have TRACE
        }
        
        # Create a custom TRACE level
        if not hasattr(logging, 'TRACE'):
            logging.TRACE = logging.DEBUG - 5
            logging.addLevelName(logging.TRACE, 'TRACE')
    
    def handle_rust_log(self, level: str, target: str, message: str):
        """Handle a log message from Rust"""
        # Get or create logger for this target
        logger_name = f"{self.logger_name_prefix}.{target}"
        logger = logging.getLogger(logger_name)
        
        # Map Rust level to Python level
        py_level = self.level_map.get(level.lower(), logging.INFO)
        
        # Log the message
        logger.log(py_level, message, extra={
            'rust_target': target,
            'rust_level': level
        })

def setup_rust_logging(
    logger_prefix: str = "rust",
    rust_log_level: str = "info"
) -> RustLogHandler:
    """
    Set up Rust logging bridge to Python.
    
    Args:
        logger_prefix: Prefix for logger names (default: "rust")
        rust_log_level: Rust log level (error, warn, info, debug, trace)
    
    Returns:
        RustLogHandler instance
    """
    handler = RustLogHandler(logger_prefix)
    logffi_python.setup_rust_logging(handler)
    logffi_python.set_rust_log_level(rust_log_level)
    return handler

# Example usage
if __name__ == "__main__":
    # Configure Python logging
    logging.basicConfig(
        level=logging.DEBUG,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Set up Rust logging bridge
    rust_handler = setup_rust_logging(rust_log_level="debug")
    
    # Now when you use Rust libraries that use LogFFI
    import my_rust_library  # Your Rust lib compiled with PyO3
    
    # Rust logs will appear in Python logging!
    my_rust_library.process_data()  # Any logffi logs show up in Python
```

### Advanced Python Integration

```python
# advanced_integration.py
import logging
import logging.handlers
import json
from datetime import datetime
import logffi_python

class StructuredRustLogHandler:
    """Enhanced handler with structured logging support"""
    
    def __init__(self):
        self.setup_handlers()
    
    def setup_handlers(self):
        """Set up different handlers for different Rust components"""
        # File handler for all Rust logs
        rust_file_handler = logging.handlers.RotatingFileHandler(
            'rust_logs.log',
            maxBytes=10485760,  # 10MB
            backupCount=5
        )
        rust_file_handler.setLevel(logging.DEBUG)
        
        # JSON formatter for structured logs
        json_formatter = JsonFormatter()
        rust_file_handler.setFormatter(json_formatter)
        
        # Add handler to rust logger
        rust_logger = logging.getLogger('rust')
        rust_logger.addHandler(rust_file_handler)
        
        # Separate handler for errors
        error_handler = logging.handlers.SMTPHandler(
            mailhost='localhost',
            fromaddr='app@example.com',
            toaddrs=['admin@example.com'],
            subject='Rust Error in Production'
        )
        error_handler.setLevel(logging.ERROR)
        rust_logger.addHandler(error_handler)
    
    def handle_rust_log(self, level: str, target: str, message: str):
        """Enhanced log handling with metrics"""
        logger = logging.getLogger(f'rust.{target}')
        
        # Parse structured data if present
        extra = {
            'rust_target': target,
            'rust_level': level,
            'timestamp': datetime.utcnow().isoformat()
        }
        
        # Try to parse JSON from message
        try:
            if message.startswith('{'):
                data = json.loads(message)
                extra.update(data)
                message = data.get('msg', message)
        except:
            pass
        
        # Log with extra context
        level_map = {
            'error': logging.ERROR,
            'warn': logging.WARNING,
            'info': logging.INFO,
            'debug': logging.DEBUG,
            'trace': 5,
        }
        
        logger.log(
            level_map.get(level.lower(), logging.INFO),
            message,
            extra=extra
        )
        
        # Update metrics
        if level == 'error':
            increment_error_counter(target)

class JsonFormatter(logging.Formatter):
    """JSON formatter for structured logging"""
    
    def format(self, record):
        log_obj = {
            'timestamp': datetime.utcnow().isoformat(),
            'level': record.levelname,
            'logger': record.name,
            'message': record.getMessage(),
        }
        
        # Add any extra fields
        for key, value in record.__dict__.items():
            if key not in ['name', 'msg', 'args', 'created', 'filename', 
                          'funcName', 'levelname', 'levelno', 'lineno', 
                          'module', 'msecs', 'message', 'pathname', 'process',
                          'processName', 'relativeCreated', 'thread', 'threadName']:
                log_obj[key] = value
        
        return json.dumps(log_obj)

def increment_error_counter(target: str):
    """Track error metrics (integrate with Prometheus, DataDog, etc.)"""
    # Example: prometheus_client.Counter
    pass
```

## Node.js Bindings with NAPI

### Rust Library Setup

```toml
# Cargo.toml
[package]
name = "logffi-node"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
logffi = { version = "0.2", default-features = false, features = ["callback"] }
napi = { version = "2", features = ["napi4"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
```

```rust
// src/lib.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

static NODE_CALLBACK: Mutex<Option<ThreadsafeFunction<LogMessage>>> = Mutex::new(None);

#[napi(object)]
pub struct LogMessage {
    pub level: String,
    pub target: String,
    pub message: String,
}

#[napi]
pub fn setup_rust_logging(callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<LogMessage> = callback
        .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    
    *NODE_CALLBACK.lock().unwrap() = Some(tsfn.clone());
    
    logffi::set_callback(Box::new(move |level, target, message| {
        if let Some(tsfn) = NODE_CALLBACK.lock().unwrap().as_ref() {
            let log_msg = LogMessage {
                level: level.to_string(),
                target: target.to_string(),
                message: message.to_string(),
            };
            
            tsfn.call(log_msg, ThreadsafeFunctionCallMode::NonBlocking);
        }
    }));
    
    Ok(())
}

#[napi]
pub fn set_rust_log_level(level: String) {
    std::env::set_var("RUST_LOG", level);
}

#[napi]
pub fn get_rust_log_level() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
}
```

### Node.js Usage

```javascript
// index.js
const { setupRustLogging, setRustLogLevel } = require('./logffi-node');
const winston = require('winston');
const pino = require('pino');

// Option 1: Bridge to Winston
function bridgeToWinston() {
    const logger = winston.createLogger({
        level: 'debug',
        format: winston.format.combine(
            winston.format.timestamp(),
            winston.format.json()
        ),
        transports: [
            new winston.transports.Console(),
            new winston.transports.File({ filename: 'rust-logs.log' })
        ]
    });

    setupRustLogging((logMessage) => {
        const { level, target, message } = logMessage;
        
        // Map Rust levels to Winston levels
        const levelMap = {
            'error': 'error',
            'warn': 'warn',
            'info': 'info',
            'debug': 'debug',
            'trace': 'silly'
        };
        
        logger.log({
            level: levelMap[level] || 'info',
            message: message,
            target: target,
            source: 'rust'
        });
    });
}

// Option 2: Bridge to Pino (high-performance)
function bridgeToPino() {
    const logger = pino({
        level: 'trace',
        transport: {
            target: 'pino-pretty',
            options: {
                colorize: true
            }
        }
    });

    setupRustLogging((logMessage) => {
        const { level, target, message } = logMessage;
        
        const child = logger.child({ target, source: 'rust' });
        
        switch(level) {
            case 'error': child.error(message); break;
            case 'warn': child.warn(message); break;
            case 'info': child.info(message); break;
            case 'debug': child.debug(message); break;
            case 'trace': child.trace(message); break;
        }
    });
}

// Option 3: Custom handler with filtering
function customHandler() {
    const logHandlers = {
        'database': (level, message) => {
            // Special handling for database logs
            console.log(`[DB ${level}] ${message}`);
        },
        'api': (level, message) => {
            // API logs go to different destination
            if (level === 'error') {
                notifyOpsTeam(message);
            }
        }
    };

    setupRustLogging((logMessage) => {
        const { level, target, message } = logMessage;
        
        // Route based on target
        const handler = logHandlers[target] || console.log;
        handler(level, message);
    });
}

// Advanced: Structured logging with context
class RustLogBridge {
    constructor(options = {}) {
        this.logger = options.logger || console;
        this.filters = options.filters || [];
        this.transformers = options.transformers || [];
        
        this.setupBridge();
    }
    
    setupBridge() {
        setupRustLogging((logMessage) => {
            // Apply filters
            if (!this.shouldLog(logMessage)) {
                return;
            }
            
            // Transform message
            const transformed = this.transform(logMessage);
            
            // Log it
            this.log(transformed);
        });
    }
    
    shouldLog(logMessage) {
        return this.filters.every(filter => filter(logMessage));
    }
    
    transform(logMessage) {
        return this.transformers.reduce(
            (msg, transformer) => transformer(msg),
            logMessage
        );
    }
    
    log(logMessage) {
        const { level, target, message } = logMessage;
        
        if (typeof this.logger[level] === 'function') {
            this.logger[level]({ target, msg: message });
        } else {
            this.logger.log(level, { target, msg: message });
        }
    }
}

// Usage
const bridge = new RustLogBridge({
    logger: winston.createLogger({ /* ... */ }),
    filters: [
        // Only log errors and warnings in production
        (msg) => process.env.NODE_ENV !== 'production' || 
                 ['error', 'warn'].includes(msg.level),
        // Filter out verbose targets
        (msg) => !msg.target.startsWith('hyper::')
    ],
    transformers: [
        // Add timestamp
        (msg) => ({ ...msg, timestamp: new Date().toISOString() }),
        // Parse structured data
        (msg) => {
            try {
                if (msg.message.startsWith('{')) {
                    const data = JSON.parse(msg.message);
                    return { ...msg, ...data };
                }
            } catch {}
            return msg;
        }
    ]
});

// Set Rust log level
setRustLogLevel('debug');

// Now use your Rust library
const myRustLib = require('./my-rust-lib');
myRustLib.processData(); // Logs appear in Node.js!
```

## WASM Bindings for Browser

```rust
// src/wasm.rs
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn setup_rust_logging_wasm() {
    logffi::set_callback(Box::new(|level, target, message| {
        let msg = format!("[{}] {}: {}", level, target, message);
        
        match level {
            "error" => console::error_1(&msg.into()),
            "warn" => console::warn_1(&msg.into()),
            "info" => console::info_1(&msg.into()),
            "debug" => console::debug_1(&msg.into()),
            "trace" => console::trace_1(&msg.into()),
            _ => console::log_1(&msg.into()),
        }
    }));
}
```

```javascript
// Browser usage
import init, { setup_rust_logging_wasm } from './my_rust_lib_wasm.js';

async function main() {
    await init();
    
    // Set up logging bridge
    setup_rust_logging_wasm();
    
    // Now Rust logs appear in browser console!
    // With proper formatting and levels
}
```

## C/C++ Bindings

```rust
// src/ffi.rs
use std::ffi::{c_char, CStr};
use std::sync::Mutex;

type LogCallback = extern "C" fn(level: *const c_char, target: *const c_char, message: *const c_char);

static C_CALLBACK: Mutex<Option<LogCallback>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn logffi_set_callback(callback: LogCallback) {
    *C_CALLBACK.lock().unwrap() = Some(callback);
    
    logffi::set_callback(Box::new(move |level, target, message| {
        if let Some(cb) = *C_CALLBACK.lock().unwrap() {
            use std::ffi::CString;
            
            let c_level = CString::new(level).unwrap();
            let c_target = CString::new(target).unwrap();
            let c_message = CString::new(message).unwrap();
            
            cb(c_level.as_ptr(), c_target.as_ptr(), c_message.as_ptr());
        }
    }));
}
```

```c
// example.c
#include <stdio.h>

void handle_rust_log(const char* level, const char* target, const char* message) {
    printf("[%s] %s: %s\n", level, target, message);
    
    // Or integrate with syslog, custom logging, etc.
    if (strcmp(level, "error") == 0) {
        syslog(LOG_ERR, "Rust Error [%s]: %s", target, message);
    }
}

int main() {
    // Set up the callback
    logffi_set_callback(handle_rust_log);
    
    // Use Rust library
    rust_library_function();
    
    return 0;
}
```

## Key Takeaways

1. **Keep bindings thin**: Just bridge logs to native logging systems
2. **Don't recreate logging APIs**: Use each language's native tools
3. **Focus on integration**: Make Rust libraries feel native in each language
4. **Performance is not the goal**: Convenience and compatibility are
5. **Structure preservation**: Pass through structured data when possible
