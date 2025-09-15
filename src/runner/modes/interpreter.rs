use crate::parser::NyashParser;
use crate::interpreter::NyashInterpreter;
use std::{fs, process};

/// Execute Nyash file with interpreter
pub fn execute_nyash_file(filename: &str, debug_fuel: Option<usize>) {
    // Read the file
    let code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };
    
    println!("📝 File contents:\n{}", code);
    println!("\n🚀 Parsing and executing...\n");
    
    // Test: immediate file creation (use relative path to avoid sandbox issues)
    std::fs::create_dir_all("development/debug_hang_issue").ok();
    std::fs::write("development/debug_hang_issue/test.txt", "START").ok();
    
    // Parse the code with debug fuel limit
    eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", debug_fuel);
    let ast = match NyashParser::parse_from_string_with_fuel(&code, debug_fuel) {
        Ok(ast) => {
            eprintln!("🔍 DEBUG: Parse completed, AST created");
            ast
        },
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };
    
    eprintln!("🔍 DEBUG: About to print parse success message...");
    println!("✅ Parse successful!");
    eprintln!("🔍 DEBUG: Parse success message printed");
    
    // Debug log file write
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("development/debug_hang_issue/debug_trace.log") 
    {
        use std::io::Write;
        let _ = writeln!(file, "=== MAIN: Parse successful ===");
        let _ = file.flush();
    }
    
    eprintln!("🔍 DEBUG: Creating interpreter...");
    
    // Execute the AST
    let mut interpreter = NyashInterpreter::new();
    eprintln!("🔍 DEBUG: Starting execution...");
    match interpreter.execute(ast) {
        Ok(result) => {
            println!("✅ Execution completed successfully!");
            println!("Result: {}", result.to_string_box().value);
            // Structured concurrency: best-effort join of spawned tasks at program end
            let join_ms: u64 = std::env::var("NYASH_JOIN_ALL_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
            crate::runtime::global_hooks::join_all_registered_futures(join_ms);
        },
        Err(e) => {
            // Use enhanced error reporting with source context
            eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
            process::exit(1);
        }
    }
}
