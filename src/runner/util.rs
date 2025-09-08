//! Runner utilities: using-target resolution and task runner helpers

use std::path::PathBuf;
use std::{fs, process};

/// Resolve a using target according to priority: modules > relative > using-paths
/// Returns Ok(resolved_path_or_token). On strict mode, ambiguous matches cause error.
pub(crate) fn resolve_using_target(
    tgt: &str,
    is_path: bool,
    modules: &[(String, String)],
    using_paths: &[String],
    context_dir: Option<&std::path::Path>,
    strict: bool,
    verbose: bool,
) -> Result<String, String> {
    if is_path { return Ok(tgt.to_string()); }
    // 1) modules mapping
    if let Some((_, p)) = modules.iter().find(|(n, _)| n == tgt) { return Ok(p.clone()); }
    // 2) build candidate list: relative then using-paths
    let rel = tgt.replace('.', "/") + ".nyash";
    let mut cand: Vec<String> = Vec::new();
    if let Some(dir) = context_dir { let c = dir.join(&rel); if c.exists() { cand.push(c.to_string_lossy().to_string()); } }
    for base in using_paths {
        let c = std::path::Path::new(base).join(&rel);
        if c.exists() { cand.push(c.to_string_lossy().to_string()); }
    }
    if cand.is_empty() {
        if verbose { eprintln!("[using] unresolved '{}' (searched: rel+paths)", tgt); }
        return Ok(tgt.to_string());
    }
    if cand.len() > 1 && strict {
        return Err(format!("ambiguous using '{}': {}", tgt, cand.join(", ")));
    }
    Ok(cand.remove(0))
}

/// Minimal task runner: read nyash.toml [env] and [tasks], run the named task via shell
pub(crate) fn run_named_task(name: &str) -> Result<(), String> {
    let cfg_path = "nyash.toml";
    let text = fs::read_to_string(cfg_path).map_err(|e| format!("read {}: {}", cfg_path, e))?;
    let doc = toml::from_str::<toml::Value>(&text).map_err(|e| format!("parse {}: {}", cfg_path, e))?;
    // Apply [env]
    if let Some(env_tbl) = doc.get("env").and_then(|v| v.as_table()) {
        for (k, v) in env_tbl.iter() {
            if let Some(s) = v.as_str() { std::env::set_var(k, s); }
        }
    }
    // Lookup [tasks]
    let tasks = doc.get("tasks").and_then(|v| v.as_table()).ok_or("[tasks] not found in nyash.toml")?;
    let cmd = tasks.get(name).and_then(|v| v.as_str()).ok_or_else(|| format!("task '{}' not found", name))?;
    // Basic variable substitution
    let root = std::env::current_dir().unwrap_or(PathBuf::from(".")).display().to_string();
    let cmd = cmd.replace("{root}", &root);
    // Run via shell
    #[cfg(windows)]
    let status = std::process::Command::new("cmd").args(["/C", &cmd]).status().map_err(|e| e.to_string())?;
    #[cfg(not(windows))]
    let status = std::process::Command::new("sh").arg("-lc").arg(&cmd).status().map_err(|e| e.to_string())?;
    if !status.success() { return Err(format!("task '{}' failed with status {:?}", name, status.code())); }
    Ok(())
}

