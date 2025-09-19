use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::MirCompiler};
use std::{fs, process};

impl NyashRunner {
    /// Execute Cranelift JIT mode (skeleton)
    pub(crate) fn execute_cranelift_mode(&self, filename: &str) {
        // Read source
        let code = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        // Parse → AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
        // AST → MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(r) => r,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };
        println!("📊 MIR Module compiled (Cranelift JIT skeleton)");

        // Execute via Cranelift JIT (feature‑gated)
        #[cfg(feature = "cranelift-jit")]
        {
            use nyash_rust::backend::cranelift_compile_and_execute;
            match cranelift_compile_and_execute(&compile_result.module, "nyash_cljit_temp") {
                Ok(result) => {
                    println!("✅ Cranelift JIT execution completed (skeleton)!");
                    println!("📊 Result: {}", result.to_string_box().value);
                }
                Err(e) => { eprintln!("❌ Cranelift JIT error: {}", e); process::exit(1); }
            }
        }
        #[cfg(not(feature = "cranelift-jit"))]
        {
            eprintln!("❌ Cranelift JIT not available. Rebuild with --features cranelift-jit");
            process::exit(1);
        }
    }
}
