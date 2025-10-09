use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Check if plugin debug mode is enabled via environment variables.
/// Returns true if NYASH_DEBUG_PLUGIN=1 OR PLUGIN_DEBUG is set.
pub(crate) fn dbg_on() -> bool {
    std::env::var("NYASH_DEBUG_PLUGIN").unwrap_or_default() == "1"
        || std::env::var("PLUGIN_DEBUG").is_ok()
}

static DBG_ONCE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Print a debug line only once per process when NYASH_DEBUG_PLUGIN=1.
pub(super) fn dbg_once(key: &str, msg: &str) {
    if !dbg_on() {
        return;
    }
    let set = DBG_ONCE.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut g) = set.lock() {
        if g.insert(key.to_string()) {
            eprintln!("{}", msg);
        }
    }
}
