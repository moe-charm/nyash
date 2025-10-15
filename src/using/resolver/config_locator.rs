//! Configuration file location logic for using resolver

use std::path::PathBuf;

/// Locate hako.toml/nyash.toml/hakorune.toml configuration file
///
/// Search order:
/// 1. CWD: ./hako.toml > ./nyash.toml > ./hakorune.toml
///    (unless NYASH_USING_TEST_FORCE_ENV_ROOT=1)
/// 2. ENV roots: NYASH_ROOT/HAKO_ROOT/HAKU_ROOT/HRN_ROOT + same filename priority
///
/// Returns Some(path) if found, None otherwise
pub fn locate_config_file() -> Option<PathBuf> {
    let force_env_root = std::env::var("NYASH_USING_TEST_FORCE_ENV_ROOT")
        .ok()
        .as_deref() == Some("1");

    // Prefer CWD hako.toml unless tests force env-root mode
    if !force_env_root {
        for name in ["hako.toml", "nyash.toml", "hakorune.toml"] {
            let p = std::path::Path::new(name);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
    }

    // Try roots: NYASH_ROOT/HAKO_ROOT/HAKU_ROOT/HRN_ROOT
    if let Some(root) = std::env::var("NYASH_ROOT").ok()
        .or_else(|| std::env::var("HAKO_ROOT").ok())
        .or_else(|| std::env::var("HAKU_ROOT").ok())
        .or_else(|| std::env::var("HRN_ROOT").ok())
    {
        for name in ["hako.toml", "nyash.toml", "hakorune.toml"] {
            let p = std::path::Path::new(&root).join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

/// Emit trace information about config file location
pub fn trace_config_location(path: &std::path::Path) {
    if !crate::config::env::resolve_trace() {
        return;
    }

    if path.exists() {
        if !crate::config::env::cli_quiet() {
            eprintln!("[using] toml: using {:?}", std::fs::canonicalize(path).ok());
        }
    }

    // One-line hint about search priority and root envs considered (for field debugging)
    if !crate::config::env::cli_quiet() {
        let root = std::env::var("NYASH_ROOT").ok()
            .or_else(|| std::env::var("HAKO_ROOT").ok())
            .or_else(|| std::env::var("HAKU_ROOT").ok())
            .or_else(|| std::env::var("HRN_ROOT").ok());
        eprintln!(
            "[using] toml search: [./hako.toml > ./nyash.toml > ./hakorune.toml] or roots {:?}",
            root
        );
    }
}
