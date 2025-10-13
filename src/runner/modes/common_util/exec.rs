use std::path::Path;

use super::io::spawn_with_timeout;

/// Emit MIR JSON and invoke the Python llvmlite harness to produce an object file.
/// - module: lib-side MIR module
/// - out_path: destination object path
/// - timeout_ms: process timeout
#[allow(dead_code)]
pub fn llvmlite_emit_object(
    module: &nyash_rust::mir::MirModule,
    out_path: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(out_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Locate python3 and harness
    let py3 = which::which("python3").map_err(|e| format!("python3 not found: {}", e))?;
    let harness = Path::new("tools/llvmlite_harness.py");
    if !harness.exists() {
        return Err(format!("llvmlite harness not found: {}", harness.display()));
    }
    // Emit MIR(JSON) to tmp
    let tmp_dir = Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let mir_json_path = tmp_dir.join("nyash_harness_mir.json");
    crate::runner::mir_json_emit::emit_mir_json_for_harness(module, &mir_json_path)
        .map_err(|e| format!("MIR JSON emit error: {}", e))?;
    // Export externs registry (abstract spec) for harness (optional)
    let extern_json_path = tmp_dir.join("externs_registry.json");
    let _ = nyash_rust::mir::externs::registry::export_json(&extern_json_path);
    crate::cli_v!(
        "[Runner/LLVM] using llvmlite harness → {} (mir={})",
        out_path,
        mir_json_path.display()
    );
    // Spawn harness
    let mut cmd = std::process::Command::new(py3);
    cmd.args([
        harness.to_string_lossy().as_ref(),
        "--in",
        &mir_json_path.display().to_string(),
        "--out",
        out_path,
    ]);
    // Provide extern spec JSON to harness
    cmd.env("NYASH_EXTERN_SPEC_JSON", extern_json_path);
    // Default-on: sanitize empty PHIs in IR to keep llvmlite happy.
    // Users can explicitly disable with NYASH_LLVM_SANITIZE_EMPTY_PHI=0.
    if std::env::var("NYASH_LLVM_SANITIZE_EMPTY_PHI").ok().map(|v| v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")).unwrap_or(false) == false {
        cmd.env("NYASH_LLVM_SANITIZE_EMPTY_PHI", "1");
    }
    let out = spawn_with_timeout(cmd, timeout_ms).map_err(|e| format!("spawn harness: {}", e))?;
    if out.timed_out || !out.status_ok {
        return Err(format!(
            "llvmlite harness failed (timeout={} code={:?})",
            out.timed_out, out.exit_code
        ));
    }
    // Verify output
    match std::fs::metadata(out_path) {
        Ok(meta) if meta.len() > 0 => {
            crate::cli_v!(
                "[LLVM] object emitted: {} ({} bytes)",
                out_path,
                meta.len()
            );
            Ok(())
        }
        _ => Err(format!("harness output not found or empty: {}", out_path)),
    }
}

/// Resolve ny-llvmc executable path with env/PATH fallbacks
pub fn resolve_ny_llvmc() -> std::path::PathBuf {
    std::env::var("NYASH_NY_LLVM_COMPILER")
        .ok()
        .and_then(|s| if !s.is_empty() { Some(std::path::PathBuf::from(s)) } else { None })
        .or_else(|| which::which("ny-llvmc").ok())
        .unwrap_or_else(|| std::path::PathBuf::from("target/release/ny-llvmc"))
}

fn hint_ny_llvmc_missing(path: &std::path::Path) -> String {
    format!(
        "ny-llvmc not found (tried: {}).\nHints:\n  - Build it: cargo build -p nyash-llvm-compiler --release\n  - Use the built binary: target/release/ny-llvmc\n  - Or set env NYASH_NY_LLVM_COMPILER=/full/path/to/ny-llvmc\n  - Or add it to PATH\n",
        path.display()
    )
}

/// Emit native executable via ny-llvmc (lib-side MIR)
#[allow(dead_code)]
pub fn ny_llvmc_emit_exe_lib(
    module: &nyash_rust::mir::MirModule,
    exe_out: &str,
    nyrt_dir: Option<&str>,
    extra_libs: Option<&str>,
) -> Result<(), String> {
    let tmp_dir = std::path::Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let json_path = tmp_dir.join("nyash_cli_emit.json");
    crate::runner::mir_json_emit::emit_mir_json_for_harness(module, &json_path)
        .map_err(|e| format!("MIR JSON emit error: {}", e))?;

    // Export externs registry (abstract spec) for harness/ny-llvmc
    let extern_json_path = tmp_dir.join("externs_registry.json");
    let _ = nyash_rust::mir::externs::registry::export_json(&extern_json_path);

    let ny_llvmc = resolve_ny_llvmc();
    if !ny_llvmc.exists() {
        return Err(hint_ny_llvmc_missing(&ny_llvmc));
    }
    let mut cmd = std::process::Command::new(ny_llvmc);
    cmd.arg("--in")
        .arg(&json_path)
        .arg("--emit")
        .arg("exe")
        .arg("--out")
        .arg(exe_out);

    // Provide extern spec JSON to ny-llvmc (which passes to harness)
    cmd.env("NYASH_EXTERN_SPEC_JSON", extern_json_path);

    let default_nyrt = std::env::var("NYASH_EMIT_EXE_NYRT")        .ok()        .or_else(|| std::env::var("NYASH_ROOT").ok().map(|r| format!("{}/target/release", r)))        .unwrap_or_else(|| "target/release".to_string());
    if let Some(dir) = nyrt_dir { cmd.arg("--nyrt").arg(dir); } else { cmd.arg("--nyrt").arg(default_nyrt); }
    if let Some(flags) = extra_libs { if !flags.trim().is_empty() { cmd.arg("--libs").arg(flags); } }
    let status = cmd.status().map_err(|e| {
        let prog_path = std::path::Path::new(cmd.get_program());
        format!(
            "failed to spawn ny-llvmc: {}\n{}",
            e,
            hint_ny_llvmc_missing(prog_path)
        )
    })?;
    if !status.success() {
        return Err(format!(
            "ny-llvmc failed with status: {:?}.\nTry adding --emit-exe-libs (e.g. \"-ldl -lpthread -lm\") or set --emit-exe-nyrt to NyRT dir (e.g. target/release).",
            status.code()
        ));
    }
    Ok(())
}

/// Emit native executable via ny-llvmc (bin-side MIR)
#[allow(dead_code)]
pub fn ny_llvmc_emit_exe_bin(
    module: &crate::mir::MirModule,
    exe_out: &str,
    nyrt_dir: Option<&str>,
    extra_libs: Option<&str>,
) -> Result<(), String> {
    let tmp_dir = std::path::Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let json_path = tmp_dir.join("nyash_cli_emit.json");
    crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, &json_path)
        .map_err(|e| format!("MIR JSON emit error: {}", e))?;
    let ny_llvmc = resolve_ny_llvmc();
    if !ny_llvmc.exists() {
        return Err(hint_ny_llvmc_missing(&ny_llvmc));
    }
    let mut cmd = std::process::Command::new(ny_llvmc);
    cmd.arg("--in")
        .arg(&json_path)
        .arg("--emit")
        .arg("exe")
        .arg("--out")
        .arg(exe_out);
    let default_nyrt = std::env::var("NYASH_EMIT_EXE_NYRT")        .ok()        .or_else(|| std::env::var("NYASH_ROOT").ok().map(|r| format!("{}/target/release", r)))        .unwrap_or_else(|| "target/release".to_string());
    if let Some(dir) = nyrt_dir { cmd.arg("--nyrt").arg(dir); } else { cmd.arg("--nyrt").arg(default_nyrt); }
    if let Some(flags) = extra_libs { if !flags.trim().is_empty() { cmd.arg("--libs").arg(flags); } }
    let status = cmd.status().map_err(|e| {
        let prog_path = std::path::Path::new(cmd.get_program());
        format!(
            "failed to spawn ny-llvmc: {}\n{}",
            e,
            hint_ny_llvmc_missing(prog_path)
        )
    })?;
    if !status.success() {
        return Err(format!(
            "ny-llvmc failed with status: {:?}.\nTry adding --emit-exe-libs (e.g. \"-ldl -lpthread -lm\") or set --emit-exe-nyrt to NyRT dir (e.g. target/release).",
            status.code()
        ));
    }
    Ok(())
}

/// Run an executable with arguments and a timeout.
/// Returns (exit_code, timed_out, stdout_text).
#[allow(dead_code)]
pub fn run_executable(
    exe_path: &str,
    args: &[&str],
    timeout_ms: u64,
) -> Result<(i32, bool, String), String> {
    let mut cmd = std::process::Command::new(exe_path);
    for a in args { cmd.arg(a); }
    let out = super::io::spawn_with_timeout(cmd, timeout_ms)
        .map_err(|e| format!("spawn exe: {}", e))?;
    let code = out.exit_code.unwrap_or(1);
    let stdout_text = String::from_utf8_lossy(&out.stdout).into_owned();
    Ok((code, out.timed_out, stdout_text))
}
