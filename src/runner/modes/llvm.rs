use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::{MirCompiler, MirInstruction}, box_trait::IntegerBox};
use nyash_rust::mir::passes::method_id_inject::inject_method_ids;
use std::{fs, process};

impl NyashRunner {
    /// Execute LLVM mode (split)
    pub(crate) fn execute_llvm_mode(&self, filename: &str) {
        // Initialize plugin host so method_id injection can resolve plugin calls
        crate::runner_plugin_init::init_bid_plugins();

        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
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
        if let Ok(out_path) = std::env::var("NYASH_LLVM_OBJ_OUT") {
            #[cfg(feature = "llvm")]
            {
                use nyash_rust::backend::llvm_compile_to_object;
                // Ensure parent directory exists for the object file
                if let Some(parent) = std::path::Path::new(&out_path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[Runner/LLVM] emitting object to {} (cwd={})", out_path, std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
                }
                if let Err(e) = llvm_compile_to_object(&module, &out_path) {
                    eprintln!("❌ LLVM object emit error: {}", e);
                    process::exit(1);
                }
                // Verify object presence and size (>0)
                match std::fs::metadata(&out_path) {
                    Ok(meta) => {
                        if meta.len() == 0 {
                            eprintln!("❌ LLVM object is empty: {}", out_path);
                            process::exit(1);
                        }
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[LLVM] object emitted: {} ({} bytes)", out_path, meta.len());
                        }
                    }
                    Err(e) => {
                        // Try one immediate retry by writing through the compiler again (rare FS lag safeguards)
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[Runner/LLVM] object not found after emit, retrying once: {} ({})", out_path, e);
                        }
                        if let Err(e2) = llvm_compile_to_object(&module, &out_path) {
                            eprintln!("❌ LLVM object emit error (retry): {}", e2);
                            process::exit(1);
                        }
                        match std::fs::metadata(&out_path) {
                            Ok(meta2) => {
                                if meta2.len() == 0 {
                                    eprintln!("❌ LLVM object is empty (after retry): {}", out_path);
                                    process::exit(1);
                                }
                                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                    eprintln!("[LLVM] object emitted after retry: {} ({} bytes)", out_path, meta2.len());
                                }
                            }
                            Err(e3) => {
                                eprintln!("❌ LLVM object not found after emit (retry): {} ({})", out_path, e3);
                                process::exit(1);
                            }
                        }
                    }
                }
                return;
            }
            #[cfg(not(feature = "llvm"))]
            {
                eprintln!("❌ LLVM backend not available (object emit).");
                process::exit(1);
            }
        }

        // Execute via LLVM backend (mock or real)
        #[cfg(feature = "llvm")]
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
                },
                Err(e) => { eprintln!("❌ LLVM execution error: {}", e); process::exit(1); }
            }
        }
        #[cfg(not(feature = "llvm"))]
        {
            println!("🔧 Mock LLVM Backend Execution:");
            println!("   Build with --features llvm for real compilation.");
            if let Some(main_func) = module.functions.get("Main.main") {
                for (_bid, block) in &main_func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            MirInstruction::Return { value: Some(_) } => { println!("✅ Mock exit code: 42"); process::exit(42); }
                            MirInstruction::Return { value: None } => { println!("✅ Mock exit code: 0"); process::exit(0); }
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
