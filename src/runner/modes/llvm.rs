use super::super::NyashRunner;
use nyash_rust::mir::passes::method_id_inject::inject_method_ids;
use nyash_rust::{
    mir::{MirCompiler, MirInstruction},
    parser::NyashParser,
};
use std::{fs, process};

impl NyashRunner {
    /// Execute LLVM mode (split)
    pub(crate) fn execute_llvm_mode(&self, filename: &str) {
        // Initialize plugin host so method_id injection can resolve plugin calls
        crate::runner_plugin_init::init_bid_plugins();

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
        // Macro expansion (env-gated)
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        println!("📊 MIR Module compiled successfully!");
        println!("📊 Functions: {}", compile_result.module.functions.len());

        // Inject method_id for BoxCall/PluginInvoke where resolvable (by-id path)
        #[allow(unused_mut)]
        let mut module = compile_result.module.clone();
        let injected = inject_method_ids(&mut module);
        if injected > 0 { crate::cli_v!("[LLVM] method_id injected: {} places", injected); }

        // If explicit object path is requested, emit object only
        if let Ok(_out_path) = std::env::var("NYASH_LLVM_OBJ_OUT") {
            #[cfg(feature = "llvm-harness")]
            {
                // Harness path (optional): if NYASH_LLVM_USE_HARNESS=1, try Python/llvmlite first.
                if crate::config::env::llvm_use_harness() {
                    if let Err(e) = crate::runner::modes::common_util::exec::llvmlite_emit_object(&module, &_out_path, 20_000) {
                        eprintln!("❌ {}", e);
                        process::exit(1);
                    }
                    return;
                }
                // Verify object presence and size (>0)
                match std::fs::metadata(&_out_path) {
                    Ok(meta) => {
                        if meta.len() == 0 {
                            eprintln!("❌ harness object is empty: {}", _out_path);
                            process::exit(1);
                        }
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[LLVM] object emitted: {} ({} bytes)",
                                _out_path,
                                meta.len()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "❌ harness output not found after emit: {} ({})",
                            _out_path, e
                        );
                        process::exit(1);
                    }
                }
                return;
            }
            #[cfg(all(not(feature = "llvm-harness"), feature = "llvm-inkwell-legacy"))]
            {
                use nyash_rust::backend::llvm_compile_to_object;
                // Ensure parent directory exists for the object file
                if let Some(parent) = std::path::Path::new(&_out_path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                crate::cli_v!(
                    "[Runner/LLVM] emitting object to {} (cwd={})",
                    _out_path,
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                );
                if let Err(e) = llvm_compile_to_object(&module, &_out_path) {
                    eprintln!("❌ LLVM object emit error: {}", e);
                    process::exit(1);
                }
                match std::fs::metadata(&_out_path) {
                    Ok(meta) if meta.len() > 0 => {
                        crate::cli_v!(
                            "[LLVM] object emitted: {} ({} bytes)",
                            _out_path,
                            meta.len()
                        );
                    }
                    _ => {
                        eprintln!("❌ LLVM object not found or empty: {}", _out_path);
                        process::exit(1);
                    }
                }
                return;
            }
            #[cfg(all(not(feature = "llvm-harness"), not(feature = "llvm-inkwell-legacy")))]
            {
                eprintln!("❌ LLVM backend not available (object emit).");
                process::exit(1);
            }
        }

        // Execute via LLVM backend (harness preferred)
        #[cfg(feature = "llvm-harness")]
        {
            if crate::config::env::llvm_use_harness() {
                // Prefer producing a native executable via ny-llvmc, then execute it
                let exe_out = "tmp/nyash_llvm_run";
                let libs = std::env::var("NYASH_LLVM_EXE_LIBS").ok();
                match crate::runner::modes::common_util::exec::ny_llvmc_emit_exe_lib(
                    &module,
                    exe_out,
                    None,
                    libs.as_deref(),
                ) {
                    Ok(()) => {
                        match crate::runner::modes::common_util::exec::run_executable(exe_out, &[], 20_000) {
                            Ok((code, _timed_out)) => {
                                println!("✅ LLVM (harness) execution completed (exit={})", code);
                                std::process::exit(code);
                            }
                            Err(e) => {
                                eprintln!("❌ run executable error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ ny-llvmc emit-exe error: {}", e);
                        eprintln!("   Hint: build ny-llvmc: cargo build -p nyash-llvm-compiler --release");
                        std::process::exit(1);
                    }
                }
            }
        }

        // Execute via LLVM backend (mock or real)
        #[cfg(feature = "llvm-inkwell-legacy")]
        {
            use nyash_rust::backend::llvm_compile_and_execute;
            let temp_path = "nyash_llvm_temp";
            match llvm_compile_and_execute(&module, temp_path) {
                Ok(result) => {
                    if let Some(int_result) = result.as_any().downcast_ref::<IntegerBox>() {
                        let exit_code = int_result.value;
                        println!("✅ LLVM execution completed!");
                        println!("📊 Exit code: {}", exit_code);
                        process::exit(exit_code as i32);
                    } else {
                        println!("✅ LLVM execution completed (non-integer result)!");
                        println!("📊 Result: {}", result.to_string_box().value);
                    }
                }
                Err(e) => {
                    eprintln!("❌ LLVM execution error: {}", e);
                    process::exit(1);
                }
            }
        }
        #[cfg(all(not(feature = "llvm-inkwell-legacy")))]
        {
            println!("🔧 Mock LLVM Backend Execution:");
            println!("   Build with --features llvm-inkwell-legacy for Rust/inkwell backend, or set NYASH_LLVM_OBJ_OUT and NYASH_LLVM_USE_HARNESS=1 for harness.");
            if let Some(main_func) = module.functions.get("Main.main") {
                for (_bid, block) in &main_func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            MirInstruction::Return { value: Some(_) } => {
                                println!("✅ Mock exit code: 42");
                                process::exit(42);
                            }
                            MirInstruction::Return { value: None } => {
                                println!("✅ Mock exit code: 0");
                                process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            }
            println!("✅ Mock exit code: 0");
            process::exit(0);
        }
    }
}

// emit_mir_json_for_harness moved to crate::runner::mir_json_emit
