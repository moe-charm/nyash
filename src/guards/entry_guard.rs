//! EntryGuardBox — new() guard for flow/static boxes
//!
//! Provides a single place to decide whether `new <Class>()` should be forbidden.
//! Currently conservative: blocks `new Main()` when `NYASH_ENABLE_FLOW=1`.

pub fn forbid_new(class_name: &str) -> Option<String> {
    if std::env::var("NYASH_ENABLE_FLOW").ok().as_deref() == Some("1") {
        if class_name == "Main" {
            return Some("Cannot instantiate static/flow box with `new`".into());
        }
    }
    None
}
