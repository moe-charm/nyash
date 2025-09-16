//! Tiny env-gated logging helpers (quiet by default)

/// Returns true if the given env var is set to "1".
pub fn on(var: &str) -> bool {
    std::env::var(var).ok().as_deref() == Some("1")
}

/// Log a message to stderr if the env var is enabled.
pub fn log(var: &str, msg: &str) {
    if on(var) {
        eprintln!("{}", msg);
    }
}

/// Log with formatting if the env var is enabled.
#[macro_export]
macro_rules! debug_logf {
    ($var:expr, $($arg:tt)*) => {{
        if std::env::var($var).ok().as_deref() == Some("1") {
            eprintln!($($arg)*);
        }
    }};
}
