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
    let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

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
        if removed > 0 { crate::cli_v!("[PyVM] escape_elide_barriers: removed {} barriers", removed); }
    }

    // Delegate to common PyVM harness
    match crate::runner::modes::common_util::pyvm::run_pyvm_harness_lib(&compile_result.module, "pyvm") {
        Ok(code) => { process::exit(code); }
        Err(e) => { eprintln!("❌ PyVM error: {}", e); process::exit(1); }
    }
}
