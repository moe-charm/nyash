use super::super::NyashRunner;
use crate::runner::json_v0_bridge;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter};
// Use the library crate's plugin init module rather than the bin crate root
use nyash_rust::runner_plugin_init;
use std::{fs, process};

impl NyashRunner {
    /// File-mode dispatcher (thin wrapper around backend/mode selection)
    pub(crate) fn run_file(&self, filename: &str) {
        // Direct v0 bridge when requested via CLI/env
        let use_ny_parser = self.config.parser_ny || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
        if use_ny_parser {
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            match json_v0_bridge::parse_source_v0_to_module(&code) {
                Ok(module) => {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        println!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                    }
                    self.execute_mir_module(&module);
                    return;
                }
                Err(e) => { eprintln!("❌ Direct bridge parse error: {}", e); process::exit(1); }
            }
        }
        // AST dump mode
        if self.config.dump_ast {
            println!("🧠 Nyash AST Dump - Processing file: {}", filename);
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            let ast = match NyashParser::parse_from_string(&code) {
                Ok(ast) => ast,
                Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
            };
            println!("{:#?}", ast);
            return;
        }

        // MIR dump/verify
        if self.config.dump_mir || self.config.verify_mir {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
            }
            self.execute_mir_mode(filename);
            return;
        }

        // WASM / AOT (feature-gated)
        if self.config.compile_wasm {
            #[cfg(feature = "wasm-backend")]
            { self.execute_wasm_mode(filename); return; }
            #[cfg(not(feature = "wasm-backend"))]
            { eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend"); process::exit(1); }
        }
        if self.config.compile_native {
            #[cfg(feature = "cranelift-jit")]
            { self.execute_aot_mode(filename); return; }
            #[cfg(not(feature = "cranelift-jit"))]
            { eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit"); process::exit(1); }
        }

        // Backend selection
        match self.config.backend.as_str() {
            "mir" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
                }
                self.execute_mir_interpreter_mode(filename);
            }
            "vm" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
                }
                self.execute_vm_mode(filename);
            }
            "cranelift" => {
                #[cfg(feature = "cranelift-jit")]
                {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        println!("⚙️  Nyash Cranelift JIT - Executing file: {}", filename);
                    }
                    self.execute_cranelift_mode(filename);
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
                self.execute_llvm_mode(filename);
            }
            _ => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
                    if let Some(fuel) = self.config.debug_fuel {
                        println!("🔥 Debug fuel limit: {} iterations", fuel);
                    } else {
                        println!("🔥 Debug fuel limit: unlimited");
                    }
                    println!("====================================================");
                }
                self.execute_nyash_file(filename);
            }
        }
    }

    /// Execute Nyash file with interpreter (common helper)
    pub(crate) fn execute_nyash_file(&self, filename: &str) {
        // Ensure plugin host and provider mappings are initialized (idempotent)
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            // Call via lib crate to avoid referring to the bin crate root
            runner_plugin_init::init_bid_plugins();
        }
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };

        println!("📝 File contents:\n{}", code);
        println!("\n🚀 Parsing and executing...\n");

        // Parse the code with debug fuel limit
        eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", self.config.debug_fuel);
        let ast = match NyashParser::parse_from_string_with_fuel(&code, self.config.debug_fuel) {
            Ok(ast) => { eprintln!("🔍 DEBUG: Parse completed, AST created"); ast },
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        println!("✅ Parse successful!");

        // Execute the AST
        let mut interpreter = NyashInterpreter::new();
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                println!("✅ Execution completed successfully!");
                // Normalize display via semantics: prefer numeric, then string, then fallback
                let disp = {
                    // Special-case: plugin IntegerBox → call .get to fetch numeric value
                    if let Some(p) = result.as_any().downcast_ref::<nyash_rust::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if p.box_type == "IntegerBox" {
                            // Scope the lock strictly to this block
                            let fetched = {
                                let host = nyash_rust::runtime::get_global_plugin_host();
                                let res = if let Ok(ro) = host.read() {
                                    if let Ok(Some(vb)) = ro.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(ib) = vb.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            Some(ib.value.to_string())
                                        } else {
                                            Some(vb.to_string_box().value)
                                        }
                                    } else { None }
                                } else { None };
                                res
                            };
                            if let Some(s) = fetched { s } else {
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                    .map(|i| i.to_string())
                                    .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                    .unwrap_or_else(|| result.to_string_box().value)
                            }
                        } else {
                            nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                .map(|i| i.to_string())
                                .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                .unwrap_or_else(|| result.to_string_box().value)
                        }
                    } else {
                        nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            .map(|i| i.to_string())
                            .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                            .unwrap_or_else(|| result.to_string_box().value)
                    }
                };
                println!("Result: {}", disp);
            },
            Err(e) => {
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }
}
