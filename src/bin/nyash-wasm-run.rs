/*!
 * Nyash WASM Runner - Execute Nyash WASM modules with host functions
 * 
 * Phase 4-3c: Standalone WASM executor for testing
 */

use nyash_rust::backend::wasm::{WasmExecutor, WasmError};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <file.wat>", args[0]);
        eprintln!("");
        eprintln!("Execute a Nyash WASM module with host functions");
        std::process::exit(1);
    }
    
    let wat_file = &args[1];
    
    if !Path::new(wat_file).exists() {
        eprintln!("❌ File not found: {}", wat_file);
        std::process::exit(1);
    }
    
    println!("🚀 Nyash WASM Runner - Executing: {} 🚀", wat_file);
    println!("");
    
    // Create executor
    let executor = WasmExecutor::new()?;
    
    // Execute WAT file
    match executor.execute_wat_file(wat_file) {
        Ok(output) => {
            if !output.is_empty() {
                println!("📝 Program output:");
                println!("{}", output);
            }
            println!("");
            println!("✅ Execution completed successfully!");
        },
        Err(e) => {
            eprintln!("❌ Execution error: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}