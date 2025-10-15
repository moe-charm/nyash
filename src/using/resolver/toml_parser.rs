//! TOML configuration file parsing for using resolver

use crate::using::errors::UsingError;
use std::path::Path;

/// Load and parse TOML configuration file
///
/// Returns parsed toml::Value on success, or error if file read or parse fails
/// Emits trace information when NYASH_RESOLVE_TRACE=1
pub fn load_and_parse_toml(path: &Path) -> Result<toml::Value, UsingError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| UsingError::ReadToml(e.to_string()))?;

    let doc = match toml::from_str::<toml::Value>(&text) {
        Ok(v) => v,
        Err(e) => {
            // When tracing is enabled, surface TOML parse issues to help diagnose
            // missing [using]/[modules] during development.
            if crate::config::env::resolve_trace() && !crate::config::env::cli_quiet() {
                eprintln!(
                    "[using] toml parse error at {:?}: {}",
                    std::fs::canonicalize(path).ok(),
                    e
                );
            }
            return Err(UsingError::ReadToml(e.to_string()));
        }
    };

    // One-line summary (trace). Detailed dumps only when NYASH_RESOLVE_DEBUG=1
    if crate::config::env::resolve_trace() {
        emit_toml_summary(&doc, path);
    }

    Ok(doc)
}

/// Emit one-line summary of TOML contents for tracing
fn emit_toml_summary(doc: &toml::Value, path: &Path) {
    let mut mods_count = 0usize;
    if let Some(mods) = doc.get("modules").and_then(|v| v.as_table()) {
        mods_count = mods.len();
    }

    let using_tbl = doc.get("using").and_then(|v| v.as_table());
    let paths_count = using_tbl
        .and_then(|t| t.get("paths").and_then(|v| v.as_array()))
        .map(|a| a.len())
        .unwrap_or(0);
    let aliases_count = using_tbl
        .and_then(|t| t.get("aliases").and_then(|v| v.as_table()))
        .map(|t| t.len())
        .unwrap_or(0);
    let pkgs_count = using_tbl
        .map(|t| t.len())
        .unwrap_or(0)
        .saturating_sub(2 /*paths+aliases*/);

    if !crate::config::env::cli_quiet() {
        eprintln!(
            "[using] loaded: {:?}; modules:{} paths:{} aliases:{} packages:{}",
            std::fs::canonicalize(path).ok(),
            mods_count,
            paths_count,
            aliases_count,
            pkgs_count
        );
    }

    if std::env::var("NYASH_RESOLVE_DEBUG").ok().as_deref() == Some("1") {
        if let Some(tbl) = doc.as_table() {
            let keys: Vec<_> = tbl.keys().cloned().collect();
            if !crate::config::env::cli_quiet() {
                eprintln!("[using] toml: root keys = {:?}", keys);
            }
        }
    }
}
