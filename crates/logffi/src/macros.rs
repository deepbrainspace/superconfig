// Enhanced define_errors! macro with structured tracing integration
// This module ONLY contains the define_errors! macro - all logging macros are in tracing.rs

/// Enhanced `define_errors!` macro with structured tracing integration.
/// 
/// Supports both thiserror-style and simplified LogFFI-style syntax.
/// See examples in the cookbook folder and comprehensive tests for usage patterns.
#[macro_export]
macro_rules! define_errors {
    // Traditional thiserror syntax (must come first to avoid conflicts)
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
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
        }
    };

    // LogFFI syntax - multiple error types in one macro call (must come before single type)
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

    // LogFFI syntax - single error type (mixed variants with mandatory braces)
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
    
    // Collect variant - handle both empty braces and braces with fields
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
    
    // Done collecting - now generate
    (@collect
        name: $name:ident,
        variants: [$($variants:tt)*],
        tokens: []
    ) => {
        define_errors!(@build $name; $($variants)*);
    };
    
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
            
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => stringify!($variant),
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
            
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => stringify!($variant),
                    )*
                }
            }
        }
    };

    // Mixed case - this is the tricky one - we need a different approach
    (@build $name:ident; $(($variant:ident, $msg:literal, ($($field_name:ident : $field_type:ty),*), $([$($attr:tt)*])?))*) => {
        // For truly mixed variants, we need to pre-process to separate unit from struct
        define_errors!(@separate_mixed $name; 
            unit_variants: [];
            struct_variants: [];
            remaining: [$(($variant, $msg, ($($field_name : $field_type),*), $([$($attr)*])?))*]
        );
    };

    // Separate mixed variants into unit and struct categories
    (@separate_mixed $name:ident;
        unit_variants: [$($unit_processed:tt)*];
        struct_variants: [$($struct_processed:tt)*];
        remaining: [($variant:ident, $msg:literal, (), $([$($attr:tt)*])?) $($rest:tt)*]
    ) => {
        // This is a unit variant
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
        // This is a struct variant
        define_errors!(@separate_mixed $name;
            unit_variants: [$($unit_processed)*];
            struct_variants: [$($struct_processed)* ($variant, $msg, ($($field_name : $field_type),+), $([$($attr)*])?)];
            remaining: [$($rest)*]
        );
    };

    // Generate the mixed enum once separation is complete
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
        }
    };

    // Helper: Simple logging with attributes  
    (@log_simple [$($attr:tt)*] ; $code:expr, $message:expr) => {
        define_errors!(@log_with_attrs $($attr)* ; $code, $message);
    };
    
    (@log_simple ; $code:expr, $message:expr) => {
        $crate::error!(target: module_path!(), "[{}] {}", $code, $message);
    };

    
    // Helper: Parse LogFFI attributes and log accordingly
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
    
    // Thiserror compatibility - different syntax, same functionality as @log_with_attrs
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

}