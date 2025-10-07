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

        // Using handling (unified resolver)
        let mut code_ref: &str;
        let cleaned_code_owned;
        let prelude_asts: Vec<nyash_rust::ast::ASTNode>;

        if crate::config::env::enable_using() {
            let options = crate::runner::modes::common_util::resolve::UsingResolveOptions {
                allow_skip_ast_merge: false,
                collect_alias_names: false,
            };

            match crate::runner::modes::common_util::resolve::resolve_using_with_preludes(
                self, &code, filename, options
            ) {
                Ok(result) => {
                    cleaned_code_owned = result.cleaned_code;
                    code_ref = &cleaned_code_owned;
                    prelude_asts = result.prelude_asts;
                }
                Err(e) => {
                    eprintln!("❌ Pipeline error: `using` resolution error: {}", e);
                    process::exit(1);
                }
            }
        } else {
            code_ref = &code;
            prelude_asts = Vec::new();
        }
        // Pre-expand '@name[:T] = expr' sugar at line-head (same as common path)
        let preexpanded_owned = crate::runner::modes::common_util::resolve::preexpand_at_local(code_ref);
        // Pre-lexical normalization (raw strings, numeric separators) shared with VM/PyVM
        let prelex_owned = crate::runner::modes::common_util::prelex::prelex_normalize(&preexpanded_owned);
        code_ref = &prelex_owned;

        // Parse to AST (main)
        let main_ast = match NyashParser::parse_from_string(code_ref) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // Merge preludes + main when enabled
        let ast = if !prelude_asts.is_empty() {
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(prelude_asts, &main_ast)
        } else { main_ast };
        // Alias desugar (to prefixed form); llvm path ignores alias set for now
        let ast = {
            use std::collections::HashSet;
            let aliases: HashSet<String> = HashSet::new();
            crate::runner::modes::common_util::resolve::alias_tools::desugar_alias_field_access(&ast, &aliases, true)
        };
        // Macro expansion (env-gated) after merge
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
        let ast = crate::runner::modes::macro_child::normalize_core_pass(&ast);

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Optional: emit static call-site trace (JSON lines) for parity checks
        if std::env::var("NYASH_CALL_TRACE").ok().as_deref() == Some("1") {
            use nyash_rust::mir::MirInstruction;
            // helper to escape strings
            fn esc(s: &str) -> String { s.replace('"', "\\\"") }
            for (fname, func) in compile_result.module.functions.iter() {
                // Attempt deterministic block order by sorting ids when possible
                let mut keys: Vec<_> = func.blocks.keys().cloned().collect();
                keys.sort_by_key(|k| k.as_u32());
                for bb_id in keys.into_iter() {
                    if let Some(bb) = func.blocks.get(&bb_id) {
                        for inst in bb.instructions.iter() {
                            match inst {
                                MirInstruction::Call { callee, args, .. } => {
                                    let bb = bb_id.as_u32();
                                    match callee {
                                        Some(nyash_rust::mir::definitions::call_unified::Callee::Global(name)) => {
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"Global:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(name), args.len(), esc(fname), bb);
                                        }
                                        Some(nyash_rust::mir::definitions::call_unified::Callee::ModuleFunction(name)) => {
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"ModuleFunction:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(name), args.len(), esc(fname), bb);
                                        }
                                        Some(nyash_rust::mir::definitions::call_unified::Callee::Method{ box_name, method, .. }) => {
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"Method:{}.{}/{}\",\"fn\":\"{}\",\"bb\":{}}}", esc(box_name), esc(method), args.len(), esc(fname), bb);
                                        }
                                        Some(nyash_rust::mir::definitions::call_unified::Callee::Extern(name)) => {
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"Extern:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(name), args.len(), esc(fname), bb);
                                        }
                                        Some(other) => {
                                            let tag = format!("{:?}", other);
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(&tag), args.len(), esc(fname), bb);
                                        }
                                        None => {
                                            eprintln!("{{\"kind\":\"call_static\",\"callee\":\"Legacy\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", args.len(), esc(fname), bb);
                                        }
                                    }
                                }
                                MirInstruction::BoxCall { method, args, .. } => {
                                    let bb = bb_id.as_u32();
                                    eprintln!("{{\"kind\":\"call_static\",\"callee\":\"BoxCall:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(method), args.len(), esc(fname), bb);
                                }
                                MirInstruction::PluginInvoke { method, args, .. } => {
                                    let bb = bb_id.as_u32();
                                    eprintln!("{{\"kind\":\"call_static\",\"callee\":\"PluginInvoke:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(method), args.len(), esc(fname), bb);
                                }
                                MirInstruction::ExternCall { iface_name, method_name, args, .. } => {
                                    let bb = bb_id.as_u32();
                                    let full = format!("{}.{}", iface_name, method_name);
                                    eprintln!("{{\"kind\":\"call_static\",\"callee\":\"Extern:{}\",\"argc\":{},\"fn\":\"{}\",\"bb\":{}}}", esc(&full), args.len(), esc(fname), bb);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        println!("📊 MIR Module compiled successfully!");
        println!("📊 Functions: {}", compile_result.module.functions.len());

        // Inject method_id for BoxCall/PluginInvoke where resolvable (by-id path)
        #[allow(unused_mut)]
        let mut module = compile_result.module.clone();
        let injected = inject_method_ids(&mut module);
        if injected > 0 { crate::cli_v!("[LLVM] method_id injected: {} places", injected); }

        // Dev/Test helper: allow executing via PyVM harness when requested
        if std::env::var("SMOKES_USE_PYVM").ok().as_deref() == Some("1") {
            #[cfg(feature = "pyvm-bridge")]
            {
                match super::common_util::pyvm::run_pyvm_harness_lib(&module, "llvm-ast") {
                    Ok(code) => { std::process::exit(code); }
                    Err(e) => { eprintln!("❌ PyVM harness error: {}", e); std::process::exit(1); }
                }
            }
            #[cfg(not(feature = "pyvm-bridge"))]
            {
                eprintln!("{}", crate::common::diagnostics::runner_pyvm_bridge_disabled_warn());
            }
        }

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
                            Ok((code, _timed_out, stdout_text)) => {
                                // Forward program stdout so parity tests can compare outputs
                                if !stdout_text.is_empty() { print!("{}", stdout_text); }
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
