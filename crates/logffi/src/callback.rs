use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "callback")] {
        use std::sync::Mutex;
        
        /// Callback function type for FFI bridges
        pub type Callback = Box<dyn Fn(&str, &str, &str) + Send + Sync>;
        
        /// Global callback storage
        static CALLBACK: Mutex<Option<Callback>> = Mutex::new(None);
        
        /// Set callback for bridging logs to other systems (Python, Node.js, etc.)
        pub fn set(callback: Callback) {
            let mut guard = CALLBACK.lock().unwrap();
            *guard = Some(callback);
        }
        
        /// Internal function to call the callback if set
        pub(crate) fn call(level: &str, target: &str, message: &str) {
            if let Ok(guard) = CALLBACK.lock() {
                if let Some(callback) = guard.as_ref() {
                    callback(level, target, message);
                }
            }
        }
    }
}