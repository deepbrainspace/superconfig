# Error Handling with LogFusion's Dual-Syntax Approach

This guide demonstrates LogFusion v0.2's **revolutionary dual-syntax `define_errors!` macro** that combines the simplicity of the new LogFusion format with full backward compatibility with thiserror syntax.

**🆕 What's New in v0.2:**

- **LogFusion Format** - Clean, attribute-based syntax for modern error definitions
- **64% Macro Optimization** - Reduced from 998 to 358 lines while adding features
- **Multiple Error Types** - Define multiple enums in one macro call
- **Auto Source Chaining** - Fields named `source` automatically become `#[source]`
- **Mixed Variants** - Unit and struct variants in the same enum
- **11 Comprehensive Tests** - Every scenario battle-tested for reliability

## Table of Contents

- [🆕 LogFusion Format Overview](#-logfusion-format-overview)
- [🔧 Thiserror Compatibility](#-thiserror-compatibility)
- [📦 Basic Error Types](#-basic-error-types)
- [🔀 Mixed Variant Types](#-mixed-variant-types)
- [📊 Logging Levels & Targets](#-logging-levels--targets)
- [⛓️ Automatic Source Chaining](#️-automatic-source-chaining)
- [🔧 Multiple Error Types](#-multiple-error-types)
- [🌍 Real-World Examples](#-real-world-examples)
- [💡 Best Practices](#-best-practices)

## 🆕 LogFusion Format Overview

The new LogFusion format provides a clean, powerful syntax for error definitions:

```rust
use logfusion::define_errors;

// 🆕 LogFusion Format - Clean, attribute-based syntax
define_errors! {
    UserError {
        NotFound { user_id: u64 } : "User {user_id} not found" [level = warn, target = "auth::users"],
        InvalidCredentials { username: String } : "Invalid credentials for user: {username}" [level = error],
        AccountLocked { 
            user_id: u64, 
            locked_until: String 
        } : "Account {user_id} locked until {locked_until}" [level = warn],
        ServiceUnavailable {} : "Authentication service temporarily unavailable" [level = error]
    }
}

fn authenticate_user(username: &str, password: &str) -> Result<User, UserError> {
    let user = find_user(username).map_err(|_| UserError::NotFound { 
        user_id: 12345 
    })?;
    
    if user.is_locked() {
        let err = UserError::AccountLocked {
            user_id: user.id,
            locked_until: user.locked_until.to_string(),
        };
        err.log(); // WARN auth::users: [AccountLocked] Account 12345 locked until 2024-01-15
        return Err(err);
    }
    
    if !verify_password(password, &user.password_hash) {
        let err = UserError::InvalidCredentials {
            username: username.to_string(),
        };
        err.log(); // ERROR auth::module: [InvalidCredentials] Invalid credentials for user: alice
        return Err(err);
    }
    
    Ok(user)
}
```

**Key LogFusion Format Features:**

- ✅ **Clean Syntax** - No repetitive `#[error(...)]` attributes
- ✅ **Field Interpolation** - `{user_id}` syntax in messages
- ✅ **Attribute-Based Logging** - `[level = warn, target = "auth::users"]`
- ✅ **Unit & Struct Variants** - Mix empty `{}` and `{ fields }` variants
- ✅ **Automatic Methods** - `.log()`, `.code()`, `.to_string()` generated

## 🔧 Thiserror Compatibility

LogFusion maintains **full backward compatibility** with existing thiserror syntax:

```rust
use logfusion::define_errors;

// Traditional thiserror syntax (still fully supported)
define_errors! {
    pub enum DatabaseError {
        #[error("Connection failed to {host}:{port}", level = error, target = "db::connection")]
        ConnectionFailed { host: String, port: u16 },
        
        #[error("Query timeout after {timeout_ms}ms", level = warn)]
        QueryTimeout { timeout_ms: u64 },
        
        #[error("Transaction rollback required")]
        TransactionFailed,
    }
}

// Mix both syntaxes if needed during migration
define_errors! {
    pub enum MixedSyntaxError {
        #[error("Legacy syntax still works")]
        LegacyVariant,
    }
    
    ModernError {
        NewVariant { field: String } : "Modern LogFusion syntax: {field}" [level = info]
    }
}
```

## 📦 Basic Error Types

### Unit Variants (No Fields)

```rust
define_errors! {
    SimpleError {
        NotFound {} : "Resource not found" [level = warn],
        Unauthorized {} : "Access denied" [level = error],
        ServiceUnavailable {} : "Service temporarily unavailable" [level = error]
    }
}

// Usage
let err = SimpleError::NotFound;
assert_eq!(err.code(), "NotFound");
assert_eq!(err.to_string(), "Resource not found");
err.log(); // WARN module_path: [NotFound] Resource not found
```

### Struct Variants (With Fields)

```rust
define_errors! {
    ValidationError {
        InvalidEmail { email: String } : "Invalid email address: {email}" [level = warn],
        PasswordTooShort { 
            length: usize, 
            min_length: usize 
        } : "Password length {length} below minimum {min_length}" [level = error],
        MissingField { field_name: String } : "Required field missing: {field_name}" [level = error]
    }
}

// Usage with field interpolation
let err = ValidationError::PasswordTooShort { 
    length: 4, 
    min_length: 8 
};
assert_eq!(err.to_string(), "Password length 4 below minimum 8");
err.log(); // ERROR module_path: [PasswordTooShort] Password length 4 below minimum 8
```

## 🔀 Mixed Variant Types

**Most Powerful Feature** - Mix unit and struct variants in the same enum:

```rust
define_errors! {
    PaymentError {
        // Unit variants (simple cases)
        InvalidCard {} : "Invalid card number" [level = warn],
        NetworkTimeout {} : "Payment network timeout" [level = error],
        
        // Struct variants (complex cases with data)
        InsufficientFunds { 
            amount: f64, 
            available: f64 
        } : "Need ${amount}, have ${available}" [level = error],
        
        ProcessingFailed { 
            transaction_id: String, 
            reason: String 
        } : "Transaction {transaction_id} failed: {reason}" [level = error],
        
        // Mix with source chaining
        NetworkError { 
            source: std::io::Error 
        } : "Network error occurred"
    }
}

// All variants work seamlessly
let simple_err = PaymentError::InvalidCard;
let complex_err = PaymentError::InsufficientFunds { amount: 100.0, available: 50.0 };
let chained_err = PaymentError::NetworkError { 
    source: std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout")
};

// All have the same interface
simple_err.log();
complex_err.log();
assert!(chained_err.source().is_some()); // Source chaining works
```

## 📊 Logging Levels & Targets

### All Log Levels Supported

```rust
define_errors! {
    SystemError {
        CriticalFailure {} : "System critical failure" [level = error],
        ConfigurationWarning {} : "Non-optimal configuration detected" [level = warn], 
        StartupInfo {} : "System initialization completed" [level = info],
        DebugInfo { details: String } : "Debug information: {details}" [level = debug],
        TraceDetail { step: u32 } : "Processing step {step}" [level = trace]
    }
}
```

### Custom Targets for Structured Logging

```rust
define_errors! {
    ApplicationError {
        DatabaseError {} : "Database connection failed" [level = error, target = "app::database"],
        NetworkError {} : "Network request failed" [level = warn, target = "app::network"],
        AuthError {} : "Authentication failed" [level = info, target = "app::auth"],
        DefaultError {} : "Uses module_path!() as target" [level = error]
    }
}

// Results in structured log messages:
// ERROR app::database: [DatabaseError] Database connection failed
// WARN  app::network: [NetworkError] Network request failed  
// INFO  app::auth: [AuthError] Authentication failed
// ERROR my_module::sub_module: [DefaultError] Uses module_path!() as target
```

## ⛓️ Automatic Source Chaining

Fields named `source` are **automatically** detected and become `#[source]`:

```rust
define_errors! {
    ChainedError {
        // Single source
        IoError { 
            source: std::io::Error 
        } : "IO operation failed",
        
        // Source with additional context
        DatabaseError {
            operation: String,
            source: std::io::Error,
            retry_count: u32
        } : "Database operation {operation} failed after {retry_count} retries",
        
        // Any Error type works as source
        GenericError {
            context: String,
            source: Box<dyn std::error::Error + Send + Sync>
        } : "Operation failed: {context}"
    }
}

// Source chaining works automatically
let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
let db_err = ChainedError::DatabaseError {
    operation: "SELECT".to_string(),
    source: io_err,
    retry_count: 3,
};

assert!(db_err.source().is_some());
assert_eq!(db_err.source().unwrap().to_string(), "File not found");
assert_eq!(db_err.to_string(), "Database operation SELECT failed after 3 retries");
```

## 🔧 Multiple Error Types

Define multiple error enums in a **single macro call**:

```rust
define_errors! {
    // First error type
    ApiError {
        BadRequest { field: String } : "Invalid field: {field}" [level = warn],
        Unauthorized {} : "Access denied" [level = error],
        RateLimited { retry_after: u64 } : "Rate limited, retry after {retry_after}s" [level = warn]
    }
    
    // Second error type in same macro
    DatabaseError {
        ConnectionFailed { host: String } : "Failed to connect to {host}" [level = error],
        QueryTimeout {} : "Query timed out" [level = warn],
        TransactionFailed { reason: String } : "Transaction failed: {reason}" [level = error]
    }
}

// Each gets its own enum with full functionality
let api_err = ApiError::BadRequest { field: "email".to_string() };
let db_err = DatabaseError::ConnectionFailed { host: "localhost".to_string() };

api_err.log(); // WARN module_path: [BadRequest] Invalid field: email
db_err.log();  // ERROR module_path: [ConnectionFailed] Failed to connect to localhost
```

## 🌍 Real-World Examples

### E-commerce Payment Processing

```rust
define_errors! {
    PaymentError {
        // Simple validation errors
        InvalidCard {} : "Invalid card number format" [level = warn],
        ExpiredCard {} : "Card has expired" [level = warn],
        
        // Business logic errors with data
        InsufficientFunds { 
            requested: f64, 
            available: f64 
        } : "Insufficient funds: requested ${requested}, available ${available}" [level = error],
        
        // External service errors with chaining
        ProcessorError { 
            processor: String,
            source: std::io::Error 
        } : "Payment processor {processor} unavailable",
        
        // Fraud detection
        SuspiciousActivity { 
            user_id: u64, 
            risk_score: f32 
        } : "Suspicious activity detected for user {user_id}, risk score: {risk_score}" [level = error, target = "fraud::detection"]
    }
}

fn process_payment(amount: f64, card: &Card) -> Result<PaymentResult, PaymentError> {
    if !card.is_valid() {
        let err = PaymentError::InvalidCard;
        err.log(); // WARN module_path: [InvalidCard] Invalid card number format
        return Err(err);
    }
    
    if card.is_expired() {
        return Err(PaymentError::ExpiredCard);
    }
    
    let balance = get_balance(card)?;
    if balance < amount {
        return Err(PaymentError::InsufficientFunds {
            requested: amount,
            available: balance,
        });
    }
    
    // Process payment...
    Ok(PaymentResult::Success)
}
```

### Microservice API Error Handling

```rust
define_errors! {
    // User service errors
    UserServiceError {
        UserNotFound { user_id: u64 } : "User {user_id} not found" [level = warn, target = "service::users"],
        DuplicateEmail { email: String } : "Email {email} already registered" [level = warn],
        ValidationFailed { field: String, reason: String } : "Validation failed for {field}: {reason}" [level = error]
    }
    
    // Order service errors  
    OrderServiceError {
        OrderNotFound { order_id: String } : "Order {order_id} not found" [level = warn, target = "service::orders"],
        InvalidStatus { 
            current: String, 
            requested: String 
        } : "Cannot change order status from {current} to {requested}" [level = error],
        InsufficientInventory { 
            product_id: String, 
            requested: u32, 
            available: u32 
        } : "Product {product_id}: requested {requested}, available {available}" [level = error]
    }
}

#[tracing::instrument(level = "info")]
async fn create_order(user_id: u64, items: Vec<OrderItem>) -> Result<Order, OrderServiceError> {
    // Validate user exists
    let _user = get_user(user_id).await.map_err(|_| {
        let err = UserServiceError::UserNotFound { user_id };
        err.log(); // WARN service::users: [UserNotFound] User 12345 not found
        err
    })?;
    
    // Check inventory
    for item in &items {
        let available = get_inventory(item.product_id).await?;
        if available < item.quantity {
            return Err(OrderServiceError::InsufficientInventory {
                product_id: item.product_id.clone(),
                requested: item.quantity,
                available,
            });
        }
    }
    
    // Create order...
    Ok(Order::new(user_id, items))
}
```

## 💡 Best Practices

### 1. **Choose the Right Format**

```rust
// ✅ Use LogFusion format for new code - cleaner and more powerful
define_errors! {
    NewServiceError {
        ValidationFailed { field: String } : "Invalid {field}" [level = warn]
    }
}

// ✅ Keep thiserror format for existing code - no need to migrate immediately
define_errors! {
    pub enum ExistingError {
        #[error("Legacy error: {message}")]
        LegacyError { message: String },
    }
}
```

### 2. **Use Appropriate Log Levels**

```rust
define_errors! {
    WellLeveledError {
        // ERROR - Actual problems requiring attention
        SystemFailure {} : "Critical system failure" [level = error],
        
        // WARN - Issues that need monitoring but don't break functionality  
        RateLimitApproaching { current: u32, limit: u32 } : "Rate limit approaching: {current}/{limit}" [level = warn],
        
        // INFO - Business events worth recording
        UserAction { user_id: u64, action: String } : "User {user_id} performed {action}" [level = info]
    }
}
```

### 3. **Structure Targets for Observability**

```rust
define_errors! {
    ObservableError {
        // Group by service/component
        DatabaseError {} : "DB connection failed" [level = error, target = "service::database"],
        ApiError {} : "External API failed" [level = error, target = "service::external_api"],
        
        // Group by business domain
        PaymentError {} : "Payment processing failed" [level = error, target = "domain::payments"],
        UserError {} : "User management error" [level = warn, target = "domain::users"]
    }
}
```

### 4. **Test Error Scenarios**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_logging_and_codes() {
        let err = PaymentError::InsufficientFunds { 
            requested: 100.0, 
            available: 50.0 
        };
        
        // Test structured fields
        assert_eq!(err.code(), "InsufficientFunds");
        assert_eq!(err.to_string(), "Insufficient funds: requested $100, available $50");
        
        // Test logging (use test subscriber to capture)
        err.log(); // Logs to tracing system
        
        // Test error chaining
        assert!(err.source().is_none()); // No source for this error
    }
    
    #[test]
    fn test_source_chaining() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let payment_err = PaymentError::ProcessorError {
            processor: "stripe".to_string(),
            source: io_err,
        };
        
        // Verify source chain
        assert!(payment_err.source().is_some());
        assert_eq!(payment_err.source().unwrap().to_string(), "File not found");
    }
}
```

### 5. **Performance Considerations**

```rust
// ✅ Good - Reasonable field count and types
define_errors! {
    EfficientError {
        RequestFailed { 
            status_code: u16, 
            method: String 
        } : "Request failed: {method} returned {status_code}" [level = warn]
    }
}

// ❌ Avoid - Too many fields or complex types that are expensive to format
define_errors! {
    InefficientError {
        OverlyDetailed { 
            request_body: String,      // Could be very large
            response_headers: HashMap<String, String>,  // Complex formatting
            stack_trace: Vec<String>,  // Potentially huge
        } : "Complex error with too much data"
    }
}
```

### 6. **Migration Strategy**

```rust
// Phase 1: Keep existing thiserror syntax working
define_errors! {
    pub enum LegacyError {
        #[error("Old style error")]
        OldVariant,
    }
}

// Phase 2: Add new variants using LogFusion format in same enum
define_errors! {
    pub enum MixedError {
        #[error("Still using old syntax")]
        OldVariant,
    }
    
    NewError {
        NewVariant {} : "Using new LogFusion syntax" [level = warn]
    }
}

// Phase 3: Gradually migrate old variants to new format as you touch them
define_errors! {
    NewError {
        ModernVariant {} : "All modern LogFusion syntax" [level = warn],
        AnotherModern { id: u64 } : "Modern with fields: {id}" [level = error]
    }
}
```

**The LogFusion dual-syntax approach gives you the power of modern error handling while respecting your existing codebase investments.**

    info!(
        user_id = user.id,
        username = username,
        "User authentication successful"
    );

    Ok(user)

}

````
## The define_errors! Macro

LogFusion's `define_errors!` macro creates error types with automatic structured logging:

### Basic Syntax

```rust
use logfusion::define_errors;

define_errors! {
    ErrorTypeName {
        VariantName { field1: Type1, field2: Type2 } : "Format string with {} and {}",
        AnotherVariant { field: Type } : "Another message with {}",
        SimpleVariant {} : "Simple error message",
    }
}
````

### Error Definition Examples

```rust
use logfusion::define_errors;

// Database errors
define_errors! {
    DatabaseError {
        ConnectionFailed { host: String, port: u16 } => "Failed to connect to database at {}:{}",
        QueryTimeout { query: String, timeout_ms: u64 } => "Query timed out after {}ms: {}",
        TableNotFound { table_name: String } => "Table '{}' does not exist",
        DuplicateKey { key: String, table: String } => "Duplicate key '{}' in table '{}'",
        ConstraintViolation { constraint: String } => "Constraint violation: {}",
    }
}

// API errors
define_errors! {
    ApiError {
        BadRequest { field: String, reason: String } => "Invalid field '{}': {}",
        Unauthorized { user_id: Option<u64> } => "Unauthorized access for user: {:?}",
        RateLimit { limit: u32, window_seconds: u32 } => "Rate limit exceeded: {} requests per {} seconds",
        ServiceUnavailable { service: String, retry_after: u64 } => "Service '{}' unavailable, retry after {} seconds",
    }
}

// Payment errors
define_errors! {
    PaymentError {
        InsufficientFunds { 
            account_id: String, 
            available: f64, 
            required: f64 
        } => "Insufficient funds in account {}: available ${:.2}, required ${:.2}",
        
        PaymentDeclined { 
            transaction_id: String, 
            decline_code: String,
            decline_reason: String 
        } => "Payment {} declined: {} - {}",
        
        InvalidCard { 
            masked_number: String 
        } => "Invalid card number: {}",
        
        ExpiredCard { 
            expiry_date: String 
        } => "Card expired: {}",
    }
}
```

## Basic Error Types

### Simple Error Variants

```rust
use logfusion::{define_errors, warn};

define_errors! {
    ValidationError {
        EmailInvalid => "Invalid email address format",
        PasswordTooShort => "Password must be at least 8 characters",
        UsernameAlreadyExists => "Username is already taken",
    }
}

fn validate_user_input(email: &str, password: &str, username: &str) -> Result<(), ValidationError> {
    if !is_valid_email(email) {
        return Err(ValidationError::EmailInvalid);
    }
    
    if password.len() < 8 {
        return Err(ValidationError::PasswordTooShort);
    }
    
    if user_exists(username) {
        return Err(ValidationError::UsernameAlreadyExists);
    }
    
    Ok(())
}
```

### Parametric Error Variants

```rust
use logfusion::define_errors;

define_errors! {
    FileSystemError {
        FileNotFound { path: String } => "File not found: {}",
        PermissionDenied { path: String, operation: String } => "Permission denied for {} operation on {}",
        DiskFull { available_bytes: u64, required_bytes: u64 } => "Disk full: {} bytes available, {} bytes required",
        InvalidPath { path: String, reason: String } => "Invalid path '{}': {}",
    }
}

fn read_config_file(path: &str) -> Result<String, FileSystemError> {
    if !std::path::Path::new(path).exists() {
        return Err(FileSystemError::FileNotFound {
            path: path.to_string(),
        });
    }
    
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(FileSystemError::PermissionDenied {
                path: path.to_string(),
                operation: "read".to_string(),
            })
        },
        Err(e) => {
            // Handle other IO errors...
            Err(FileSystemError::InvalidPath {
                path: path.to_string(),
                reason: e.to_string(),
            })
        }
    }
}
```

## Structured Error Logging

LogFusion errors automatically log with structured fields when created:

```rust
use logfusion::{define_errors, info, warn};

define_errors! {
    OrderError {
        ProductOutOfStock { 
            product_id: u64, 
            requested_quantity: u32, 
            available_quantity: u32 
        } => "Product {} out of stock: requested {}, available {}",
        
        InvalidDiscount { 
            discount_code: String, 
            user_id: u64, 
            reason: String 
        } => "Invalid discount code '{}' for user {}: {}",
        
        PaymentProcessingFailed { 
            order_id: String, 
            payment_method: String, 
            error_code: String 
        } => "Payment processing failed for order {} using {}: {}",
    }
}

fn process_order(order: &Order) -> Result<OrderConfirmation, OrderError> {
    // Check inventory
    for item in &order.items {
        let available = get_inventory(item.product_id);
        if available < item.quantity {
            // This error is automatically logged with structured fields:
            // ERROR order_error="ProductOutOfStock" product_id=123 requested_quantity=5 available_quantity=2
            return Err(OrderError::ProductOutOfStock {
                product_id: item.product_id,
                requested_quantity: item.quantity,
                available_quantity: available,
            });
        }
    }
    
    // Validate discount
    if let Some(discount) = &order.discount_code {
        if !is_valid_discount(discount, order.user_id) {
            return Err(OrderError::InvalidDiscount {
                discount_code: discount.clone(),
                user_id: order.user_id,
                reason: "Code expired or not applicable".to_string(),
            });
        }
    }
    
    // Process payment
    match process_payment(&order.payment) {
        Ok(payment_result) => {
            info!(
                order_id = order.id,
                user_id = order.user_id,
                total_amount = order.total,
                payment_method = order.payment.method,
                "Order processed successfully"
            );
            Ok(OrderConfirmation::new(order, payment_result))
        },
        Err(payment_error) => {
            Err(OrderError::PaymentProcessingFailed {
                order_id: order.id.clone(),
                payment_method: order.payment.method.clone(),
                error_code: payment_error.code(),
            })
        }
    }
}
```

## Error Context and Chaining

Combine LogFusion errors with external error types and context:

```rust
use logfusion::{define_errors, error};

define_errors! {
    ServiceError {
        DatabaseUnavailable { 
            operation: String, 
            underlying_error: String 
        } => "Database unavailable for operation '{}': {}",
        
        ExternalServiceFailed { 
            service_name: String, 
            endpoint: String, 
            status_code: u16 
        } => "External service '{}' failed at {}: HTTP {}",
        
        ConfigurationError { 
            config_key: String, 
            expected_type: String 
        } => "Invalid configuration for '{}': expected {}",
    }
}

// Implementing From for error conversion with context
impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseUnavailable {
            operation: "database_query".to_string(),
            underlying_error: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(err: reqwest::Error) -> Self {
        let status_code = err.status().map(|s| s.as_u16()).unwrap_or(0);
        let url = err.url().map(|u| u.to_string()).unwrap_or_else(|| "unknown".to_string());
        
        ServiceError::ExternalServiceFailed {
            service_name: "external_api".to_string(),
            endpoint: url,
            status_code,
        }
    }
}

async fn fetch_user_data(user_id: u64) -> Result<UserData, ServiceError> {
    // Database operation - automatically converts sqlx::Error to ServiceError
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&pool)
        .await?;
    
    // External API call - automatically converts reqwest::Error to ServiceError
    let external_data = reqwest::get(&format!("https://api.example.com/users/{}", user_id))
        .await?
        .json::<ExternalUserData>()
        .await?;
    
    Ok(UserData::combine(user, external_data))
}
```

## Integration with Result Types

Use LogFusion errors in comprehensive error handling patterns:

```rust
use logfusion::{define_errors, info, warn, error};

define_errors! {
    UserServiceError {
        UserNotFound { user_id: u64 } => "User {} not found",
        EmailAlreadyExists { email: String } => "Email '{}' is already registered",
        InvalidOperation { user_id: u64, operation: String } => "Invalid operation '{}' for user {}",
        PermissionDenied { user_id: u64, resource: String } => "User {} denied access to resource '{}'",
    }
}

struct UserService {
    database: Database,
    cache: Cache,
}

impl UserService {
    async fn create_user(&self, user_data: CreateUserRequest) -> Result<User, UserServiceError> {
        // Check if email already exists
        if self.database.user_exists_by_email(&user_data.email).await? {
            return Err(UserServiceError::EmailAlreadyExists {
                email: user_data.email,
            });
        }
        
        // Create user
        let user = self.database.create_user(user_data).await?;
        
        // Update cache
        self.cache.set_user(user.id, &user).await;
        
        info!(
            user_id = user.id,
            email = user.email,
            "User created successfully"
        );
        
        Ok(user)
    }
    
    async fn update_user_profile(&self, user_id: u64, updates: ProfileUpdate) -> Result<User, UserServiceError> {
        // Verify user exists
        let mut user = self.get_user(user_id).await?;
        
        // Check permissions
        if !user.can_update_profile() {
            return Err(UserServiceError::PermissionDenied {
                user_id,
                resource: "profile".to_string(),
            });
        }
        
        // Apply updates
        user.apply_updates(updates);
        
        // Save to database
        let updated_user = self.database.update_user(user).await?;
        
        // Invalidate cache
        self.cache.delete_user(user_id).await;
        
        info!(
            user_id = user_id,
            fields_updated = format!("{:?}", updates.changed_fields()),
            "User profile updated"
        );
        
        Ok(updated_user)
    }
    
    async fn get_user(&self, user_id: u64) -> Result<User, UserServiceError> {
        // Try cache first
        if let Some(user) = self.cache.get_user(user_id).await {
            return Ok(user);
        }
        
        // Fetch from database
        match self.database.get_user(user_id).await? {
            Some(user) => {
                // Cache for next time
                self.cache.set_user(user_id, &user).await;
                Ok(user)
            },
            None => Err(UserServiceError::UserNotFound { user_id }),
        }
    }
}
```

## Advanced Error Patterns

### Error Recovery and Retry Logic

```rust
use logfusion::{define_errors, warn, info};
use std::time::Duration;

define_errors! {
    RetryableError {
        ServiceTemporarilyUnavailable { 
            service: String, 
            retry_count: u32, 
            max_retries: u32 
        } => "Service '{}' temporarily unavailable (attempt {}/{})",
        
        NetworkTimeout { 
            endpoint: String, 
            timeout_ms: u64,
            attempt: u32 
        } => "Network timeout connecting to {} after {}ms (attempt {})",
        
        RateLimitExceeded { 
            service: String, 
            reset_time: u64,
            attempt: u32 
        } => "Rate limit exceeded for service '{}', reset at {} (attempt {})",
    }
}

async fn retry_with_backoff<F, T, E>(
    operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, RetryableError>
where
    F: Fn() -> Result<T, E>,
    E: Into<RetryableError>,
{
    for attempt in 1..=max_retries {
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt = attempt,
                        max_retries = max_retries,
                        "Operation succeeded after retries"
                    );
                }
                return Ok(result);
            },
            Err(e) => {
                let error = e.into();
                
                if attempt < max_retries {
                    let delay = base_delay * 2_u32.pow(attempt - 1);
                    warn!(
                        attempt = attempt,
                        max_retries = max_retries,
                        delay_ms = delay.as_millis() as u64,
                        error = format!("{}", error),
                        "Operation failed, retrying after delay"
                    );
                    
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(error);
                }
            }
        }
    }
    
    unreachable!()
}
```

### Error Aggregation

```rust
use logfusion::{define_errors, error, warn};

define_errors! {
    ValidationErrors {
        MultipleFieldErrors { 
            errors: Vec<String>,
            field_count: usize 
        } => "Validation failed for {} fields: {}",
        
        SystemValidationFailed { 
            validator: String, 
            error_count: usize 
        } => "System validator '{}' found {} errors",
    }
}

struct FormValidator {
    errors: Vec<String>,
}

impl FormValidator {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }
    
    fn validate_email(&mut self, email: &str) {
        if !email.contains('@') {
            self.errors.push("Email must contain @ symbol".to_string());
        }
        if email.len() < 5 {
            self.errors.push("Email too short".to_string());
        }
    }
    
    fn validate_password(&mut self, password: &str) {
        if password.len() < 8 {
            self.errors.push("Password must be at least 8 characters".to_string());
        }
        if !password.chars().any(|c| c.is_numeric()) {
            self.errors.push("Password must contain at least one number".to_string());
        }
    }
    
    fn validate_username(&mut self, username: &str) {
        if username.len() < 3 {
            self.errors.push("Username must be at least 3 characters".to_string());
        }
        if username.contains(' ') {
            self.errors.push("Username cannot contain spaces".to_string());
        }
    }
    
    fn finish(self) -> Result<(), ValidationErrors> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::MultipleFieldErrors {
                field_count: self.errors.len(),
                errors: self.errors,
            })
        }
    }
}

fn validate_registration_form(form: &RegistrationForm) -> Result<(), ValidationErrors> {
    let mut validator = FormValidator::new();
    
    validator.validate_email(&form.email);
    validator.validate_password(&form.password);
    validator.validate_username(&form.username);
    
    validator.finish()
}
```

## Error Recovery and Handling

### Graceful Degradation

```rust
use logfusion::{define_errors, warn, info};

define_errors! {
    CacheError {
        CacheUnavailable { operation: String } => "Cache unavailable for operation: {}",
        CacheTimeout { key: String, timeout_ms: u64 } => "Cache timeout for key '{}' after {}ms",
        SerializationError { key: String, error: String } => "Failed to serialize data for key '{}': {}",
    }
}

struct UserService {
    database: Database,
    cache: Option<Cache>, // Cache is optional for graceful degradation
}

impl UserService {
    async fn get_user_with_fallback(&self, user_id: u64) -> Result<User, ServiceError> {
        // Try cache first (with graceful degradation)
        if let Some(cache) = &self.cache {
            match cache.get_user(user_id).await {
                Ok(Some(user)) => {
                    info!(
                        user_id = user_id,
                        source = "cache",
                        "User retrieved from cache"
                    );
                    return Ok(user);
                },
                Ok(None) => {
                    // Cache miss - proceed to database
                },
                Err(cache_error) => {
                    warn!(
                        user_id = user_id,
                        error = format!("{}", cache_error),
                        "Cache error, falling back to database"
                    );
                    // Continue to database fallback
                }
            }
        }
        
        // Fallback to database
        match self.database.get_user(user_id).await {
            Ok(Some(user)) => {
                info!(
                    user_id = user_id,
                    source = "database",
                    "User retrieved from database"
                );
                
                // Try to repopulate cache (best effort)
                if let Some(cache) = &self.cache {
                    if let Err(cache_error) = cache.set_user(user_id, &user).await {
                        warn!(
                            user_id = user_id,
                            error = format!("{}", cache_error),
                            "Failed to update cache after database fetch"
                        );
                    }
                }
                
                Ok(user)
            },
            Ok(None) => Err(ServiceError::UserNotFound { user_id }),
            Err(db_error) => Err(ServiceError::DatabaseUnavailable {
                operation: "get_user".to_string(),
                underlying_error: db_error.to_string(),
            }),
        }
    }
}
```

## Best Practices

### Error Message Design

```rust
use logfusion::define_errors;

define_errors! {
    ApiError {
        // ✅ Good: Specific, actionable error messages
        InvalidEmailFormat { 
            email: String 
        } => "Email address '{}' is not valid. Please check the format (example: user@domain.com)",
        
        PasswordTooWeak { 
            requirements: Vec<String> 
        } => "Password does not meet security requirements: {}",
        
        RateLimitExceeded { 
            limit: u32, 
            window_seconds: u32, 
            retry_after: u64 
        } => "Rate limit of {} requests per {} seconds exceeded. Try again in {} seconds",
        
        // ❌ Avoid: Vague error messages
        // GeneralError => "Something went wrong",
        // ProcessingFailed => "Failed to process request",
    }
}
```

### Error Context Best Practices

```rust
use logfusion::{define_errors, error};

define_errors! {
    OrderProcessingError {
        // ✅ Good: Include relevant context for debugging
        InventoryCheckFailed { 
            product_id: u64, 
            product_name: String,
            requested_quantity: u32,
            available_quantity: u32,
            warehouse_id: String 
        } => "Insufficient inventory for product '{}' (ID: {}): requested {}, available {} in warehouse {}",
        
        PaymentValidationFailed { 
            order_id: String,
            user_id: u64,
            payment_method: String,
            validation_errors: Vec<String> 
        } => "Payment validation failed for order {} (user {}): {}",
        
        ShippingCalculationError { 
            order_id: String,
            destination_zip: String,
            total_weight_kg: f64,
            service_error: String 
        } => "Cannot calculate shipping for order {} to {}: total weight {}kg, error: {}",
    }
}
```

### Structured Logging Integration

```rust
use logfusion::{define_errors, error, warn, info};

define_errors! {
    TransactionError {
        InsufficientBalance { 
            account_id: String, 
            requested_amount: f64, 
            available_balance: f64 
        } => "Insufficient balance in account {}: requested ${:.2}, available ${:.2}",
        
        TransactionLimitExceeded { 
            account_id: String, 
            transaction_amount: f64, 
            daily_limit: f64, 
            daily_total: f64 
        } => "Transaction limit exceeded for account {}: ${:.2} transaction would exceed daily limit of ${:.2} (current total: ${:.2})",
    }
}

fn process_transaction(transaction: &Transaction) -> Result<TransactionResult, TransactionError> {
    let account = get_account(&transaction.account_id)?;
    
    // Pre-transaction logging
    info!(
        account_id = transaction.account_id,
        transaction_id = transaction.id,
        amount = transaction.amount,
        transaction_type = transaction.transaction_type,
        "Processing transaction"
    );
    
    // Validation with structured error logging
    if account.balance < transaction.amount {
        // LogFusion automatically logs this error with structured fields
        return Err(TransactionError::InsufficientBalance {
            account_id: transaction.account_id.clone(),
            requested_amount: transaction.amount,
            available_balance: account.balance,
        });
    }
    
    // Check daily limits
    let daily_total = get_daily_transaction_total(&transaction.account_id)?;
    if daily_total + transaction.amount > account.daily_limit {
        return Err(TransactionError::TransactionLimitExceeded {
            account_id: transaction.account_id.clone(),
            transaction_amount: transaction.amount,
            daily_limit: account.daily_limit,
            daily_total,
        });
    }
    
    // Process transaction
    let result = execute_transaction(transaction)?;
    
    // Success logging
    info!(
        account_id = transaction.account_id,
        transaction_id = transaction.id,
        amount = transaction.amount,
        final_balance = result.final_balance,
        "Transaction completed successfully"
    );
    
    Ok(result)
}
```

## Next Steps

Now that you understand error handling with LogFusion, explore these related topics:

- **[Spans and Instrumentation](04-spans-instrumentation.md)** - Add tracing spans for error context
- **[Advanced Tracing Integration](05-advanced-tracing.md)** - Error tracking with OpenTelemetry
- **[FFI Integration](06-ffi-integration.md)** - Error handling across language boundaries

## Troubleshooting

### Common Issues

**Q: My errors aren't being logged automatically**

```rust
// Make sure you're using define_errors! macro and returning the error
define_errors! {
    MyError {
        SomeError { field: String } => "Error: {}",
    }
}

// ✅ This will log automatically when the error is created
return Err(MyError::SomeError { field: "value".to_string() });
```

**Q: Error fields aren't appearing in structured logs**

```rust
// Make sure to use meaningful field names and types
define_errors! {
    MyError {
        // ✅ Good: Structured fields
        ValidationFailed { field_name: String, error_message: String } => "Validation failed for {}: {}",
        
        // ❌ Less useful: No structured context
        ValidationFailed => "Validation failed",
    }
}
```

**Q: Error messages are too generic**

```rust
// Make error messages specific and actionable
define_errors! {
    PaymentError {
        // ✅ Good: Specific, actionable message
        CardDeclined { reason: String, retry_allowed: bool } => "Card declined: {}. Retry allowed: {}",
        
        // ❌ Too generic
        PaymentFailed => "Payment failed",
    }
}
```
