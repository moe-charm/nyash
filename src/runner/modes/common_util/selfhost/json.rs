use crate::mir::MirModule;
use std::path::Path;

/// Extract the first JSON v0 line from stdout text.
/// Heuristic: a line starting with '{' and containing keys "version" and "kind".
pub fn first_json_v0_line<S: AsRef<str>>(s: S) -> Option<String> {
    for line in s.as_ref().lines() {
        let t = line.trim();
        if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") {
            return Some(t.to_string());
        }
    }
    None
}

/// Parse a JSON v0 line into MirModule using the existing bridge.
pub fn parse_json_v0_line(line: &str) -> Result<MirModule, String> {
    crate::runner::json_v0_bridge::parse_json_v0_to_module(line)
        .map_err(|e| format!("JSON v0 parse error: {}", e))
}

/// Emit MIR JSON for PyVM and execute the Python runner. Returns exit code on success.
/// Prints a verbose note when `NYASH_CLI_VERBOSE=1`.
pub fn run_pyvm_module(module: &MirModule, label: &str) -> Option<i32> {
    // Resolve python3 and runner path
    let py3 = which::which("python3").ok()?;
    let runner = Path::new("tools/pyvm_runner.py");
    if !runner.exists() {
        return None;
    }
    // Prepare JSON for harness
    let tmp_dir = Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
    if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, &mir_json_path) {
        eprintln!("❌ PyVM MIR JSON emit error: {}", e);
        return None;
    }
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[Bridge] using PyVM ({}) → {}", label, mir_json_path.display());
    }
    // Select entry (prefer Main.main; top-level main only if allowed)
    let allow_top = crate::config::env::entry_allow_toplevel_main();
    let entry = if module.functions.contains_key("Main.main") {
        "Main.main"
    } else if allow_top && module.functions.contains_key("main") {
        "main"
    } else if module.functions.contains_key("main") {
        eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence.");
        "main"
    } else {
        "Main.main"
    };
    let status = std::process::Command::new(py3)
        .args([
            "tools/pyvm_runner.py",
            "--in",
            &mir_json_path.display().to_string(),
            "--entry",
            entry,
        ])
        .status()
        .map_err(|e| format!("spawn pyvm: {}", e))
        .ok()?;
    Some(status.code().unwrap_or(1))
}
