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
fn resolve_ny_llvmc() -> std::path::PathBuf {
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
    if let Some(dir) = nyrt_dir { cmd.arg("--nyrt").arg(dir); } else { cmd.arg("--nyrt").arg("target/release"); }
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
    if let Some(dir) = nyrt_dir { cmd.arg("--nyrt").arg(dir); } else { cmd.arg("--nyrt").arg("target/release"); }
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

/// Run an executable with arguments and a timeout. Returns (exit_code, timed_out).
#[allow(dead_code)]
pub fn run_executable(exe_path: &str, args: &[&str], timeout_ms: u64) -> Result<(i32, bool), String> {
    let mut cmd = std::process::Command::new(exe_path);
    for a in args { cmd.arg(a); }
    let out = super::io::spawn_with_timeout(cmd, timeout_ms)
        .map_err(|e| format!("spawn exe: {}", e))?;
    let code = out.exit_code.unwrap_or(1);
    Ok((code, out.timed_out))
}
