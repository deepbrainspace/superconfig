# FFI Integration and Callback Logging

This guide covers LogFusion's Foreign Function Interface (FFI) capabilities and callback-based logging. LogFusion provides seamless integration with C libraries, callback functions, and cross-language logging scenarios while tracing handles the underlying observability infrastructure.

**What LogFusion provides:** FFI-safe logging macros, callback registration system, C-compatible interfaces, and auto-initialization across language boundaries.

**What tracing provides:** The underlying logging infrastructure, structured logging support, and subscriber management that works across FFI boundaries.

## Table of Contents

- [FFI Logging Basics](#ffi-logging-basics)
- [Callback Registration](#callback-registration)
- [C Library Integration](#c-library-integration)
- [Cross-Language Tracing](#cross-language-tracing)
- [Memory Safety and FFI](#memory-safety-and-ffi)
- [Performance Considerations](#performance-considerations)
- [Error Handling Across FFI](#error-handling-across-ffi)
- [Advanced FFI Patterns](#advanced-ffi-patterns)

## FFI Logging Basics

### Basic FFI Logging Setup

```rust
use logfusion::{info, warn, error, ffi_info, ffi_warn, ffi_error};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// FFI-safe logging functions that can be called from C
#[no_mangle]
pub extern "C" fn log_info(message: *const c_char) {
    if message.is_null() {
        return;
    }
    
    unsafe {
        if let Ok(c_str) = CStr::from_ptr(message).to_str() {
            info!(target: "ffi", message = c_str, "FFI info message");
        }
    }
}

#[no_mangle]
pub extern "C" fn log_warning(message: *const c_char) {
    if message.is_null() {
        return;
    }
    
    unsafe {
        if let Ok(c_str) = CStr::from_ptr(message).to_str() {
            warn!(target: "ffi", message = c_str, "FFI warning message");
        }
    }
}

#[no_mangle]
pub extern "C" fn log_error(message: *const c_char, error_code: c_int) {
    if message.is_null() {
        return;
    }
    
    unsafe {
        if let Ok(c_str) = CStr::from_ptr(message).to_str() {
            error!(
                target: "ffi", 
                message = c_str, 
                error_code = error_code,
                "FFI error message"
            );
        }
    }
}

// Initialize LogFusion for FFI usage
#[no_mangle]
pub extern "C" fn logfusion_init() -> c_int {
    // LogFusion auto-initialization will happen on first log call
    // This function can be used for explicit initialization
    info!("LogFusion initialized for FFI usage");
    0 // Success
}

// Shutdown function for clean FFI teardown
#[no_mangle]
pub extern "C" fn logfusion_shutdown() {
    info!("LogFusion shutting down");
    // Flush any pending logs
}
```

### FFI-Safe Structured Logging

```rust
use logfusion::{info, error};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_double};

// Structured logging with multiple parameters
#[no_mangle]
pub extern "C" fn log_user_action(
    user_id: c_int,
    action: *const c_char,
    timestamp: c_double,
    success: c_int,
) {
    if action.is_null() {
        return;
    }
    
    unsafe {
        if let Ok(action_str) = CStr::from_ptr(action).to_str() {
            info!(
                target: "ffi::user_actions",
                user_id = user_id,
                action = action_str,
                timestamp = timestamp,
                success = success != 0,
                "User action logged from FFI"
            );
        }
    }
}

// Database operation logging via FFI
#[no_mangle]
pub extern "C" fn log_database_operation(
    operation: *const c_char,
    table_name: *const c_char,
    rows_affected: c_int,
    duration_ms: c_double,
    error_code: c_int,
) {
    if operation.is_null() || table_name.is_null() {
        return;
    }
    
    unsafe {
        let operation_str = CStr::from_ptr(operation).to_str().unwrap_or("unknown");
        let table_str = CStr::from_ptr(table_name).to_str().unwrap_or("unknown");
        
        if error_code == 0 {
            info!(
                target: "ffi::database",
                operation = operation_str,
                table = table_str,
                rows_affected = rows_affected,
                duration_ms = duration_ms,
                "Database operation completed successfully"
            );
        } else {
            error!(
                target: "ffi::database",
                operation = operation_str,
                table = table_str,
                error_code = error_code,
                duration_ms = duration_ms,
                "Database operation failed"
            );
        }
    }
}

// Network request logging
#[no_mangle]
pub extern "C" fn log_http_request(
    method: *const c_char,
    url: *const c_char,
    status_code: c_int,
    response_time_ms: c_double,
    content_length: c_int,
) {
    if method.is_null() || url.is_null() {
        return;
    }
    
    unsafe {
        let method_str = CStr::from_ptr(method).to_str().unwrap_or("unknown");
        let url_str = CStr::from_ptr(url).to_str().unwrap_or("unknown");
        
        info!(
            target: "ffi::http",
            method = method_str,
            url = url_str,
            status_code = status_code,
            response_time_ms = response_time_ms,
            content_length = content_length,
            "HTTP request processed"
        );
    }
}
```

## Callback Registration

### Callback-Based Logging System

```rust
use logfusion::{info, warn, error};
use std::ffi::{CStr, c_void};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;

// Callback function type for custom log handlers
pub type LogCallback = extern "C" fn(
    level: c_int,
    target: *const c_char,
    message: *const c_char,
    context: *mut c_void,
);

// Global callback registry
static INIT: Once = Once::new();
static mut CALLBACKS: Option<Arc<Mutex<HashMap<String, (LogCallback, *mut c_void)>>>> = None;

fn get_callbacks() -> &'static Arc<Mutex<HashMap<String, (LogCallback, *mut c_void)>>> {
    unsafe {
        INIT.call_once(|| {
            CALLBACKS = Some(Arc::new(Mutex::new(HashMap::new())));
        });
        CALLBACKS.as_ref().unwrap()
    }
}

// Register a callback for specific log targets
#[no_mangle]
pub extern "C" fn register_log_callback(
    target: *const c_char,
    callback: LogCallback,
    context: *mut c_void,
) -> c_int {
    if target.is_null() {
        return -1;
    }
    
    unsafe {
        if let Ok(target_str) = CStr::from_ptr(target).to_str() {
            let callbacks = get_callbacks();
            if let Ok(mut map) = callbacks.lock() {
                map.insert(target_str.to_string(), (callback, context));
                info!(
                    target = target_str,
                    "Log callback registered"
                );
                return 0; // Success
            }
        }
    }
    
    -1 // Error
}

// Unregister a callback
#[no_mangle]
pub extern "C" fn unregister_log_callback(target: *const c_char) -> c_int {
    if target.is_null() {
        return -1;
    }
    
    unsafe {
        if let Ok(target_str) = CStr::from_ptr(target).to_str() {
            let callbacks = get_callbacks();
            if let Ok(mut map) = callbacks.lock() {
                if map.remove(target_str).is_some() {
                    info!(
                        target = target_str,
                        "Log callback unregistered"
                    );
                    return 0;
                }
            }
        }
    }
    
    -1 // Error
}

// Trigger callbacks for specific events
fn trigger_callbacks(level: i32, target: &str, message: &str) {
    let callbacks = get_callbacks();
    if let Ok(map) = callbacks.lock() {
        if let Some((callback, context)) = map.get(target) {
            unsafe {
                let target_cstr = std::ffi::CString::new(target).unwrap();
                let message_cstr = std::ffi::CString::new(message).unwrap();
                
                callback(
                    level,
                    target_cstr.as_ptr(),
                    message_cstr.as_ptr(),
                    *context,
                );
            }
        }
    }
}

// Enhanced logging functions that trigger callbacks
#[no_mangle]
pub extern "C" fn log_with_callback(
    level: c_int,
    target: *const c_char,
    message: *const c_char,
) {
    if target.is_null() || message.is_null() {
        return;
    }
    
    unsafe {
        if let (Ok(target_str), Ok(message_str)) = (
            CStr::from_ptr(target).to_str(),
            CStr::from_ptr(message).to_str()
        ) {
            // Log normally through LogFusion
            match level {
                1 => error!(target: target_str, message = message_str, "Callback error"),
                2 => warn!(target: target_str, message = message_str, "Callback warning"),
                3 => info!(target: target_str, message = message_str, "Callback info"),
                _ => info!(target: target_str, message = message_str, "Callback message"),
            }
            
            // Trigger registered callbacks
            trigger_callbacks(level, target_str, message_str);
        }
    }
}
```

## C Library Integration

### Integrating with Existing C Libraries

```rust
use logfusion::{info, warn, error, instrument};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// Example: Integrating with a hypothetical C graphics library
extern "C" {
    fn graphics_init() -> c_int;
    fn graphics_render_frame(frame_data: *mut c_void) -> c_int;
    fn graphics_cleanup();
}

// Wrapper functions with logging
#[instrument(name = "graphics_initialization")]
pub fn initialize_graphics() -> Result<(), GraphicsError> {
    info!("Initializing graphics subsystem");
    
    unsafe {
        let result = graphics_init();
        if result == 0 {
            info!("Graphics initialization successful");
            Ok(())
        } else {
            error!(error_code = result, "Graphics initialization failed");
            Err(GraphicsError::InitializationFailed(result))
        }
    }
}

#[instrument(skip(frame_data), fields(frame_size = frame_size))]
pub fn render_frame(frame_data: *mut c_void, frame_size: usize) -> Result<(), GraphicsError> {
    info!("Rendering graphics frame");
    
    unsafe {
        let result = graphics_render_frame(frame_data);
        if result == 0 {
            info!(frame_rendered = true, "Frame rendered successfully");
            Ok(())
        } else {
            error!(
                error_code = result,
                frame_size = frame_size,
                "Frame rendering failed"
            );
            Err(GraphicsError::RenderFailed(result))
        }
    }
}

#[instrument]
pub fn cleanup_graphics() {
    info!("Cleaning up graphics subsystem");
    unsafe {
        graphics_cleanup();
    }
    info!("Graphics cleanup completed");
}

#[derive(Debug)]
pub enum GraphicsError {
    InitializationFailed(c_int),
    RenderFailed(c_int),
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsError::InitializationFailed(code) => {
                write!(f, "Graphics initialization failed with code: {}", code)
            }
            GraphicsError::RenderFailed(code) => {
                write!(f, "Graphics rendering failed with code: {}", code)
            }
        }
    }
}

impl std::error::Error for GraphicsError {}
```

### Database Driver Integration

```rust
use logfusion::{info, error, instrument};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::time::Instant;

// Mock C database API
extern "C" {
    fn db_connect(connection_string: *const c_char) -> *mut c_void;
    fn db_execute(connection: *mut c_void, query: *const c_char) -> c_int;
    fn db_close(connection: *mut c_void);
}

pub struct DatabaseConnection {
    handle: *mut c_void,
}

impl DatabaseConnection {
    #[instrument(skip(connection_string))]
    pub fn connect(connection_string: &str) -> Result<Self, DatabaseError> {
        info!("Connecting to database");
        
        let cstring = CString::new(connection_string)
            .map_err(|_| DatabaseError::InvalidConnectionString)?;
        
        unsafe {
            let handle = db_connect(cstring.as_ptr());
            if handle.is_null() {
                error!("Database connection failed");
                Err(DatabaseError::ConnectionFailed)
            } else {
                info!("Database connection established");
                Ok(DatabaseConnection { handle })
            }
        }
    }
    
    #[instrument(skip(self), fields(query_preview = %&query[..query.len().min(100)]))]
    pub fn execute(&self, query: &str) -> Result<i32, DatabaseError> {
        let start_time = Instant::now();
        
        info!("Executing database query");
        
        let cquery = CString::new(query)
            .map_err(|_| DatabaseError::InvalidQuery)?;
        
        unsafe {
            let result = db_execute(self.handle, cquery.as_ptr());
            let duration = start_time.elapsed();
            
            if result >= 0 {
                info!(
                    rows_affected = result,
                    duration_ms = duration.as_millis() as u64,
                    "Query executed successfully"
                );
                Ok(result)
            } else {
                error!(
                    error_code = result,
                    duration_ms = duration.as_millis() as u64,
                    "Query execution failed"
                );
                Err(DatabaseError::QueryFailed(result))
            }
        }
    }
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        info!("Closing database connection");
        unsafe {
            db_close(self.handle);
        }
        info!("Database connection closed");
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    InvalidConnectionString,
    ConnectionFailed,
    InvalidQuery,
    QueryFailed(c_int),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::InvalidConnectionString => write!(f, "Invalid connection string"),
            DatabaseError::ConnectionFailed => write!(f, "Database connection failed"),
            DatabaseError::InvalidQuery => write!(f, "Invalid query"),
            DatabaseError::QueryFailed(code) => write!(f, "Query failed with code: {}", code),
        }
    }
}

impl std::error::Error for DatabaseError {}
```

## Cross-Language Tracing

### Python Integration Example

```rust
use logfusion::{info, warn, error};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_double};

// FFI functions for Python integration
#[no_mangle]
pub extern "C" fn python_log_info(message: *const c_char, module: *const c_char) {
    if message.is_null() || module.is_null() {
        return;
    }
    
    unsafe {
        let msg = CStr::from_ptr(message).to_str().unwrap_or("invalid");
        let mod_name = CStr::from_ptr(module).to_str().unwrap_or("python");
        
        info!(
            target: "python",
            module = mod_name,
            message = msg,
            "Python log message"
        );
    }
}

#[no_mangle]
pub extern "C" fn python_log_function_call(
    function_name: *const c_char,
    module_name: *const c_char,
    duration_seconds: c_double,
    args_count: c_int,
) {
    if function_name.is_null() || module_name.is_null() {
        return;
    }
    
    unsafe {
        let func = CStr::from_ptr(function_name).to_str().unwrap_or("unknown");
        let module = CStr::from_ptr(module_name).to_str().unwrap_or("unknown");
        
        info!(
            target: "python::function_calls",
            function = func,
            module = module,
            duration_seconds = duration_seconds,
            args_count = args_count,
            "Python function call traced"
        );
    }
}

#[no_mangle]
pub extern "C" fn python_log_exception(
    exception_type: *const c_char,
    exception_message: *const c_char,
    traceback: *const c_char,
    module_name: *const c_char,
) {
    if exception_type.is_null() {
        return;
    }
    
    unsafe {
        let exc_type = CStr::from_ptr(exception_type).to_str().unwrap_or("Exception");
        let exc_msg = if exception_message.is_null() {
            "No message"
        } else {
            CStr::from_ptr(exception_message).to_str().unwrap_or("invalid message")
        };
        let tb = if traceback.is_null() {
            "No traceback"
        } else {
            CStr::from_ptr(traceback).to_str().unwrap_or("invalid traceback")
        };
        let module = if module_name.is_null() {
            "unknown"
        } else {
            CStr::from_ptr(module_name).to_str().unwrap_or("unknown")
        };
        
        error!(
            target: "python::exceptions",
            exception_type = exc_type,
            exception_message = exc_msg,
            traceback = tb,
            module = module,
            "Python exception occurred"
        );
    }
}
```

### JavaScript/Node.js Integration

```rust
use logfusion::{info, warn, error};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_double};

// FFI functions for Node.js integration
#[no_mangle]
pub extern "C" fn nodejs_log_request(
    method: *const c_char,
    url: *const c_char,
    status_code: c_int,
    response_time_ms: c_double,
    user_agent: *const c_char,
) {
    if method.is_null() || url.is_null() {
        return;
    }
    
    unsafe {
        let method_str = CStr::from_ptr(method).to_str().unwrap_or("unknown");
        let url_str = CStr::from_ptr(url).to_str().unwrap_or("unknown");
        let ua_str = if user_agent.is_null() {
            "unknown"
        } else {
            CStr::from_ptr(user_agent).to_str().unwrap_or("unknown")
        };
        
        info!(
            target: "nodejs::requests",
            method = method_str,
            url = url_str,
            status_code = status_code,
            response_time_ms = response_time_ms,
            user_agent = ua_str,
            "Node.js HTTP request processed"
        );
    }
}

#[no_mangle]
pub extern "C" fn nodejs_log_error(
    error_name: *const c_char,
    error_message: *const c_char,
    stack_trace: *const c_char,
    file_name: *const c_char,
    line_number: c_int,
) {
    if error_name.is_null() {
        return;
    }
    
    unsafe {
        let name = CStr::from_ptr(error_name).to_str().unwrap_or("Error");
        let message = if error_message.is_null() {
            "No message"
        } else {
            CStr::from_ptr(error_message).to_str().unwrap_or("invalid message")
        };
        let stack = if stack_trace.is_null() {
            "No stack trace"
        } else {
            CStr::from_ptr(stack_trace).to_str().unwrap_or("invalid stack trace")
        };
        let file = if file_name.is_null() {
            "unknown"
        } else {
            CStr::from_ptr(file_name).to_str().unwrap_or("unknown")
        };
        
        error!(
            target: "nodejs::errors",
            error_name = name,
            error_message = message,
            stack_trace = stack,
            file_name = file,
            line_number = line_number,
            "Node.js error occurred"
        );
    }
}

#[no_mangle]
pub extern "C" fn nodejs_log_performance(
    operation_name: *const c_char,
    duration_ms: c_double,
    memory_used_mb: c_double,
    cpu_time_ms: c_double,
) {
    if operation_name.is_null() {
        return;
    }
    
    unsafe {
        let op_name = CStr::from_ptr(operation_name).to_str().unwrap_or("unknown");
        
        info!(
            target: "nodejs::performance",
            operation = op_name,
            duration_ms = duration_ms,
            memory_used_mb = memory_used_mb,
            cpu_time_ms = cpu_time_ms,
            "Node.js performance metrics"
        );
    }
}
```

## Memory Safety and FFI

### Safe String Handling

```rust
use logfusion::{info, error};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// Safe string conversion utilities
pub fn safe_c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .ok()
            .map(|s| s.to_string())
    }
}

pub fn safe_string_to_c_str(s: &str) -> Result<CString, std::ffi::NulError> {
    CString::new(s)
}

// Safe logging with error handling
#[no_mangle]
pub extern "C" fn safe_log_message(
    level: c_int,
    target: *const c_char,
    message: *const c_char,
    context_keys: *const *const c_char,
    context_values: *const *const c_char,
    context_count: c_int,
) -> c_int {
    // Validate input parameters
    if target.is_null() || message.is_null() {
        return -1; // Invalid parameters
    }
    
    if context_count > 0 && (context_keys.is_null() || context_values.is_null()) {
        return -2; // Invalid context arrays
    }
    
    // Safe string extraction
    let target_str = match safe_c_str_to_string(target) {
        Some(s) => s,
        None => return -3, // Invalid target string
    };
    
    let message_str = match safe_c_str_to_string(message) {
        Some(s) => s,
        None => return -4, // Invalid message string
    };
    
    // Extract context safely
    let mut context = std::collections::HashMap::new();
    if context_count > 0 {
        unsafe {
            for i in 0..context_count as isize {
                let key_ptr = *context_keys.offset(i);
                let value_ptr = *context_values.offset(i);
                
                if let (Some(key), Some(value)) = (
                    safe_c_str_to_string(key_ptr),
                    safe_c_str_to_string(value_ptr)
                ) {
                    context.insert(key, value);
                }
            }
        }
    }
    
    // Log with structured context
    match level {
        1 => error!(
            target: &target_str,
            message = message_str,
            context = ?context,
            "FFI error message"
        ),
        2 => warn!(
            target: &target_str,
            message = message_str,
            context = ?context,
            "FFI warning message"
        ),
        _ => info!(
            target: &target_str,
            message = message_str,
            context = ?context,
            "FFI info message"
        ),
    }
    
    0 // Success
}

// Memory leak prevention
#[no_mangle]
pub extern "C" fn allocate_log_buffer(size: c_int) -> *mut c_char {
    if size <= 0 || size > 1024 * 1024 { // 1MB limit
        return std::ptr::null_mut();
    }
    
    unsafe {
        let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
        let ptr = std::alloc::alloc(layout) as *mut c_char;
        
        if !ptr.is_null() {
            info!(
                buffer_size = size,
                buffer_ptr = ?ptr,
                "Log buffer allocated"
            );
        }
        
        ptr
    }
}

#[no_mangle]
pub extern "C" fn free_log_buffer(ptr: *mut c_char, size: c_int) {
    if ptr.is_null() || size <= 0 {
        return;
    }
    
    unsafe {
        let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
        std::alloc::dealloc(ptr as *mut u8, layout);
        
        info!(
            buffer_ptr = ?ptr,
            buffer_size = size,
            "Log buffer freed"
        );
    }
}
```

## Performance Considerations

### High-Performance FFI Logging

```rust
use logfusion::{info, debug};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicU64, Ordering};

// Performance counters
static FFI_CALLS: AtomicU64 = AtomicU64::new(0);
static TOTAL_LOG_TIME_NS: AtomicU64 = AtomicU64::new(0);

// High-performance logging with minimal overhead
#[no_mangle]
pub extern "C" fn fast_log(level: c_int, message: *const c_char) {
    let start = std::time::Instant::now();
    
    // Increment call counter
    FFI_CALLS.fetch_add(1, Ordering::Relaxed);
    
    if message.is_null() {
        return;
    }
    
    unsafe {
        // Fast path: avoid string validation in release builds
        #[cfg(debug_assertions)]
        let message_str = CStr::from_ptr(message).to_str().unwrap_or("invalid");
        
        #[cfg(not(debug_assertions))]
        let message_str = CStr::from_ptr(message).to_string_lossy();
        
        match level {
            1 => error!(target: "ffi_fast", "{}", message_str),
            2 => warn!(target: "ffi_fast", "{}", message_str),
            _ => info!(target: "ffi_fast", "{}", message_str),
        }
    }
    
    // Update performance metrics
    let elapsed = start.elapsed();
    TOTAL_LOG_TIME_NS.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
}

// Batched logging for high-throughput scenarios
#[no_mangle]
pub extern "C" fn batch_log(
    messages: *const *const c_char,
    levels: *const c_int,
    count: c_int,
) -> c_int {
    if messages.is_null() || levels.is_null() || count <= 0 {
        return -1;
    }
    
    let mut processed = 0;
    
    unsafe {
        for i in 0..count as isize {
            let message_ptr = *messages.offset(i);
            let level = *levels.offset(i);
            
            if !message_ptr.is_null() {
                if let Ok(message_str) = CStr::from_ptr(message_ptr).to_str() {
                    match level {
                        1 => error!(target: "ffi_batch", batch_index = i, "{}", message_str),
                        2 => warn!(target: "ffi_batch", batch_index = i, "{}", message_str),
                        _ => info!(target: "ffi_batch", batch_index = i, "{}", message_str),
                    }
                    processed += 1;
                }
            }
        }
    }
    
    info!(
        target: "ffi_batch",
        total_messages = count,
        processed_messages = processed,
        "Batch logging completed"
    );
    
    processed
}

// Get performance statistics
#[no_mangle]
pub extern "C" fn get_ffi_performance_stats(
    total_calls: *mut c_int,
    avg_time_ns: *mut c_int,
) -> c_int {
    if total_calls.is_null() || avg_time_ns.is_null() {
        return -1;
    }
    
    let calls = FFI_CALLS.load(Ordering::Relaxed);
    let total_time = TOTAL_LOG_TIME_NS.load(Ordering::Relaxed);
    
    unsafe {
        *total_calls = calls as c_int;
        *avg_time_ns = if calls > 0 {
            (total_time / calls) as c_int
        } else {
            0
        };
    }
    
    debug!(
        total_ffi_calls = calls,
        average_time_ns = if calls > 0 { total_time / calls } else { 0 },
        "FFI performance statistics retrieved"
    );
    
    0
}
```

## Error Handling Across FFI

### Robust Error Propagation

```rust
use logfusion::{error, define_errors};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

// Define FFI-specific errors
define_errors! {
    FFIError {
        NullPointer { function: String } => "Null pointer passed to FFI function: {}",
        InvalidString { function: String, error: String } => "Invalid string in FFI function {}: {}",
        BufferOverflow { function: String, size: usize, max_size: usize } => "Buffer overflow in {}: size {} exceeds maximum {}",
        OperationFailed { function: String, error_code: i32 } => "Operation failed in {}: error code {}",
    }
}

// Error state management for FFI
static FFI_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

// Set the last error for retrieval by C code
fn set_last_error(error: &FFIError) {
    let error_message = error.to_string();
    error!(ffi_error = error_message, "FFI error occurred");
    
    if let Ok(mut last_error) = FFI_LAST_ERROR.lock() {
        *last_error = Some(error_message);
    }
}

// Retrieve the last error message
#[no_mangle]
pub extern "C" fn get_last_error(buffer: *mut c_char, buffer_size: c_int) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }
    
    if let Ok(last_error) = FFI_LAST_ERROR.lock() {
        if let Some(ref error_msg) = *last_error {
            let error_bytes = error_msg.as_bytes();
            let copy_size = std::cmp::min(error_bytes.len(), (buffer_size - 1) as usize);
            
            unsafe {
                std::ptr::copy_nonoverlapping(
                    error_bytes.as_ptr(),
                    buffer as *mut u8,
                    copy_size,
                );
                *buffer.add(copy_size) = 0; // Null terminate
            }
            
            return copy_size as c_int;
        }
    }
    
    0 // No error
}

// Clear the last error
#[no_mangle]
pub extern "C" fn clear_last_error() {
    if let Ok(mut last_error) = FFI_LAST_ERROR.lock() {
        *last_error = None;
    }
}

// Example FFI function with comprehensive error handling
#[no_mangle]
pub extern "C" fn process_data_with_error_handling(
    input: *const c_char,
    output: *mut c_char,
    output_size: c_int,
) -> c_int {
    // Validate input parameters
    if input.is_null() {
        let error = FFIError::NullPointer {
            function: "process_data_with_error_handling".to_string(),
        };
        set_last_error(&error);
        return -1;
    }
    
    if output.is_null() || output_size <= 0 {
        let error = FFIError::NullPointer {
            function: "process_data_with_error_handling".to_string(),
        };
        set_last_error(&error);
        return -2;
    }
    
    // Convert input string safely
    let input_str = unsafe {
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(e) => {
                let error = FFIError::InvalidString {
                    function: "process_data_with_error_handling".to_string(),
                    error: e.to_string(),
                };
                set_last_error(&error);
                return -3;
            }
        }
    };
    
    // Process the data
    let processed = format!("processed: {}", input_str);
    let processed_bytes = processed.as_bytes();
    
    // Check buffer size
    if processed_bytes.len() >= output_size as usize {
        let error = FFIError::BufferOverflow {
            function: "process_data_with_error_handling".to_string(),
            size: processed_bytes.len(),
            max_size: output_size as usize - 1,
        };
        set_last_error(&error);
        return -4;
    }
    
    // Copy result to output buffer
    unsafe {
        std::ptr::copy_nonoverlapping(
            processed_bytes.as_ptr(),
            output as *mut u8,
            processed_bytes.len(),
        );
        *output.add(processed_bytes.len()) = 0; // Null terminate
    }
    
    // Clear any previous error
    clear_last_error();
    
    processed_bytes.len() as c_int
}
```

## Advanced FFI Patterns

### Thread-Safe FFI Operations

```rust
use logfusion::{info, warn, error};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// Thread-safe global state management
pub struct FFIState {
    connections: HashMap<u32, DatabaseConnection>,
    next_id: u32,
}

impl FFIState {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
            next_id: 1,
        }
    }
}

static FFI_STATE: Mutex<FFIState> = Mutex::new(FFIState {
    connections: HashMap::new(),
    next_id: 1,
});

// Thread-safe connection management
#[no_mangle]
pub extern "C" fn create_connection(connection_string: *const c_char) -> c_int {
    if connection_string.is_null() {
        error!("Null connection string provided");
        return -1;
    }
    
    let conn_str = unsafe {
        match CStr::from_ptr(connection_string).to_str() {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "Invalid connection string");
                return -2;
            }
        }
    };
    
    info!(connection_string = conn_str, "Creating new connection");
    
    // Create connection (mock implementation)
    let connection = DatabaseConnection {
        handle: std::ptr::null_mut(), // Mock handle
    };
    
    if let Ok(mut state) = FFI_STATE.lock() {
        let id = state.next_id;
        state.connections.insert(id, connection);
        state.next_id += 1;
        
        info!(connection_id = id, "Connection created successfully");
        return id as c_int;
    }
    
    error!("Failed to acquire FFI state lock");
    -3
}

#[no_mangle]
pub extern "C" fn destroy_connection(connection_id: c_int) -> c_int {
    if connection_id <= 0 {
        error!(connection_id = connection_id, "Invalid connection ID");
        return -1;
    }
    
    info!(connection_id = connection_id, "Destroying connection");
    
    if let Ok(mut state) = FFI_STATE.lock() {
        if state.connections.remove(&(connection_id as u32)).is_some() {
            info!(connection_id = connection_id, "Connection destroyed successfully");
            return 0;
        } else {
            warn!(connection_id = connection_id, "Connection not found");
            return -2;
        }
    }
    
    error!("Failed to acquire FFI state lock");
    -3
}

// Async operation support
use std::sync::mpsc;
use std::thread;

#[no_mangle]
pub extern "C" fn async_operation(
    operation_id: c_int,
    callback: extern "C" fn(c_int, c_int, *const c_char),
) -> c_int {
    info!(operation_id = operation_id, "Starting async operation");
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn background thread for async work
    thread::spawn(move || {
        // Simulate async work
        thread::sleep(std::time::Duration::from_millis(100));
        
        let result = format!("Operation {} completed", operation_id);
        let result_cstr = CString::new(result).unwrap();
        
        info!(operation_id = operation_id, "Async operation completed");
        
        // Call back to C code
        callback(operation_id, 0, result_cstr.as_ptr());
        
        // Keep the CString alive for the callback
        tx.send(result_cstr).ok();
    });
    
    0 // Success - operation started
}
```

## Next Steps

You now have comprehensive knowledge of LogFusion's capabilities. Consider exploring:

- **Integration with specific monitoring platforms** (Datadog, New Relic, etc.)
- **Custom subscriber development** for specialized logging needs
- **Performance optimization** for high-throughput scenarios
- **Security considerations** for production deployments

## Troubleshooting

### Common FFI Issues

**Q: Getting segmentation faults in FFI calls**

```rust
// Always validate pointers before use
#[no_mangle]
pub extern "C" fn safe_ffi_function(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        error!("Null pointer passed to FFI function");
        return -1;
    }
    
    // Proceed with validated pointer
    0
}
```

**Q: String encoding issues across language boundaries**

```rust
// Use proper string validation
unsafe {
    match CStr::from_ptr(c_string).to_str() {
        Ok(s) => info!(message = s, "Valid UTF-8 string"),
        Err(e) => error!(error = %e, "Invalid UTF-8 in C string"),
    }
}
```

**Q: Memory leaks in FFI logging**

```rust
// Always manage string lifetimes properly
let c_string = CString::new("message").unwrap();
let ptr = c_string.as_ptr();
// Keep c_string alive while ptr is used
call_c_function(ptr);
// c_string is dropped here, cleaning up memory
```
