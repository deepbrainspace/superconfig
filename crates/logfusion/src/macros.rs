// Enhanced define_errors! macro with structured tracing integration
// This module ONLY contains the define_errors! macro - all logging macros are in tracing.rs
//
// MACRO ORGANIZATION:
// 1. THISERROR COMPATIBILITY - Traditional thiserror enum syntax 
// 2. LOGFUSION SYNTAX - Simplified error definition with attributes
// 3. LOGFUSION INTERNAL PROCESSING - Token parsing and variant collection
// 4. ENUM GENERATION - Pattern matching for different variant types
// 5. MIXED VARIANT PROCESSING - Handling unit + struct variants together  
// 6. LOGGING HELPERS - Shared utilities + LogFusion & thiserror attribute parsing

/// Enhanced `define_errors!` macro with structured tracing integration.
/// 
/// Supports both thiserror-style and simplified LogFusion-style syntax.
/// See examples in the cookbook folder and comprehensive tests for usage patterns.
#[macro_export]
macro_rules! define_errors {

    // ==================================================================================
    // THISERROR COMPATIBILITY SECTION
    // ==================================================================================
    // Traditional thiserror syntax (must come first to avoid conflicts with LogFusion)
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                #[error($msg:literal $(, level = $level:ident)? $(, target = $target:literal)? $(, source)?)]
                $variant:ident $({
                    $(
                        $(#[$field_meta:meta])*
                        $field_name:ident: $field_type:ty
                    ),* $(,)?
                })?,
            )*
        }
    ) => {
        // Generate thiserror Error enum with source chain support
        #[derive(thiserror::Error, Debug)]
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                #[error($msg)]
                $variant $({
                    $(
                        $(#[$field_meta])*
                        $field_name: $field_type
                    ),*
                })?,
            )*
        }

        impl $name {
            /// Automatically log this error with structured tracing (preserves source chain)
            pub fn log(&self) {
                match self {
                    $(
                        Self::$variant { .. } => {
                            let code = self.code();
                            let message = self.to_string();
                            
                            // Use traditional thiserror attribute parsing
                            define_errors!(@log_thiserror $($level)? $($target)? ; code, message);
                        },
                    )*
                }
            }
            
            /// Get error code for API stability
            /// 
            /// Returns a static string identifier for this error variant.
            /// Useful for programmatic error handling and API responses.
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
            
            /// Get structured error information for debugging and metrics
            /// 
            /// Returns a tuple of (code, level, target) for this error variant.
            /// This is useful for error analytics, monitoring, and structured logging.
            /// 
            /// # Returns
            /// - `code`: Static string identifier for the error variant
            /// - `level`: Log level as specified in attributes (defaults to "error")
            /// - `target`: Log target module (defaults to current module)
            pub fn error_info(&self) -> (&'static str, &'static str, &'static str) {
                match self {
                    $(
                        Self::$variant { .. } => {
                            let code = stringify!($variant);
                            // For thiserror format, we extract from thiserror attributes
                            define_errors!(@extract_thiserror_info $($level)? $($target)? ; code)
                        },
                    )*
                }
            }
        }
    };

    // ==================================================================================
    // LOGFUSION SYNTAX SECTION  
    // ==================================================================================

    // Multiple error types in one macro call (must come before single type)
    (
        $first_name:ident {
            $($first_tokens:tt)*
        }
        $($rest_name:ident {
            $($rest_tokens:tt)*
        })+
    ) => {
        // Process the first error type
        define_errors! {
            $first_name {
                $($first_tokens)*
            }
        }
        
        // Process the remaining error types
        define_errors! {
            $($rest_name {
                $($rest_tokens)*
            })+
        }
    };

    // Single error type (mixed variants with mandatory braces)
    (
        $name:ident {
            $($tokens:tt)*
        }
    ) => {
        // Collect all the variant information first
        define_errors!(@collect 
            name: $name,
            variants: [],
            tokens: [$($tokens)*]
        );
    };
    
    // ==================================================================================
    // LOGFUSION INTERNAL PROCESSING PATTERNS
    // ==================================================================================
    
    // Parse LogFusion variant syntax: VariantName { fields... } : "message" [attributes]
    (@collect
        name: $name:ident,
        variants: [$($variants:tt)*],
        tokens: [
            $variant:ident { $($field_name:ident : $field_type:ty),* $(,)? } : $msg:literal $([$($attr:tt)*])? 
            $(, $($rest:tt)*)?
        ]
    ) => {
        define_errors!(@collect
            name: $name,
            variants: [$($variants)*
                ($variant, $msg, ($($field_name : $field_type),*), $([$($attr)*])?)
            ],
            tokens: [$($($rest)*)?]
        );
    };
    
    // All variants collected - dispatch to appropriate enum generator
    (@collect
        name: $name:ident,
        variants: [$($variants:tt)*],
        tokens: []
    ) => {
        define_errors!(@build $name; $($variants)*);
    };
    
    // -----------------------------------------------------------------------------------
    // ENUM GENERATION PATTERNS
    // -----------------------------------------------------------------------------------
    
    // Build the final enum - handle empty and non-empty field cases separately
    (@build $name:ident; $(($variant:ident, $msg:literal, (), $([$($attr:tt)*])?))*) => {
        // All unit variants (no fields)
        #[derive(thiserror::Error, Debug)]
        pub enum $name {
            $(
                #[error($msg)]
                $variant,
            )*
        }

        impl $name {
            pub fn log(&self) {
                match self {
                    $(
                        Self::$variant => {
                            let code = self.code();
                            let message = self.to_string();
                            define_errors!(@log_simple $([$($attr)*])? ; code, message);
                        },
                    )*
                }
            }
            
            /// Get error code for API stability
            /// 
            /// Returns a static string identifier for this error variant.
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => stringify!($variant),
                    )*
                }
            }
            
            /// Get structured error information for debugging and metrics
            /// 
            /// Returns a tuple of (code, level, target) for this error variant.
            /// 
            /// # Returns
            /// - `code`: Static string identifier for the error variant
            /// - `level`: Log level as specified in attributes (defaults to "error")
            /// - `target`: Log target module (defaults to current module)
            pub fn error_info(&self) -> (&'static str, &'static str, &'static str) {
                match self {
                    $(
                        Self::$variant => {
                            let code = stringify!($variant);
                            define_errors!(@extract_info $([$($attr)*])? ; code)
                        },
                    )*
                }
            }
        }
    };

    (@build $name:ident; $(($variant:ident, $msg:literal, ($($field_name:ident : $field_type:ty),+), $([$($attr:tt)*])?))*) => {
        // All struct variants (with fields)
        #[derive(thiserror::Error, Debug)]
        pub enum $name {
            $(
                #[error($msg)]
                $variant {
                    $($field_name : $field_type),+
                },
            )*
        }

        impl $name {
            pub fn log(&self) {
                match self {
                    $(
                        Self::$variant { .. } => {
                            let code = self.code();
                            let message = self.to_string();
                            define_errors!(@log_simple $([$($attr)*])? ; code, message);
                        },
                    )*
                }
            }
            
            /// Get error code for API stability
            /// 
            /// Returns a static string identifier for this error variant.
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
            
            /// Get structured error information for debugging and metrics
            /// 
            /// Returns a tuple of (code, level, target) for this error variant.
            /// 
            /// # Returns
            /// - `code`: Static string identifier for the error variant
            /// - `level`: Log level as specified in attributes (defaults to "error")
            /// - `target`: Log target module (defaults to current module)
            pub fn error_info(&self) -> (&'static str, &'static str, &'static str) {
                match self {
                    $(
                        Self::$variant { .. } => {
                            let code = stringify!($variant);
                            define_errors!(@extract_info $([$($attr)*])? ; code)
                        },
                    )*
                }
            }
        }
    };

    // -----------------------------------------------------------------------------------
    // MIXED VARIANT PROCESSING (Unit + Struct variants in same enum)
    // -----------------------------------------------------------------------------------
    
    // Mixed variants (some unit, some struct) - requires separation and special handling
    (@build $name:ident; $(($variant:ident, $msg:literal, ($($field_name:ident : $field_type:ty),*), $([$($attr:tt)*])?))*) => {
        // For truly mixed variants, we need to pre-process to separate unit from struct
        define_errors!(@separate_mixed $name; 
            unit_variants: [];
            struct_variants: [];
            remaining: [$(($variant, $msg, ($($field_name : $field_type),*), $([$($attr)*])?))*]
        );
    };

    // Sort variants into unit (no fields) and struct (with fields) categories
    (@separate_mixed $name:ident;
        unit_variants: [$($unit_processed:tt)*];
        struct_variants: [$($struct_processed:tt)*];
        remaining: [($variant:ident, $msg:literal, (), $([$($attr:tt)*])?) $($rest:tt)*]
    ) => {
        // Empty fields () = unit variant
        define_errors!(@separate_mixed $name;
            unit_variants: [$($unit_processed)* ($variant, $msg, $([$($attr)*])?)];
            struct_variants: [$($struct_processed)*];
            remaining: [$($rest)*]
        );
    };

    (@separate_mixed $name:ident;
        unit_variants: [$($unit_processed:tt)*];
        struct_variants: [$($struct_processed:tt)*];
        remaining: [($variant:ident, $msg:literal, ($($field_name:ident : $field_type:ty),+), $([$($attr:tt)*])?) $($rest:tt)*]
    ) => {
        // Has fields = struct variant
        define_errors!(@separate_mixed $name;
            unit_variants: [$($unit_processed)*];
            struct_variants: [$($struct_processed)* ($variant, $msg, ($($field_name : $field_type),+), $([$($attr)*])?)];
            remaining: [$($rest)*]
        );
    };

    // Generate final enum with both unit and struct variants
    (@separate_mixed $name:ident;
        unit_variants: [$(($unit_variant:ident, $unit_msg:literal, $([$($unit_attr:tt)*])?))*];
        struct_variants: [$(($struct_variant:ident, $struct_msg:literal, ($($struct_field_name:ident : $struct_field_type:ty),+), $([$($struct_attr:tt)*])?))*];
        remaining: []
    ) => {
        #[derive(thiserror::Error, Debug)]
        pub enum $name {
            $(
                #[error($unit_msg)]
                $unit_variant,
            )*
            $(
                #[error($struct_msg)]
                $struct_variant {
                    $($struct_field_name : $struct_field_type),+
                },
            )*
        }

        impl $name {
            pub fn log(&self) {
                match self {
                    $(
                        Self::$unit_variant => {
                            let code = self.code();
                            let message = self.to_string();
                            define_errors!(@log_simple $([$($unit_attr)*])? ; code, message);
                        },
                    )*
                    $(
                        Self::$struct_variant { .. } => {
                            let code = self.code();
                            let message = self.to_string();
                            define_errors!(@log_simple $([$($struct_attr)*])? ; code, message);
                        },
                    )*
                }
            }
            
            /// Get error code for API stability
            /// 
            /// Returns a static string identifier for this error variant.
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$unit_variant => stringify!($unit_variant),
                    )*
                    $(
                        Self::$struct_variant { .. } => stringify!($struct_variant),
                    )*
                }
            }
            
            /// Get structured error information for debugging and metrics
            /// 
            /// Returns a tuple of (code, level, target) for this error variant.
            /// 
            /// # Returns
            /// - `code`: Static string identifier for the error variant
            /// - `level`: Log level as specified in attributes (defaults to "error")
            /// - `target`: Log target module (defaults to current module)
            pub fn error_info(&self) -> (&'static str, &'static str, &'static str) {
                match self {
                    $(
                        Self::$unit_variant => {
                            let code = stringify!($unit_variant);
                            define_errors!(@extract_info $([$($unit_attr)*])? ; code)
                        },
                    )*
                    $(
                        Self::$struct_variant { .. } => {
                            let code = stringify!($struct_variant);
                            define_errors!(@extract_info $([$($struct_attr)*])? ; code)
                        },
                    )*
                }
            }
        }
    };

    // ==================================================================================
    // LOGGING HELPER PATTERNS
    // ==================================================================================
    
    // -----------------------------------------------------------------------------------
    // SHARED LOGGING UTILITIES (used by both thiserror and LogFusion)
    // -----------------------------------------------------------------------------------
    
    // Simple logging dispatcher - routes to appropriate attribute parser
    (@log_simple [$($attr:tt)*] ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs $($attr)* ; $code, $message);
    };
    
    (@log_simple ; $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };

    // -----------------------------------------------------------------------------------
    // LOGFUSION ATTRIBUTE PARSING (level = X, target = Y syntax)
    // -----------------------------------------------------------------------------------
    (@log_with_attrs level = error, target = $target:literal ; $code:expr, $message:expr) => {
        $crate::error!(target: $target, "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = warn, target = $target:literal ; $code:expr, $message:expr) => {
        $crate::warn!(target: $target, "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = info, target = $target:literal ; $code:expr, $message:expr) => {
        $crate::info!(target: $target, "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = debug, target = $target:literal ; $code:expr, $message:expr) => {
        $crate::debug!(target: $target, "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = trace, target = $target:literal ; $code:expr, $message:expr) => {
        $crate::trace!(target: $target, "[{}] {}", $code, $message);
    };
    
    // Log level only (default target)
    (@log_with_attrs level = error ; $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = warn ; $code:expr, $message:expr) => {
        $crate::warn!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = info ; $code:expr, $message:expr) => {
        $crate::info!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = debug ; $code:expr, $message:expr) => {
        $crate::debug!(target: module_path!(), "[{}] {}", $code, $message);
    };
    (@log_with_attrs level = trace ; $code:expr, $message:expr) => {
        $crate::trace!(target: module_path!(), "[{}] {}", $code, $message);
    };
    
    // Target only (default level = error)  
    (@log_with_attrs target = $target:literal ; $code:expr, $message:expr) => {
        $crate::error!(target: $target, "[{}] {}", $code, $message);
    };
    
    // Neither level nor target (both defaults)
    (@log_with_attrs ; $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };
    
    // -----------------------------------------------------------------------------------
    // THISERROR ATTRIBUTE PARSING (compatibility layer)
    // -----------------------------------------------------------------------------------
    // These delegate to @log_with_attrs but handle thiserror's different syntax
    (@log_thiserror $level:ident $target:literal ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs level = $level, target = $target ; $code, $message);
    };
    (@log_thiserror $level:ident ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs level = $level ; $code, $message);
    };
    (@log_thiserror $target:literal ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs target = $target ; $code, $message);
    };
    (@log_thiserror ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs ; $code, $message);
    };

    // -----------------------------------------------------------------------------------
    // STRUCTURED ERROR INFO EXTRACTION
    // -----------------------------------------------------------------------------------
    // For thiserror format - different attribute parsing
    (@extract_thiserror_info $level:ident $target:literal ; $code:expr) => {
        ($code, stringify!($level), $target)
    };
    (@extract_thiserror_info $level:ident ; $code:expr) => {
        ($code, stringify!($level), module_path!())
    };
    (@extract_thiserror_info $target:literal ; $code:expr) => {
        ($code, "error", $target)
    };
    (@extract_thiserror_info ; $code:expr) => {
        ($code, "error", module_path!())
    };
    
    // For LogFusion format - bracket-based attributes
    (@extract_info [level = $level:ident, target = $target:literal] ; $code:expr) => {
        ($code, stringify!($level), $target)
    };
    (@extract_info [target = $target:literal, level = $level:ident] ; $code:expr) => {
        ($code, stringify!($level), $target)
    };
    (@extract_info [level = $level:ident] ; $code:expr) => {
        ($code, stringify!($level), module_path!())
    };
    (@extract_info [target = $target:literal] ; $code:expr) => {
        ($code, "error", $target)
    };
    (@extract_info ; $code:expr) => {
        ($code, "error", module_path!())
    };

}