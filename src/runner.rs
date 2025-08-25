/*!
 * Execution Runner Module - Nyash File and Mode Execution Coordinator
 * 
 * This module handles all execution logic, backend selection, and mode coordination,
 * separated from CLI parsing and the main entry point.
 */

use nyash_rust::cli::CliConfig;
use nyash_rust::{
    box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox, BoxCore},
    tokenizer::{NyashTokenizer},
    ast::ASTNode,
    parser::NyashParser,
    interpreter::NyashInterpreter,
    mir::{MirCompiler, MirPrinter, MirInstruction},
    backend::VM,
};
use nyash_rust::runtime::{NyashRuntime, NyashRuntimeBuilder};
use nyash_rust::box_factory::builtin::BuiltinGroups;
use nyash_rust::interpreter::SharedState;
use nyash_rust::box_factory::user_defined::UserDefinedBoxFactory;
use nyash_rust::core::model::BoxDeclaration as CoreBoxDecl;
use std::sync::Arc;

#[cfg(feature = "wasm-backend")]
use nyash_rust::backend::{wasm::WasmBackend, aot::AotBackend};

#[cfg(feature = "llvm")]
use nyash_rust::backend::{llvm_compile_and_execute};
use std::{fs, process};
mod modes;

// v2 plugin system imports
use nyash_rust::runtime;
use nyash_rust::runner_plugin_init;

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
        // 🏭 Phase 9.78b: Initialize unified registry
        runtime::init_global_unified_registry();
        
        // Try to initialize BID plugins from nyash.toml (best-effort)
        runner_plugin_init::init_bid_plugins();

        // Optional: enable VM stats via CLI flags
        if self.config.vm_stats {
            std::env::set_var("NYASH_VM_STATS", "1");
        }
        if self.config.vm_stats_json {
            // Prefer explicit JSON flag over any default
            std::env::set_var("NYASH_VM_STATS_JSON", "1");
        }
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
            // Delegate file-mode execution to modes::common dispatcher
            self.run_file(filename);
        } else {
            self.execute_demo_mode();
        }
    }

    // init_bid_plugins moved to runner_plugin_init.rs

    /// Execute file-based mode with backend selection

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
        
        // Test: immediate file creation (use relative path to avoid sandbox issues)
        std::fs::create_dir_all("development/debug_hang_issue").ok();
        std::fs::write("development/debug_hang_issue/test.txt", "START").ok();
        
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
            .open("development/debug_hang_issue/debug_trace.log") 
        {
            use std::io::Write;
            let _ = writeln!(file, "=== MAIN: Parse successful ===");
            let _ = file.flush();
        }
        
        eprintln!("🔍 DEBUG: Creating interpreter...");
        
        // Execute the AST
        let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
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

        // Compile to MIR (opt passes configurable)
        let mut mir_compiler = MirCompiler::with_options(!self.config.no_optimize);
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
            let mut printer = if self.config.mir_verbose { MirPrinter::verbose() } else { MirPrinter::new() };
            if self.config.mir_verbose_effects { printer.set_show_effects_inline(true); }
            
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

        // Prepare runtime and collect Box declarations for VM user-defined types
        let runtime = {
            let rt = NyashRuntimeBuilder::new()
                .with_builtin_groups(BuiltinGroups::native_full())
                .build();
            self.collect_box_declarations(&ast, &rt);
            // Register UserDefinedBoxFactory backed by the same declarations
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() {
                reg.register(udf);
            }
            rt
        };

        // Compile to MIR (opt passes configurable)
        let mut mir_compiler = MirCompiler::with_options(!self.config.no_optimize);
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Execute with VM using prepared runtime
        let mut vm = VM::with_runtime(runtime);
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

    

    /// Collect Box declarations from AST and register into runtime
    fn collect_box_declarations(&self, ast: &ASTNode, runtime: &NyashRuntime) {
        fn walk(node: &ASTNode, runtime: &NyashRuntime) {
            match node {
                ASTNode::Program { statements, .. } => {
                    for st in statements { walk(st, runtime); }
                }
                ASTNode::FunctionDeclaration { body, .. } => {
                    // Walk into function bodies to find nested box declarations
                    for st in body { walk(st, runtime); }
                }
                ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    // Walk into methods/constructors to find nested box declarations
                    for (_mname, mnode) in methods {
                        walk(mnode, runtime);
                    }
                    for (_ckey, cnode) in constructors {
                        walk(cnode, runtime);
                    }
                    let decl = CoreBoxDecl {
                        name: name.clone(),
                        fields: fields.clone(),
                        public_fields: public_fields.clone(),
                        private_fields: private_fields.clone(),
                        methods: methods.clone(),
                        constructors: constructors.clone(),
                        init_fields: init_fields.clone(),
                        weak_fields: weak_fields.clone(),
                        is_interface: *is_interface,
                        extends: extends.clone(),
                        implements: implements.clone(),
                        type_parameters: type_parameters.clone(),
                    };
                    if let Ok(mut map) = runtime.box_declarations.write() {
                        map.insert(name.clone(), decl);
                    }
                }
                _ => {}
            }
        }
        walk(ast, runtime);
    }

    // execute_wasm_mode moved to runner::modes::wasm

    // execute_aot_mode moved to runner::modes::aot

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
                let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
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
            let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
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
            let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
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
            vm_stats: false,
            vm_stats_json: false,
        };
        
        let runner = NyashRunner::new(config);
        assert_eq!(runner.config.backend, "interpreter");
    }
}
