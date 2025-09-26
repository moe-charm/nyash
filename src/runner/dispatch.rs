/*!
 * Runner dispatch helpers — execute MIR module and report result
 */

use super::*;
use crate::runner::json_v0_bridge;
use nyash_rust::parser::NyashParser;
use std::{fs, process};

/// Thin file dispatcher: select backend and delegate to mode executors
pub(crate) fn execute_file_with_backend(runner: &NyashRunner, filename: &str) {
    // Selfhost pipeline (Ny -> JSON v0) behind env gate
    if std::env::var("NYASH_USE_NY_COMPILER").ok().as_deref() == Some("1") {
        if runner.try_run_selfhost_pipeline(filename) {
            return;
        } else {
            crate::cli_v!("[ny-compiler] fallback to default path (MVP unavailable for this input)");
        }
    }

    // Direct v0 bridge when requested via CLI/env
    let groups = runner.config.as_groups();
    let use_ny_parser = groups.parser.parser_ny
        || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
    if use_ny_parser {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        match json_v0_bridge::parse_source_v0_to_module(&code) {
            Ok(module) => {
                crate::cli_v!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                runner.execute_mir_module(&module);
                return;
            }
            Err(e) => {
                eprintln!("❌ Direct bridge parse error: {}", e);
                process::exit(1);
            }
        }
    }

    // AST dump mode
    if groups.debug.dump_ast {
        println!("🧠 Nyash AST Dump - Processing file: {}", filename);
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // Optional macro expansion dump (no-op expansion for now)
        let ast2 = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, true);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else {
            ast.clone()
        };
        println!("{:#?}", ast2);
        return;
    }

    // Dump expanded AST as JSON v0 and exit
    if runner.config.dump_expanded_ast_json {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        let expanded = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, false);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else { ast };
        let j = crate::r#macro::ast_json::ast_to_json(&expanded);
        println!("{}", j.to_string());
        return;
    }

    // MIR dump/verify
    if groups.debug.dump_mir || groups.debug.verify_mir {
        crate::cli_v!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
        runner.execute_mir_mode(filename);
        return;
    }

    // WASM / AOT (feature-gated)
    if groups.compile_wasm {
        #[cfg(feature = "wasm-backend")]
        {
            super::modes::wasm::execute_wasm_mode(runner, filename);
            return;
        }
        #[cfg(not(feature = "wasm-backend"))]
        {
            eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend");
            process::exit(1);
        }
    }
    if groups.compile_native {
        #[cfg(feature = "cranelift-jit")]
        {
            runner.execute_aot_mode(filename);
            return;
        }
        #[cfg(not(feature = "cranelift-jit"))]
        {
            eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit");
            process::exit(1);
        }
    }

    // Backend selection
    match groups.backend.backend.as_str() {
        "mir" => {
            crate::cli_v!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
            runner.execute_mir_mode(filename);
        }
        "vm" => {
            crate::cli_v!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            // Prefer lightweight in-crate MIR interpreter as VM fallback
            runner.execute_vm_fallback_interpreter(filename);
        }
        #[cfg(feature = "cranelift-jit")]
        "jit-direct" => {
            crate::cli_v!("⚡ Nyash JIT-Direct Backend - Executing file: {} ⚡", filename);
            #[cfg(feature = "cranelift-jit")]
            {
                // Use independent JIT-direct runner method (no VM execute loop)
                runner.run_file_jit_direct(filename);
            }
            #[cfg(not(feature = "cranelift-jit"))]
            {
                eprintln!("❌ Cranelift backend not available. Please rebuild with: cargo build --features cranelift-jit");
                process::exit(1);
            }
        }
        "llvm" => {
            crate::cli_v!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
            runner.execute_llvm_mode(filename);
        }
        other => {
            eprintln!("❌ Unknown backend: {}. Use 'vm' or 'llvm'.", other);
            std::process::exit(2);
        }
    }
}

impl NyashRunner {
    pub(crate) fn execute_mir_module(&self, module: &crate::mir::MirModule) {
        // If CLI requested MIR JSON emit, write to file and exit immediately.
        let groups = self.config.as_groups();
        if let Some(path) = groups.emit.emit_mir_json.as_ref() {
            let p = std::path::Path::new(path);
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, p) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            println!("MIR JSON written: {}", p.display());
            std::process::exit(0);
        }
        // If CLI requested EXE emit, generate JSON then invoke ny-llvmc to link NyRT and exit.
        if let Some(exe_out) = groups.emit.emit_exe.as_ref() {
            if let Err(e) = crate::runner::modes::common_util::exec::ny_llvmc_emit_exe_bin(
                module,
                exe_out,
                groups.emit.emit_exe_nyrt.as_deref(),
                groups.emit.emit_exe_libs.as_deref(),
            ) {
                eprintln!("❌ {}", e);
                std::process::exit(1);
            }
            println!("EXE written: {}", exe_out);
            std::process::exit(0);
        }
        use crate::backend::MirInterpreter;
        use crate::box_trait::{BoolBox, IntegerBox, StringBox};
        use crate::boxes::FloatBox;
        use crate::mir::MirType;

        let mut interp = MirInterpreter::new();
        match interp.execute_module(module) {
            Ok(result) => {
                println!("✅ MIR interpreter execution completed!");
                if let Some(func) = module.functions.get("main") {
                    let (ety, sval) = match &func.signature.return_type {
                        MirType::Float => {
                            if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                                ("Float", format!("{}", fb.value))
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Float", format!("{}", ib.value as f64))
                            } else {
                                ("Float", result.to_string_box().value)
                            }
                        }
                        MirType::Integer => {
                            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Integer", ib.value.to_string())
                            } else {
                                ("Integer", result.to_string_box().value)
                            }
                        }
                        MirType::Bool => {
                            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                                ("Bool", bb.value.to_string())
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Bool", (ib.value != 0).to_string())
                            } else {
                                ("Bool", result.to_string_box().value)
                            }
                        }
                        MirType::String => {
                            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                                ("String", sb.value.clone())
                            } else {
                                ("String", result.to_string_box().value)
                            }
                        }
                        _ => (result.type_name(), result.to_string_box().value),
                    };
                    println!("ResultType(MIR): {}", ety);
                    println!("Result: {}", sval);
                } else {
                    println!("Result: {:?}", result);
                }
            }
            Err(e) => {
                eprintln!("❌ MIR interpreter error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
