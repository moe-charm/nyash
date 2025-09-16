/*!
 * Runner trace utilities — centralized verbose logging
 */

/// Returns true when runner-level verbose tracing is enabled.
/// Controlled by `NYASH_CLI_VERBOSE=1` or `NYASH_RESOLVE_TRACE=1`.
pub fn enabled() -> bool {
    std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1")
        || std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1")
}

/// Emit a single-line trace message when enabled.
pub fn log<S: AsRef<str>>(msg: S) {
    if enabled() {
        eprintln!("{}", msg.as_ref());
    }
}

