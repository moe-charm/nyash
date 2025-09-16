//! Interpreter utilities: debug toggles

/// Global debug toggle for interpreter layer (NYASH_DEBUG=1)
pub fn debug_on() -> bool {
    std::env::var("NYASH_DEBUG").unwrap_or_default() == "1"
}
