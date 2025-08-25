use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::{MirCompiler, MirInstruction}, box_trait::IntegerBox};
use std::{fs, process};

impl NyashRunner {
    /// Execute LLVM mode (split)
    pub(crate) fn execute_llvm_mode(&self, filename: &str) {
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

        // Execute via LLVM backend (mock or real)
        #[cfg(feature = "llvm")]
        {
            use nyash_rust::backend::llvm_compile_and_execute;
            let temp_path = "nyash_llvm_temp";
            match llvm_compile_and_execute(&compile_result.module, temp_path) {
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
            if let Some(main_func) = compile_result.module.functions.get("Main.main") {
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

