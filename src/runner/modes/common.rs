use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter, box_factory::builtin::BuiltinGroups};
use std::{fs, process};

impl NyashRunner {
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

