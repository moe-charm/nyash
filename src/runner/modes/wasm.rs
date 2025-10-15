use super::super::NyashRunner;
#[cfg(feature = "wasm-backend")]
use nyash_rust::{parser::NyashParser, mir::MirCompiler, backend::wasm::WasmBackend};
#[cfg(feature = "wasm-backend")]
use std::{fs, process};

impl NyashRunner {
    /// Execute WASM compilation mode (split)
    #[cfg(feature = "wasm-backend")]
    pub(crate) fn execute_wasm_mode(&self, filename: &str) {
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
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };

        // Compile to WAT text
        let mut wasm_backend = WasmBackend::new();
        let wat_text = match wasm_backend.compile_to_wat(compile_result.module) {
            Ok(wat) => wat,
            Err(e) => { eprintln!("❌ WASM compilation error: {}", e); process::exit(1); }
        };

        // Determine output file
        let groups = self.config.as_groups();
        let output = groups.output_file.as_deref().unwrap_or_else(|| {
            if filename.ends_with(".hako") { filename.strip_suffix(".hako").unwrap_or(filename) }
            else if filename.ends_with(".nyash") { filename.strip_suffix(".nyash").unwrap_or(filename) } else { filename }
        });
        let output_file = format!("{}.wat", output);

        match fs::write(&output_file, wat_text) {
            Ok(()) => { println!("✅ WASM compilation successful!\nOutput written to: {}", output_file); },
            Err(e) => { eprintln!("❌ Error writing WASM file {}: {}", output_file, e); process::exit(1); }
        }
    }
}
