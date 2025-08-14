/*!
 * Execution Runner Module - Nyash File and Mode Execution Coordinator
 * 
 * This module handles all execution logic, backend selection, and mode coordination,
 * separated from CLI parsing and the main entry point.
 */

use crate::cli::CliConfig;
use crate::{
    box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox, BoxCore},
    tokenizer::{NyashTokenizer},
    ast::ASTNode,
    parser::NyashParser,
    interpreter::NyashInterpreter,
    mir::{MirCompiler, MirPrinter},
    backend::{VM, wasm::WasmBackend, aot::AotBackend},
};
use std::{fs, process};

/// Main execution coordinator
pub struct NyashRunner {
    config: CliConfig,
}

impl NyashRunner {
    /// Create a new runner with the given configuration
    pub fn new(config: CliConfig) -> Self {
        Self { config }
    }

    /// Run Nyash based on the configuration
    pub fn run(&self) {
        // Benchmark mode - can run without a file
        if self.config.benchmark {
            println!("📊 Nyash Performance Benchmark Suite");
            println!("====================================");
            println!("Running {} iterations per test...", self.config.iterations);
            println!();
            
            self.execute_benchmark_mode();
            return;
        }

        if let Some(ref filename) = self.config.file {
            self.execute_file_mode(filename);
        } else {
            self.execute_demo_mode();
        }
    }

    /// Execute file-based mode with backend selection
    fn execute_file_mode(&self, filename: &str) {
        if self.config.dump_mir || self.config.verify_mir {
            println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
            self.execute_mir_mode(filename);
        } else if self.config.compile_wasm {
            println!("🌐 Nyash WASM Compiler - Processing file: {} 🌐", filename);
            self.execute_wasm_mode(filename);
        } else if self.config.compile_native {
            println!("🚀 Nyash AOT Compiler - Processing file: {} 🚀", filename);
            self.execute_aot_mode(filename);
        } else if self.config.backend == "vm" {
            println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            self.execute_vm_mode(filename);
        } else {
            println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
            if let Some(fuel) = self.config.debug_fuel {
                println!("🔥 Debug fuel limit: {} iterations", fuel);
            } else {
                println!("🔥 Debug fuel limit: unlimited");
            }
            println!("====================================================");
            
            self.execute_nyash_file(filename);
        }
    }

    /// Execute demo mode with all demonstrations
    fn execute_demo_mode(&self) {
        println!("🦀 Nyash Rust Implementation - Everything is Box! 🦀");
        println!("====================================================");
        
        // Demonstrate basic Box creation and operations
        demo_basic_boxes();
        
        // Demonstrate Box operations
        demo_box_operations();
        
        // Demonstrate Box collections
        demo_box_collections();
        
        // Demonstrate Environment & Scope management
        demo_environment_system();
        
        // Demonstrate Tokenizer system  
        demo_tokenizer_system();
        
        // Demonstrate Parser system
        demo_parser_system();
        
        // Demonstrate Interpreter system
        demo_interpreter_system();
        
        println!("\n🎉 All Box operations completed successfully!");
        println!("Memory safety guaranteed by Rust's borrow checker! 🛡️");
    }

    /// Execute Nyash file with interpreter
    fn execute_nyash_file(&self, filename: &str) {
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
        
        // Test: immediate file creation
        std::fs::write("/mnt/c/git/nyash/development/debug_hang_issue/test.txt", "START").ok();
        
        // Parse the code with debug fuel limit
        eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", self.config.debug_fuel);
        let ast = match NyashParser::parse_from_string_with_fuel(&code, self.config.debug_fuel) {
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
            .open("/mnt/c/git/nyash/development/debug_hang_issue/debug_trace.log") 
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
            },
            Err(e) => {
                // Use enhanced error reporting with source context
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }

    /// Execute MIR compilation and processing mode
    fn execute_mir_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Verify MIR if requested
        if self.config.verify_mir {
            println!("🔍 Verifying MIR...");
            match &compile_result.verification_result {
                Ok(()) => println!("✅ MIR verification passed!"),
                Err(errors) => {
                    eprintln!("❌ MIR verification failed:");
                    for error in errors {
                        eprintln!("  • {}", error);
                    }
                    process::exit(1);
                }
            }
        }

        // Dump MIR if requested
        if self.config.dump_mir {
            let mut printer = if self.config.mir_verbose {
                MirPrinter::verbose()
            } else {
                MirPrinter::new()
            };
            
            println!("🚀 MIR Output for {}:", filename);
            println!("{}", printer.print_module(&compile_result.module));
        }
    }

    /// Execute VM mode
    fn execute_vm_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Execute with VM
        let mut vm = VM::new();
        match vm.execute_module(&compile_result.module) {
            Ok(result) => {
                println!("✅ VM execution completed successfully!");
                println!("Result: {:?}", result);
            },
            Err(e) => {
                eprintln!("❌ VM execution error: {}", e);
                process::exit(1);
            }
        }
    }

    /// Execute WASM compilation mode
    fn execute_wasm_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Compile to WASM
        let mut wasm_backend = WasmBackend::new();
        let wasm_code = match wasm_backend.compile_module(compile_result.module) {
            Ok(wasm) => wasm,
            Err(e) => {
                eprintln!("❌ WASM compilation error: {}", e);
                process::exit(1);
            }
        };

        // Determine output file
        let output = self.config.output_file.as_deref()
            .unwrap_or_else(|| {
                if filename.ends_with(".nyash") {
                    filename.strip_suffix(".nyash").unwrap_or(filename)
                } else {
                    filename
                }
            });
        let output_file = format!("{}.wat", output);

        // Write WASM output
        let output_str = match std::str::from_utf8(&wasm_code) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("❌ Generated WASM is not valid UTF-8");
                process::exit(1);
            }
        };
        
        match fs::write(&output_file, output_str) {
            Ok(()) => {
                println!("✅ WASM compilation successful!");
                println!("Output written to: {}", output_file);
            },
            Err(e) => {
                eprintln!("❌ Error writing WASM file {}: {}", output_file, e);
                process::exit(1);
            }
        }
    }

    /// Execute AOT compilation mode
    fn execute_aot_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Compile via AOT backend
        let mut aot_backend = match AotBackend::new() {
            Ok(backend) => backend,
            Err(e) => {
                eprintln!("❌ Failed to create AOT backend: {}", e);
                process::exit(1);
            }
        };

        let output = self.config.output_file.as_deref()
            .unwrap_or_else(|| {
                if filename.ends_with(".nyash") {
                    filename.strip_suffix(".nyash").unwrap_or(filename)
                } else {
                    filename
                }
            });

        match aot_backend.compile_to_executable(compile_result.module, output) {
            Ok(()) => {
                println!("✅ AOT compilation successful!");
                println!("Executable written to: {}", output);
            },
            Err(e) => {
                eprintln!("❌ AOT compilation error: {}", e);
                process::exit(1);
            }
        }
    }

    /// Execute benchmark mode
    fn execute_benchmark_mode(&self) {
        println!("🏁 Running benchmark mode with {} iterations", self.config.iterations);
        
        // Simple benchmark test file
        let test_code = r#"
        local x
        x = 42
        local y 
        y = x + 58
        return y
        "#;

        println!("\n🧪 Test code:");
        println!("{}", test_code);
        
        // Benchmark interpreter
        println!("\n⚡ Interpreter Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut interpreter = NyashInterpreter::new();
                let _ = interpreter.execute(ast);
            }
        }
        let interpreter_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", 
            self.config.iterations, interpreter_time, 
            self.config.iterations as f64 / interpreter_time.as_secs_f64());

        // Benchmark VM if available
        println!("\n🚀 VM Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut mir_compiler = MirCompiler::new();
                if let Ok(compile_result) = mir_compiler.compile(ast) {
                    let mut vm = VM::new();
                    let _ = vm.execute_module(&compile_result.module);
                }
            }
        }
        let vm_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", 
            self.config.iterations, vm_time, 
            self.config.iterations as f64 / vm_time.as_secs_f64());

        // Performance comparison
        let speedup = interpreter_time.as_secs_f64() / vm_time.as_secs_f64();
        println!("\n📊 Performance Summary:");
        println!("  VM is {:.2}x {} than Interpreter", 
            if speedup > 1.0 { speedup } else { 1.0 / speedup },
            if speedup > 1.0 { "faster" } else { "slower" });
    }
}

// Demo functions (moved from main.rs)
fn demo_basic_boxes() {
    println!("\n📦 1. Basic Box Creation:");
    
    // Create different types of boxes
    let string_box = StringBox::new("Hello, Nyash!".to_string());
    let int_box = IntegerBox::new(42);
    let bool_box = BoolBox::new(true);
    let void_box = VoidBox::new();
    
    println!("  StringBox: {}", string_box.to_string_box().value);
    println!("  IntegerBox: {}", int_box.to_string_box().value);
    println!("  BoolBox: {}", bool_box.to_string_box().value);
    println!("  VoidBox: {}", void_box.to_string_box().value);
    
    // Show unique IDs
    println!("  Box IDs: String={}, Integer={}, Bool={}, Void={}", 
        string_box.box_id(), int_box.box_id(), bool_box.box_id(), void_box.box_id());
}

fn demo_box_operations() {
    println!("\n🔄 2. Box Operations:");
    
    // Addition between boxes
    let left = IntegerBox::new(10);
    let right = IntegerBox::new(32);
    let add_box = AddBox::new(Box::new(left), Box::new(right));
    
    println!("  10 + 32 = {}", add_box.to_string_box().value);
    
    // String concatenation
    let str1 = StringBox::new("Hello, ".to_string());
    let str2 = StringBox::new("World!".to_string());
    let concat_box = AddBox::new(Box::new(str1), Box::new(str2));
    
    println!("  \"Hello, \" + \"World!\" = {}", concat_box.to_string_box().value);
}

fn demo_box_collections() {
    println!("\n📚 3. Box Collections:");
    
    // This would be expanded when ArrayBox is implemented
    println!("  Box collections functionality placeholder");
    println!("  (ArrayBox and other collection types will be demonstrated here)");
}

fn demo_environment_system() {
    println!("\n🌍 4. Environment & Scope Management:");
    println!("  Environment demo placeholder - full testing done in interpreter");
}

fn demo_tokenizer_system() {
    println!("\n🔤 5. Tokenizer System:");
    
    // Test code to tokenize
    let test_code = "x = 42 + y";
    println!("  Input: {}", test_code);
    
    // Tokenize the code
    let mut tokenizer = NyashTokenizer::new(test_code);
    
    match tokenizer.tokenize() {
        Ok(tokens) => {
            println!("  Tokenized {} tokens successfully", tokens.len());
        },
        Err(e) => println!("  Tokenization error: {}", e),
    }
}

fn demo_parser_system() {
    println!("\n🌳 6. Parser & AST System:");
    
    // Test simple box declaration
    println!("  📝 Simple Box Declaration Test:");
    let simple_code = r#"
    box TestBox {
        value
        
        getValue() {
            return this.value
        }
    }
    "#;
    
    match NyashParser::parse_from_string(simple_code) {
        Ok(ast) => {
            println!("    Input: {}", simple_code.trim());
            println!("    AST: {}", ast);
            
            if let ASTNode::Program { statements, .. } = &ast {
                println!("    Program has {} statements", statements.len());
                for (i, stmt) in statements.iter().enumerate() {
                    println!("      [{}] {}", i, stmt.info());
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test assignment and method call
    println!("\n  🚀 Assignment & Method Call Test:");
    let assignment_code = r#"
    obj = new TestBox()
    obj.value = "test123"
    print("Direct field: " + obj.value)
    print("Method call: " + obj.getValue())
    "#;
    
    match NyashParser::parse_from_string(assignment_code) {
        Ok(ast) => {
            println!("    Successfully parsed assignment & method call code");
            
            if let ASTNode::Program { statements, .. } = &ast {
                println!("    Parsed {} statements:", statements.len());
                for (i, stmt) in statements.iter().enumerate() {
                    println!("      [{}] {} ({})", i, stmt.info(), stmt.node_type());
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test expression parsing
    println!("\n  ⚡ Expression Parsing Test:");
    let expr_code = r#"
    result = x + y * z
    condition = a == b && c < d
    "#;
    
    match NyashParser::parse_from_string(expr_code) {
        Ok(ast) => {
            println!("    Successfully parsed complex expressions");
            
            if let ASTNode::Program { statements, .. } = &ast {
                for (i, stmt) in statements.iter().enumerate() {
                    if let ASTNode::Assignment { target, value, .. } = stmt {
                        println!("      Assignment [{}]: {} = {}", i, target.info(), value.info());
                    }
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test control structures
    println!("\n  🔄 Control Structure Test:");
    let control_code = r#"
    if condition {
        print("True branch")
    } else {
        print("False branch")
    }
    
    loop {
        print("Loop body")
        return
    }
    "#;
    
    match NyashParser::parse_from_string(control_code) {
        Ok(ast) => {
            println!("    Successfully parsed control structures");
            
            if let ASTNode::Program { statements, .. } = &ast {
                for (i, stmt) in statements.iter().enumerate() {
                    println!("      [{}] {} ({})", i, stmt.info(), stmt.node_type());
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
}

fn demo_interpreter_system() {
    println!("\n🎭 7. Interpreter System:");
    
    // Simple execution test
    let simple_code = r#"
    local x
    x = 42
    return x
    "#;
    
    println!("  📝 Simple Variable Test:");
    println!("    Code: {}", simple_code.trim());
    
    match NyashParser::parse_from_string(simple_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            match interpreter.execute(ast) {
                Ok(result) => {
                    println!("    ✅ Result: {}", result.to_string_box().value);
                },
                Err(e) => {
                    println!("    ❌ Execution error: {}", e);
                }
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Expression evaluation test
    let expr_code = r#"
    local result
    result = 10 + 32
    return result
    "#;
    
    println!("\n  ⚡ Expression Evaluation Test:");
    println!("    Code: {}", expr_code.trim());
    
    match NyashParser::parse_from_string(expr_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            match interpreter.execute(ast) {
                Ok(result) => {
                    println!("    ✅ Result: {}", result.to_string_box().value);
                },
                Err(e) => {
                    println!("    ❌ Execution error: {}", e);
                }
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let config = CliConfig {
            file: None,
            debug_fuel: Some(100000),
            dump_mir: false,
            verify_mir: false,
            mir_verbose: false,
            backend: "interpreter".to_string(),
            compile_wasm: false,
            compile_native: false,
            output_file: None,
            benchmark: false,
            iterations: 10,
        };
        
        let runner = NyashRunner::new(config);
        assert_eq!(runner.config.backend, "interpreter");
    }
}