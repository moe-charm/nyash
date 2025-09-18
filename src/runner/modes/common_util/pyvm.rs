// no extra imports needed

/// Run PyVM harness over a MIR module, returning the exit code
#[allow(dead_code)]
pub fn run_pyvm_harness(module: &crate::mir::MirModule, tag: &str) -> Result<i32, String> {
    let py3 = which::which("python3").map_err(|e| format!("python3 not found: {}", e))?;
    let runner = std::path::Path::new("tools/pyvm_runner.py");
    if !runner.exists() {
        return Err(format!("PyVM runner not found: {}", runner.display()));
    }
    let tmp_dir = std::path::Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
    crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, &mir_json_path)
        .map_err(|e| format!("PyVM MIR JSON emit error: {}", e))?;
    crate::cli_v!("[ny-compiler] using PyVM ({} ) → {}", tag, mir_json_path.display());
    // Determine entry function hint (prefer Main.main if present)
    let entry = if module.functions.contains_key("Main.main") {
        "Main.main"
    } else if module.functions.contains_key("main") {
        "main"
    } else {
        "Main.main"
    };
    let status = std::process::Command::new(py3)
        .args([
            runner.to_string_lossy().as_ref(),
            "--in",
            &mir_json_path.display().to_string(),
            "--entry",
            entry,
        ])
        .status()
        .map_err(|e| format!("spawn pyvm: {}", e))?;
    let code = status.code().unwrap_or(1);
    if !status.success() {
        crate::cli_v!("❌ PyVM ({}) failed (status={})", tag, code);
    }
    Ok(code)
}

/// Run PyVM harness over a nyash_rust (lib) MIR module, returning the exit code
#[allow(dead_code)]
pub fn run_pyvm_harness_lib(module: &nyash_rust::mir::MirModule, tag: &str) -> Result<i32, String> {
    let py3 = which::which("python3").map_err(|e| format!("python3 not found: {}", e))?;
    let runner = std::path::Path::new("tools/pyvm_runner.py");
    if !runner.exists() {
        return Err(format!("PyVM runner not found: {}", runner.display()));
    }
    let tmp_dir = std::path::Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
    crate::runner::mir_json_emit::emit_mir_json_for_harness(module, &mir_json_path)
        .map_err(|e| format!("PyVM MIR JSON emit error: {}", e))?;
    crate::cli_v!("[Runner] using PyVM ({} ) → {}", tag, mir_json_path.display());
    // Determine entry function hint (prefer Main.main if present)
    let entry = if module.functions.contains_key("Main.main") {
        "Main.main"
    } else if module.functions.contains_key("main") {
        "main"
    } else {
        "Main.main"
    };
    let status = std::process::Command::new(py3)
        .args([
            runner.to_string_lossy().as_ref(),
            "--in",
            &mir_json_path.display().to_string(),
            "--entry",
            entry,
        ])
        .status()
        .map_err(|e| format!("spawn pyvm: {}", e))?;
    let code = status.code().unwrap_or(1);
    if !status.success() {
        crate::cli_v!("❌ PyVM ({}) failed (status={})", tag, code);
    }
    Ok(code)
}
