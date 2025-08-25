use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter, box_factory::builtin::BuiltinGroups};
use std::{fs, process};

impl NyashRunner {
    /// File-mode dispatcher (thin wrapper around backend/mode selection)
    pub(crate) fn run_file(&self, filename: &str) {
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
            #[cfg(feature = "wasm-backend")]
            { self.execute_aot_mode(filename); return; }
            #[cfg(not(feature = "wasm-backend"))]
            { eprintln!("❌ AOT backend not available. Please rebuild with: cargo build --features wasm-backend"); process::exit(1); }
        }

        // Backend selection
        match self.config.backend.as_str() {
            "vm" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
                }
                self.execute_vm_mode(filename);
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
        let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                println!("✅ Execution completed successfully!");
                println!("Result: {}", result.to_string_box().value);
            },
            Err(e) => {
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }
}
