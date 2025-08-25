use super::super::NyashRunner;
#[cfg(feature = "wasm-backend")]
use nyash_rust::{parser::NyashParser, mir::MirCompiler, backend::aot::AotBackend};
#[cfg(feature = "wasm-backend")]
use std::{fs, process};

impl NyashRunner {
    /// Execute AOT compilation mode (split)
    #[cfg(feature = "wasm-backend")]
    pub(crate) fn execute_aot_mode(&self, filename: &str) {
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

        // Determine output file (no extension change)
        let output = self.config.output_file.as_deref().unwrap_or(filename);

        let mut aot_backend = AotBackend::new();
        match aot_backend.compile_to_executable(compile_result.module, output) {
            Ok(()) => { println!("✅ AOT compilation successful!\nExecutable written to: {}", output); },
            Err(e) => { eprintln!("❌ AOT compilation error: {}", e); process::exit(1); }
        }
    }
}

