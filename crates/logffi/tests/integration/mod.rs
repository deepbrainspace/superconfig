mod auto_initialization;
mod define_errors_logffi; // Temporarily disabled - focus on minimal_test first
mod define_errors_thiserror;
mod logffi_structured_logging;
mod logging_macros;
mod thiserror;
mod tracing;

#[cfg(feature = "callback")]
mod callback_functionality;
