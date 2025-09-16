//! Runner tracing helpers (verbose-guarded)

/// Return whether CLI verbose logging is enabled
pub fn cli_verbose() -> bool { crate::config::env::cli_verbose() }

#[macro_export]
macro_rules! cli_v {
    ($($arg:tt)*) => {{
        if crate::config::env::cli_verbose() { eprintln!($($arg)*); }
    }};
}

/// Unstructured trace output function used by pipeline helpers
pub fn log<S: AsRef<str>>(msg: S) { eprintln!("{}", msg.as_ref()); }
