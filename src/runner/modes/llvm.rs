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
        if injected > 0 && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[LLVM] method_id injected: {} places", injected);
        }

        // If explicit object path is requested, emit object only
        if let Ok(_out_path) = std::env::var("NYASH_LLVM_OBJ_OUT") {
            #[cfg(feature = "llvm-harness")]
            {
                // Harness path (optional): if NYASH_LLVM_USE_HARNESS=1, try Python/llvmlite first.
                let use_harness = crate::config::env::llvm_use_harness();
                if use_harness {
                    if let Some(parent) = std::path::Path::new(&_out_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let py = which::which("python3").ok();
                    if let Some(py3) = py {
                        let harness = std::path::Path::new("tools/llvmlite_harness.py");
                        if harness.exists() {
                            // 1) Emit MIR(JSON) to a temp file
                            let tmp_dir = std::path::Path::new("tmp");
                            let _ = std::fs::create_dir_all(tmp_dir);
                            let mir_json_path = tmp_dir.join("nyash_harness_mir.json");
                            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness(
                                &module,
                                &mir_json_path,
                            ) {
                                eprintln!("❌ MIR JSON emit error: {}", e);
                                process::exit(1);
                            }
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!(
                                    "[Runner/LLVM] using llvmlite harness → {} (mir={})",
                                    _out_path,
                                    mir_json_path.display()
                                );
                            }
                            // 2) Run harness with --in/--out（失敗時は即エラー）
                            let mut cmd = std::process::Command::new(py3);
                            cmd.args([
                                harness.to_string_lossy().as_ref(),
                                "--in",
                                &mir_json_path.display().to_string(),
                                "--out",
                                &_out_path,
                            ]);
                            let out = crate::runner::modes::common_util::io::spawn_with_timeout(cmd, 20_000)
                                .map_err(|e| format!("spawn harness: {}", e))
                                .unwrap();
                            if out.timed_out || !out.status_ok {
                                eprintln!(
                                    "❌ llvmlite harness failed (timeout={} code={:?})",
                                    out.timed_out,
                                    out.exit_code
                                );
                                process::exit(1);
                            }
                            // Verify
                            match std::fs::metadata(&_out_path) {
                                Ok(meta) if meta.len() > 0 => {
                                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref()
                                        == Some("1")
                                    {
                                        eprintln!(
                                            "[LLVM] object emitted by harness: {} ({} bytes)",
                                            _out_path,
                                            meta.len()
                                        );
                                    }
                                    return;
                                }
                                _ => {
                                    eprintln!(
                                        "❌ harness output not found or empty: {}",
                                        _out_path
                                    );
                                    process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("❌ harness script not found: {}", harness.display());
                            process::exit(1);
                        }
                    }
                    eprintln!("❌ python3 not found in PATH. Install Python 3 to use the harness.");
                    process::exit(1);
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
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[Runner/LLVM] emitting object to {} (cwd={})",
                        _out_path,
                        std::env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default()
                    );
                }
                if let Err(e) = llvm_compile_to_object(&module, &_out_path) {
                    eprintln!("❌ LLVM object emit error: {}", e);
                    process::exit(1);
                }
                match std::fs::metadata(&_out_path) {
                    Ok(meta) if meta.len() > 0 => {
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[LLVM] object emitted: {} ({} bytes)",
                                _out_path,
                                meta.len()
                            );
                        }
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
