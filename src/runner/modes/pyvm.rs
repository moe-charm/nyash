use super::super::NyashRunner;
use nyash_rust::{mir::MirCompiler, parser::NyashParser};
use std::{fs, process};

/// Execute using PyVM only (no Rust VM runtime). Emits MIR(JSON) and invokes tools/pyvm_runner.py.
pub fn execute_pyvm_only(_runner: &NyashRunner, filename: &str) {
    // Read the file
    let code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };

    // Parse to AST
    let ast = match NyashParser::parse_from_string(&code) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };

    // Compile to MIR (respect default optimizer setting)
    let mut mir_compiler = MirCompiler::with_options(true);
    let mut compile_result = match mir_compiler.compile(ast) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ MIR compilation error: {}", e);
            process::exit(1);
        }
    };

    // Optional: VM-only escape analysis elision pass retained for parity with VM path
    if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
        let removed = nyash_rust::mir::passes::escape::escape_elide_barriers_vm(&mut compile_result.module);
        if removed > 0 && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[PyVM] escape_elide_barriers: removed {} barriers", removed);
        }
    }

    // Emit MIR JSON for PyVM harness
    let tmp_dir = std::path::Path::new("tmp");
    let _ = std::fs::create_dir_all(tmp_dir);
    let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
    if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness(
        &compile_result.module,
        &mir_json_path,
    ) {
        eprintln!("❌ PyVM MIR JSON emit error: {}", e);
        process::exit(1);
    }

    // Pick entry: prefer Main.main or main
    let entry = if compile_result.module.functions.contains_key("Main.main") {
        "Main.main"
    } else if compile_result.module.functions.contains_key("main") {
        "main"
    } else {
        "Main.main"
    };

    // Locate python3 and run harness
    let py = which::which("python3").ok();
    if let Some(py3) = py {
        let runner = std::path::Path::new("tools/pyvm_runner.py");
        if !runner.exists() {
            eprintln!("❌ PyVM runner not found: {}", runner.display());
            process::exit(1);
        }
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!(
                "[Runner/PyVM] {} (mir={})",
                filename,
                mir_json_path.display()
            );
        }
        let mut cmd = std::process::Command::new(py3);
        cmd.args([
            runner.to_string_lossy().as_ref(),
            "--in",
            &mir_json_path.display().to_string(),
            "--entry",
            entry,
        ]);
        let out = crate::runner::modes::common_util::io::spawn_with_timeout(cmd, 10_000)
            .map_err(|e| format!("spawn pyvm: {}", e))
            .unwrap();
        let code = if out.timed_out { 1 } else { out.exit_code.unwrap_or(1) };
        if out.timed_out && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("❌ PyVM timeout");
        }
        process::exit(code);
    } else {
        eprintln!("❌ python3 not found in PATH. Install Python 3 to use PyVM.");
        process::exit(1);
    }
}
