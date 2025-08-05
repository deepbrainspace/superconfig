# Error Handling with LogFFI and Tracing

This guide demonstrates how to combine LogFFI's `define_errors!` macro with tracing's structured logging for comprehensive error handling. LogFFI provides automatic error logging capabilities, while tracing handles the underlying structured logging implementation.

**What LogFFI provides:** The `define_errors!` macro for automatic error logging, convenient macros, and re-exported tracing-subscriber components.

**What tracing provides:** The structured logging implementation, error context handling, and integration with observability platforms.

## Table of Contents

- [Error Handling Overview](#error-handling-overview)
- [The define_errors! Macro](#the-define_errors-macro)
- [Basic Error Types](#basic-error-types)
- [Structured Error Logging](#structured-error-logging)
- [Error Context and Chaining](#error-context-and-chaining)
- [Integration with Result Types](#integration-with-result-types)
- [Advanced Error Patterns](#advanced-error-patterns)
- [Error Recovery and Handling](#error-recovery-and-handling)
- [Best Practices](#best-practices)

## Error Handling Overview

LogFFI's error handling combines automatic logging with structured error information:

```rust
use logffi::{define_errors, error, info};

// Define errors with automatic logging - LogFFI syntax
define_errors! {
    UserError {
        NotFound { user_id: u64 } : "User {} not found",
        InvalidCredentials { username: String } : "Invalid credentials for user: {}",
        AccountLocked { user_id: u64, locked_until: String } : "Account {} locked until {}",
    }
}

fn authenticate_user(username: &str, password: &str) -> Result<User, UserError> {
    let user = find_user(username)?;
    
    if user.is_locked() {
        // Error automatically logged with structured fields
        return Err(UserError::AccountLocked {
            user_id: user.id,
            locked_until: user.locked_until.to_string(),
        });
    }
    
    if !verify_password(password, &user.password_hash) {
        // Structured error logging happens automatically
        return Err(UserError::InvalidCredentials {
            username: username.to_string(),
        });
    }
    
    info!(
        user_id = user.id,
        username = username,
        "User authentication successful"
    );
    
    Ok(user)
}
```

## The define_errors! Macro

LogFFI's `define_errors!` macro creates error types with automatic structured logging:

### Basic Syntax

```rust
use logffi::define_errors;

define_errors! {
    ErrorTypeName {
        VariantName { field1: Type1, field2: Type2 } : "Format string with {} and {}",
        AnotherVariant { field: Type } : "Another message with {}",
        SimpleVariant {} : "Simple error message",
    }
}
```

### Error Definition Examples

```rust
use logffi::define_errors;

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
use logffi::{define_errors, warn};

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
use logffi::define_errors;

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

LogFFI errors automatically log with structured fields when created:

```rust
use logffi::{define_errors, info, warn};

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

Combine LogFFI errors with external error types and context:

```rust
use logffi::{define_errors, error};

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

Use LogFFI errors in comprehensive error handling patterns:

```rust
use logffi::{define_errors, info, warn, error};

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
use logffi::{define_errors, warn, info};
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
use logffi::{define_errors, error, warn};

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
use logffi::{define_errors, warn, info};

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
use logffi::define_errors;

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
use logffi::{define_errors, error};

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
use logffi::{define_errors, error, warn, info};

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
        // LogFFI automatically logs this error with structured fields
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

Now that you understand error handling with LogFFI, explore these related topics:

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
