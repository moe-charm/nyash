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
        } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[ny-compiler] fallback to default path (MVP unavailable for this input)");
        }
    }

    // Direct v0 bridge when requested via CLI/env
    let use_ny_parser = runner.config.parser_ny
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
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!(
                        "🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀",
                        filename
                    );
                }
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
    if runner.config.dump_ast {
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
        println!("{:#?}", ast);
        return;
    }

    // MIR dump/verify
    if runner.config.dump_mir || runner.config.verify_mir {
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
        }
        runner.execute_mir_mode(filename);
        return;
    }

    // WASM / AOT (feature-gated)
    if runner.config.compile_wasm {
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
    if runner.config.compile_native {
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
    match runner.config.backend.as_str() {
        "mir" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
            }
            runner.execute_mir_mode(filename);
        }
        "vm" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            }
            runner.execute_vm_mode(filename);
        }
        #[cfg(feature = "cranelift-jit")]
        "jit-direct" => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!(
                    "⚡ Nyash JIT-Direct Backend - Executing file: {} ⚡",
                    filename
                );
            }
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
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
            }
            runner.execute_llvm_mode(filename);
        }
        _ => {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!(
                    "🦀 Nyash Rust Implementation - Executing file: {} 🦀",
                    filename
                );
                if let Some(fuel) = runner.config.debug_fuel {
                    println!("🔥 Debug fuel limit: {} iterations", fuel);
                } else {
                    println!("🔥 Debug fuel limit: unlimited");
                }
                println!("====================================================");
            }
            super::modes::interpreter::execute_nyash_file(
                filename,
                runner.config.debug_fuel.clone(),
            );
        }
    }
}

impl NyashRunner {
    pub(crate) fn execute_mir_module(&self, module: &crate::mir::MirModule) {
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
