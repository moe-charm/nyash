use std::path::PathBuf;

/// Minimal task runner: read nyash.toml [env] and [tasks], run the named task via shell
pub(super) fn run_named_task(name: &str) -> Result<(), String> {
    let cfg_path = "nyash.toml";
    let text =
        std::fs::read_to_string(cfg_path).map_err(|e| format!("read {}: {}", cfg_path, e))?;
    let doc =
        toml::from_str::<toml::Value>(&text).map_err(|e| format!("parse {}: {}", cfg_path, e))?;
    // Apply [env]
    if let Some(env_tbl) = doc.get("env").and_then(|v| v.as_table()) {
        for (k, v) in env_tbl.iter() {
            if let Some(s) = v.as_str() {
                std::env::set_var(k, s);
            }
        }
    }
    // Lookup [tasks]
    let tasks = doc
        .get("tasks")
        .and_then(|v| v.as_table())
        .ok_or("[tasks] not found in nyash.toml")?;
    let cmd = tasks
        .get(name)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("task '{}' not found", name))?;
    // Basic variable substitution
    let root = std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .display()
        .to_string();
    let cmd = cmd.replace("{root}", &root);
    // Run via shell
    #[cfg(windows)]
    let status = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .status()
        .map_err(|e| e.to_string())?;
    #[cfg(not(windows))]
    let status = std::process::Command::new("sh")
        .arg("-lc")
        .arg(&cmd)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "task '{}' failed with status {:?}",
            name,
            status.code()
        ));
    }
    Ok(())
}
