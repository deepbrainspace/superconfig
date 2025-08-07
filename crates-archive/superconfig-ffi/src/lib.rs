//! SuperConfig FFI Wrapper
//!
//! This crate provides FFI-compatible interfaces for SuperConfig, enabling use from
//! Python and Node.js environments through direct PyO3 and NAPI bindings.
//!
//! # Features
//!
//! - `python`: Enable Python bindings via PyO3
//! - `nodejs`: Enable Node.js bindings via NAPI-RS
//! - `all`: Enable all language targets
//!
//! # Example Usage
//!
//! ```rust
//! use superconfig_ffi::SuperConfig;
//!
//! // Create new configuration instance
//! let config = SuperConfig::new();
//! ```

use superconfig::SuperConfig as CoreSuperConfig;

// Compile-time check to prevent multiple FFI features
#[cfg(all(feature = "python", any(feature = "nodejs", feature = "wasm")))]
compile_error!(
    "Only one FFI feature can be enabled at a time. Choose one of: python, nodejs, wasm"
);

#[cfg(all(feature = "nodejs", feature = "wasm"))]
compile_error!(
    "Only one FFI feature can be enabled at a time. Choose one of: python, nodejs, wasm"
);

#[cfg(all(feature = "python", not(feature = "nodejs"), not(feature = "wasm")))]
use pyo3::types::PyModuleMethods;

#[cfg(feature = "nodejs")]
use napi_derive::napi;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// FFI-compatible wrapper around SuperConfig
///
/// This struct wraps the core SuperConfig implementation and provides
/// FFI-compatible methods for multiple target languages.
#[derive(Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass(unsendable))]
#[cfg_attr(feature = "nodejs", napi)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct SuperConfig {
    inner: CoreSuperConfig,
}

// Python bindings implementation
#[cfg(all(feature = "python", not(feature = "nodejs"), not(feature = "wasm")))]
#[pyo3::pymethods]
impl SuperConfig {
    /// Create a new SuperConfig instance
    #[new]
    fn new() -> Self {
        Self {
            inner: CoreSuperConfig::new(),
        }
    }

    /// Enable debug verbosity level
    fn with_debug_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_debug_verbosity(),
        }
    }

    /// Enable trace verbosity level
    fn with_trace_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_trace_verbosity(),
        }
    }

    /// Get the current verbosity level as a string
    fn get_verbosity(&self) -> String {
        format!("{:?}", self.inner.verbosity())
    }
}

// Node.js bindings implementation
#[cfg(all(feature = "nodejs", not(feature = "python"), not(feature = "wasm")))]
#[napi]
impl SuperConfig {
    /// Create a new SuperConfig instance
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreSuperConfig::new(),
        }
    }

    /// Enable debug verbosity level
    #[napi]
    pub fn with_debug_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_debug_verbosity(),
        }
    }

    /// Enable trace verbosity level
    #[napi]
    pub fn with_trace_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_trace_verbosity(),
        }
    }

    /// Get the current verbosity level as a string
    #[napi]
    pub fn get_verbosity(&self) -> String {
        format!("{:?}", self.inner.verbosity())
    }
}

// WebAssembly bindings implementation
#[cfg(all(feature = "wasm", not(feature = "python"), not(feature = "nodejs")))]
#[wasm_bindgen]
impl SuperConfig {
    /// Create a new SuperConfig instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreSuperConfig::new(),
        }
    }

    /// Enable debug verbosity level
    #[wasm_bindgen]
    pub fn with_debug_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_debug_verbosity(),
        }
    }

    /// Enable trace verbosity level
    #[wasm_bindgen]
    pub fn with_trace_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_trace_verbosity(),
        }
    }

    /// Get the current verbosity level as a string
    #[wasm_bindgen]
    pub fn get_verbosity(&self) -> String {
        format!("{:?}", self.inner.verbosity())
    }
}

// Default implementation (no FFI features enabled)
#[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
impl Default for SuperConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(any(feature = "python", feature = "nodejs", feature = "wasm")))]
impl SuperConfig {
    /// Create a new SuperConfig instance
    pub fn new() -> Self {
        Self {
            inner: CoreSuperConfig::new(),
        }
    }

    /// Enable debug verbosity level
    pub fn with_debug_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_debug_verbosity(),
        }
    }

    /// Enable trace verbosity level
    pub fn with_trace_verbosity(&self) -> Self {
        Self {
            inner: self.inner.clone().with_trace_verbosity(),
        }
    }

    /// Get the current verbosity level as a string
    pub fn get_verbosity(&self) -> String {
        format!("{:?}", self.inner.verbosity())
    }
}

// Python module initialization
#[cfg(all(feature = "python", not(feature = "nodejs"), not(feature = "wasm")))]
#[pyo3::pymodule]
fn superconfig_ffi(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    m.add_class::<SuperConfig>()?;
    Ok(())
}
