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
    mir::{MirCompiler, MirPrinter, MirInstruction},
    backend::{VM, wasm::WasmBackend, aot::AotBackend},
};

#[cfg(feature = "llvm")]
use crate::backend::{llvm_compile_and_execute};
use std::{fs, process};

// BID prototype imports
use crate::bid::{PluginRegistry, PluginBoxInstance};

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
        match &self.config.command {
            crate::cli::NyashCommand::Run { 
                file, benchmark, iterations, .. 
            } => {
                // Try to initialize BID plugins from nyash.toml (best-effort)
                self.init_bid_plugins();
                
                // Benchmark mode - can run without a file
                if *benchmark {
                    println!("📊 Nyash Performance Benchmark Suite");
                    println!("====================================");
                    println!("Running {} iterations per test...", iterations);
                    println!();
                    
                    self.execute_benchmark_mode(*iterations);
                    return;
                }

                if let Some(ref filename) = file {
                    self.execute_file_mode(filename);
                } else {
                    self.execute_demo_mode();
                }
            },
            crate::cli::NyashCommand::Bid { subcommand } => {
                self.execute_bid_command(subcommand);
            }
        }
    }

    fn init_bid_plugins(&self) {
        // Best-effort init; do not fail the program if missing
        if let Ok(()) = crate::bid::registry::init_global_from_config("nyash.toml") {
            let reg = crate::bid::registry::global().unwrap();
            // If FileBox plugin is present, try a birth/fini cycle as a smoke test
            if let Some(plugin) = reg.get_by_name("FileBox") {
                if let Ok(inst) = PluginBoxInstance::birth(plugin) {
                    println!("🔌 BID plugin loaded: FileBox (instance_id={})", inst.instance_id);
                    // Drop will call fini
                    return;
                }
            }
            println!("🔌 BID registry initialized");
        }
    }

    /// Execute BID commands
    fn execute_bid_command(&self, subcommand: &crate::cli::BidSubcommand) {
        use crate::cli::BidSubcommand;
        
        match subcommand {
            BidSubcommand::Gen { target, bid_file, output_dir, force, dry_run } => {
                self.execute_bid_gen(target, bid_file, output_dir.as_deref(), *force, *dry_run);
            }
        }
    }
    
    /// Execute BID code generation
    fn execute_bid_gen(&self, target: &str, bid_file: &str, output_dir: Option<&str>, force: bool, dry_run: bool) {
        use crate::bid::{BidDefinition, CodeGenerator, CodeGenTarget, CodeGenOptions};
        use std::path::{Path, PathBuf};
        
        println!("🚀 BID Code Generator");
        println!("Target: {}", target);
        println!("BID file: {}", bid_file);
        
        // Parse target
        let target = match CodeGenTarget::from_str(target) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        };
        
        // Load BID definition
        let bid_path = Path::new(bid_file);
        let bid = match BidDefinition::load_from_file(bid_path) {
            Ok(bid) => bid,
            Err(e) => {
                eprintln!("❌ Failed to load BID file '{}': {}", bid_file, e);
                std::process::exit(1);
            }
        };
        
        println!("✅ Loaded BID definition: {}", bid.name());
        
        // Determine output directory
        let output_dir = match output_dir {
            Some(dir) => PathBuf::from(dir),
            None => {
                let base_name = bid_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                PathBuf::from("out").join(target.as_str()).join(base_name)
            }
        };
        
        println!("📁 Output directory: {}", output_dir.display());
        
        // Create generation options
        let options = CodeGenOptions::new(target.clone(), output_dir)
            .with_force(force)
            .with_dry_run(dry_run);
        
        // Generate code
        match CodeGenerator::generate(&bid, &options) {
            Ok(result) => {
                if dry_run {
                    println!("\n🔍 Dry run - preview of generated files:");
                    CodeGenerator::preview_files(&result);
                } else {
                    println!("\n✅ Code generation successful!");
                    println!("Generated {} files for target '{}':", result.files.len(), target.as_str());
                    for file in &result.files {
                        println!("  📄 {}", file.path.display());
                    }
                }
            },
            Err(e) => {
                eprintln!("❌ Code generation failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    /// Execute file-based mode with backend selection
    fn execute_file_mode(&self, filename: &str) {
        // Extract run configuration from the command
        if let crate::cli::NyashCommand::Run { 
            dump_mir, verify_mir, compile_wasm, compile_native, backend, debug_fuel, output_file, ..
        } = &self.config.command {
            
            if *dump_mir || *verify_mir {
                println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
                self.execute_mir_mode(filename, *dump_mir, *verify_mir, *debug_fuel);
            } else if *compile_wasm {
                println!("🌐 Nyash WASM Compiler - Processing file: {} 🌐", filename);
                self.execute_wasm_mode(filename, output_file.as_deref());
            } else if *compile_native {
                println!("🚀 Nyash AOT Compiler - Processing file: {} 🚀", filename);
                self.execute_aot_mode(filename, output_file.as_deref());
            } else if backend == "vm" {
                println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
                self.execute_vm_mode(filename);
            } else if backend == "llvm" {
                println!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
                self.execute_llvm_mode(filename);
            } else {
                println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
                if let Some(fuel) = debug_fuel {
                    println!("🔥 Debug fuel limit: {} iterations", fuel);
                } else {
                    println!("🔥 Debug fuel limit: unlimited");
                }
                println!("====================================================");
                
                self.execute_nyash_file(filename, *debug_fuel);
            }
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
    fn execute_nyash_file(&self, filename: &str, debug_fuel: Option<usize>) {
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
            },
            Err(e) => {
                // Use enhanced error reporting with source context
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }

    /// Execute MIR compilation and processing mode
    fn execute_mir_mode(&self, filename: &str, dump_mir: bool, verify_mir: bool, debug_fuel: Option<usize>) {
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
        if verify_mir {
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
        if dump_mir {
            // Extract mir_verbose from the command if needed
            let mir_verbose = if let crate::cli::NyashCommand::Run { mir_verbose, .. } = &self.config.command {
                *mir_verbose
            } else {
                false
            };
            
            let mut printer = if mir_verbose {
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
    fn execute_wasm_mode(&self, filename: &str, output_file: Option<&str>) {
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

        // Compile to WASM (Phase 9.77a fix: use compile_to_wat instead of compile_module)
        let mut wasm_backend = WasmBackend::new();
        let wat_text = match wasm_backend.compile_to_wat(compile_result.module) {
            Ok(wat) => wat,
            Err(e) => {
                eprintln!("❌ WASM compilation error: {}", e);
                process::exit(1);
            }
        };

        // Determine output file
        let output = output_file
            .unwrap_or_else(|| {
                if filename.ends_with(".nyash") {
                    filename.strip_suffix(".nyash").unwrap_or(filename)
                } else {
                    filename
                }
            });
        let output_file = format!("{}.wat", output);

        // Write WAT output (already a string)
        let output_str = wat_text;
        
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
    fn execute_aot_mode(&self, filename: &str, output_file: Option<&str>) {
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

        let output = output_file
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

    /// Execute LLVM mode
    fn execute_llvm_mode(&self, filename: &str) {
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

        println!("📊 MIR Module compiled successfully!");
        println!("📊 Functions: {}", compile_result.module.functions.len());

        // Execute via LLVM backend (mock implementation)
        #[cfg(feature = "llvm")]
        {
            let temp_path = "nyash_llvm_temp";
            match llvm_compile_and_execute(&compile_result.module, temp_path) {
                Ok(result) => {
                    if let Some(int_result) = result.as_any().downcast_ref::<IntegerBox>() {
                        let exit_code = int_result.value;
                        println!("✅ LLVM execution completed!");
                        println!("📊 Exit code: {}", exit_code);
                        
                        // Exit with the same code for testing
                        process::exit(exit_code as i32);
                    } else {
                        println!("✅ LLVM execution completed (non-integer result)!");
                        println!("📊 Result: {}", result.to_string_box().value);
                    }
                },
                Err(e) => {
                    eprintln!("❌ LLVM execution error: {}", e);
                    process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "llvm"))]
        {
            // Mock implementation for demonstration
            println!("🔧 Mock LLVM Backend Execution:");
            println!("   This demonstrates the LLVM backend integration structure.");
            println!("   For actual LLVM compilation, build with --features llvm");
            println!("   and ensure LLVM 17+ development libraries are installed.");
            
            // Analyze the MIR to provide a meaningful mock result
            if let Some(main_func) = compile_result.module.functions.get("Main.main") {
                for (_block_id, block) in &main_func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            MirInstruction::Return { value: Some(_) } => {
                                println!("   📊 Found return instruction - would generate LLVM return 42");
                                println!("✅ Mock LLVM execution completed!");
                                println!("📊 Mock exit code: 42");
                                process::exit(42);
                            }
                            MirInstruction::Return { value: None } => {
                                println!("   📊 Found void return - would generate LLVM return 0");
                                println!("✅ Mock LLVM execution completed!");
                                println!("📊 Mock exit code: 0");
                                process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            println!("✅ Mock LLVM execution completed!");
            println!("📊 Mock exit code: 0");
            process::exit(0);
        }
    }

    /// Execute benchmark mode
    fn execute_benchmark_mode(&self, iterations: u32) {
        println!("🏁 Running benchmark mode with {} iterations", iterations);
        
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
        for _ in 0..iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut interpreter = NyashInterpreter::new();
                let _ = interpreter.execute(ast);
            }
        }
        let interpreter_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", 
            iterations, interpreter_time, 
            iterations as f64 / interpreter_time.as_secs_f64());

        // Benchmark VM if available
        println!("\n🚀 VM Backend:");
        let start = std::time::Instant::now();
        for _ in 0..iterations {
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
            iterations, vm_time, 
            iterations as f64 / vm_time.as_secs_f64());

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
            command: crate::cli::NyashCommand::Run {
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
            },
        };
        
        let runner = NyashRunner::new(config);
        
        // Test that we can access the command structure
        match &runner.config.command {
            crate::cli::NyashCommand::Run { backend, .. } => {
                assert_eq!(backend, "interpreter");
            },
            _ => panic!("Expected Run command"),
        }
    }
}
