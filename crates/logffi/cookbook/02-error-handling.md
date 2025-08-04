# Error Handling with define_errors!

The `define_errors!` macro provides a powerful way to create error types with automatic logging, error codes, and source chaining.

## Basic Error Definition

```rust
use logffi::define_errors;

define_errors! {
    pub enum AppError {
        #[error("Configuration file not found: {path}")]
        ConfigNotFound {
            path: String,
        },
        
        #[error("Invalid configuration value: {key} = {value}")]
        InvalidConfig {
            key: String,
            value: String,
        },
        
        #[error("Database connection failed")]
        DatabaseConnectionFailed,
    }
}

// What you get for free:
// ✅ Display trait implementation
// ✅ Error trait implementation
// ✅ Debug trait
// ✅ code() method
// ✅ kind() method
// ✅ log() method
// ✅ Constructor methods (new_config_not_found, etc.)
```

## Using the Generated Error Types

```rust
fn load_config(path: &str) -> Result<Config, AppError> {
    // Method 1: Manual creation
    if !std::path::Path::new(path).exists() {
        let error = AppError::ConfigNotFound {
            path: path.to_string(),
        };
        error.log();  // Manually log
        return Err(error);
    }
    
    // Method 2: Using constructor (Recommended!)
    // Constructor automatically logs the error
    if !validate_path(path) {
        return Err(AppError::new_config_not_found(path.to_string()));
    }
    
    // Parse config...
    Ok(config)
}
```

## Error Levels and Targets

```rust
define_errors! {
    pub enum ServiceError {
        // Warnings for recoverable issues
        #[error("Cache miss for key: {key}", level = warn, target = "cache")]
        CacheMiss {
            key: String,
        },
        
        // Errors for failures
        #[error("API request failed: {endpoint}", level = error, target = "api::client")]
        ApiRequestFailed {
            endpoint: String,
        },
        
        // Debug for detailed diagnostics
        #[error("Retry attempt {attempt} for {operation}", level = debug, target = "retry")]
        RetryAttempt {
            operation: String,
            attempt: u32,
        },
        
        // Info for important events
        #[error("Rate limit reached, throttling", level = info, target = "ratelimit")]
        RateLimitReached,
    }
}

// Usage shows different log levels
fn api_call_with_retry() -> Result<Response, ServiceError> {
    for attempt in 1..=3 {
        // This logs at DEBUG level to "retry" target
        ServiceError::new_retry_attempt("api_call".to_string(), attempt);
        
        match make_request() {
            Ok(response) => return Ok(response),
            Err(_) if attempt < 3 => continue,
            Err(_) => {
                // This logs at ERROR level to "api::client" target
                return Err(ServiceError::new_api_request_failed(
                    "/users".to_string()
                ));
            }
        }
    }
    unreachable!()
}
```

**Log Output:**

```
[DEBUG retry] Retry attempt 1 for api_call
[DEBUG retry] Retry attempt 2 for api_call
[DEBUG retry] Retry attempt 3 for api_call
[ERROR api::client] API request failed: /users
```

## Custom Error Codes

```rust
define_errors! {
    pub enum BusinessError {
        // Auto-generated code: "UserNotFound"
        #[error("User not found: {id}")]
        UserNotFound {
            id: u64,
        },
        
        // Custom error code for monitoring/alerting
        #[error("Payment processing failed", code = "PAY_001")]
        PaymentFailed,
        
        #[error("Insufficient funds", code = "PAY_002")]
        InsufficientFunds,
        
        #[error("Invalid card", code = "PAY_003")]
        InvalidCard,
    }
}

fn process_payment() -> Result<(), BusinessError> {
    // Error codes are useful for:
    // 1. Monitoring and alerting
    // 2. API responses
    // 3. Error tracking systems
    
    Err(BusinessError::new_payment_failed())
}

// In your monitoring system:
fn alert_on_error(error: &BusinessError) {
    match error.code() {
        "PAY_001" => send_critical_alert("Payment system down"),
        "PAY_002" => increment_metric("payments.insufficient_funds"),
        "PAY_003" => log_fraud_attempt(),
        _ => {}
    }
}
```

## FFI-Friendly Error Handling

```rust
use std::ffi::CString;

define_errors! {
    pub enum FfiError {
        #[error("Invalid UTF-8 in string", code = "FFI_UTF8")]
        InvalidUtf8,
        
        #[error("Null pointer provided", code = "FFI_NULL")]
        NullPointer,
        
        #[error("Buffer too small: need {required}, got {provided}", code = "FFI_BUF")]
        BufferTooSmall {
            required: usize,
            provided: usize,
        },
    }
}

// FFI-friendly error info
#[repr(C)]
pub struct ErrorInfo {
    pub code: *const i8,
    pub message: *const i8,
}

// Convert errors for FFI
impl From<&FfiError> for ErrorInfo {
    fn from(error: &FfiError) -> Self {
        ErrorInfo {
            code: CString::new(error.code()).unwrap().into_raw(),
            message: CString::new(error.to_string()).unwrap().into_raw(),
        }
    }
}

// Example FFI function
#[no_mangle]
pub extern "C" fn process_data(
    input: *const u8,
    len: usize,
    error_out: *mut ErrorInfo,
) -> i32 {
    if input.is_null() {
        let error = FfiError::new_null_pointer();
        unsafe {
            *error_out = ErrorInfo::from(&error);
        }
        return -1;
    }
    
    // Process data...
    0
}
```

## Integration with Result and Option

```rust
define_errors! {
    pub enum DataError {
        #[error("Record not found: {id}")]
        NotFound {
            id: String,
        },
        
        #[error("Invalid data format")]
        InvalidFormat,
    }
}

// Extension traits for convenient error handling
trait ResultExt<T> {
    fn or_not_found(self, id: impl Into<String>) -> Result<T, DataError>;
}

impl<T> ResultExt<T> for Option<T> {
    fn or_not_found(self, id: impl Into<String>) -> Result<T, DataError> {
        self.ok_or_else(|| DataError::new_not_found(id.into()))
    }
}

// Clean usage
fn get_user(id: &str) -> Result<User, DataError> {
    database::find_user(id)
        .or_not_found(id)?  // Automatically creates and logs error
        .parse()
        .map_err(|_| DataError::new_invalid_format())
}
```
