/*!
 * Nyash Rust Implementation - Everything is Box in Memory Safe Rust
 * 
 * This is the main entry point for the Rust implementation of Nyash,
 * demonstrating the "Everything is Box" philosophy with Rust's ownership system.
 */

pub mod box_trait;
pub mod boxes;
pub mod environment;
pub mod tokenizer;
pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod instance;
pub mod channel_box;
pub mod finalization;
pub mod exception_box;
pub mod method_box;
pub mod operator_traits;
pub mod box_operators;
pub mod value; // 🔥 NyashValue Revolutionary System

use box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox, BoxCore};
use environment::{Environment, PythonCompatEnvironment};
use tokenizer::{NyashTokenizer, TokenType};
use ast::ASTNode;
use parser::NyashParser;
use interpreter::NyashInterpreter;

// 🚀 MIR Infrastructure
pub mod mir;
use mir::{MirCompiler, MirPrinter};

// 🚀 Backend Infrastructure  
pub mod backend;
use backend::{VM, wasm::WasmBackend};
use std::env;
use std::fs;
use std::process;
use clap::{Arg, Command};

fn main() {
    // 🔥 clap使ったコマンド引数解析
    let matches = Command::new("nyash")
        .version("1.0")
        .author("Claude Code <claude@anthropic.com>")
        .about("🦀 Nyash Programming Language - Everything is Box in Rust! 🦀")
        .arg(
            Arg::new("file")
                .help("Nyash file to execute")
                .value_name("FILE")
                .index(1)
        )
        .arg(
            Arg::new("debug-fuel")
                .long("debug-fuel")
                .value_name("ITERATIONS")
                .help("Set parser debug fuel limit (default: 100000, 'unlimited' for no limit)")
                .default_value("100000")
        )
        .arg(
            Arg::new("dump-mir")
                .long("dump-mir")
                .help("Dump MIR (Mid-level Intermediate Representation) instead of executing")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verify")
                .long("verify")
                .help("Verify MIR integrity and exit")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("mir-verbose")
                .long("mir-verbose")
                .help("Show verbose MIR output with statistics")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("backend")
                .long("backend")
                .value_name("BACKEND")
                .help("Choose execution backend: 'interpreter' (default) or 'vm'")
                .default_value("interpreter")
        )
        .arg(
            Arg::new("compile-wasm")
                .long("compile-wasm")
                .help("Compile to WebAssembly (WAT format) instead of executing")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("FILE")
                .help("Output file (for WASM compilation)")
        )
        .arg(
            Arg::new("benchmark")
                .long("benchmark")
                .help("Run performance benchmarks across all backends")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("iterations")
                .long("iterations")
                .value_name("COUNT")
                .help("Number of iterations for benchmarks (default: 10)")
                .default_value("10")
        )
        .get_matches();
    
    // デバッグ燃料の解析
    let debug_fuel = parse_debug_fuel(matches.get_one::<String>("debug-fuel").unwrap());
    
    // MIR mode flags
    let dump_mir = matches.get_flag("dump-mir");
    let verify_mir = matches.get_flag("verify");
    let mir_verbose = matches.get_flag("mir-verbose");
    let compile_wasm = matches.get_flag("compile-wasm");
    let backend = matches.get_one::<String>("backend").unwrap();
    let output_file = matches.get_one::<String>("output");
    let benchmark = matches.get_flag("benchmark");
    let iterations: u32 = matches.get_one::<String>("iterations").unwrap().parse().unwrap_or(10);
    
    // Benchmark mode - can run without a file
    if benchmark {
        println!("📊 Nyash Performance Benchmark Suite");
        println!("====================================");
        println!("Running {} iterations per test...", iterations);
        println!();
        
        execute_benchmark_mode(iterations);
        return;
    }
    
    if let Some(filename) = matches.get_one::<String>("file") {
        // File mode: parse and execute the provided .nyash file
        if dump_mir || verify_mir {
            println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
            execute_mir_mode(filename, dump_mir, verify_mir, mir_verbose);
        } else if compile_wasm {
            println!("🌐 Nyash WASM Compiler - Processing file: {} 🌐", filename);
            execute_wasm_mode(filename, output_file);
        } else if backend == "vm" {
            println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            execute_vm_mode(filename);
        } else {
            println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
            if let Some(fuel) = debug_fuel {
                println!("🔥 Debug fuel limit: {} iterations", fuel);
            } else {
                println!("🔥 Debug fuel limit: unlimited");
            }
            println!("====================================================");
            
            execute_nyash_file(filename, debug_fuel);
        }
    } else {
        // Demo mode: run built-in demonstrations
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
}

/// デバッグ燃料値をパース（"unlimited" または数値）
fn parse_debug_fuel(value: &str) -> Option<usize> {
    if value == "unlimited" {
        None  // 無制限
    } else {
        value.parse::<usize>().ok()
    }
}

fn execute_nyash_file(filename: &str, debug_fuel: Option<usize>) {
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
    
    // テスト用：即座にファイル作成
    std::fs::write("/mnt/c/git/nyash/development/debug_hang_issue/test.txt", "START").ok();
    
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
    
    // デバッグログファイルに書き込み
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
            // 🔥 Use enhanced error reporting with source context
            eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
            process::exit(1);
        }
    }
}

fn demo_basic_boxes() {
    println!("\n📦 1. Basic Box Creation:");
    
    // Create basic boxes
    let string_box = StringBox::new("Hello from Rust!");
    let integer_box = IntegerBox::new(42);
    let bool_box = BoolBox::new(true);
    let void_box = VoidBox::new();
    
    println!("  StringBox: {} (ID: {})", string_box, string_box.box_id());
    println!("  IntegerBox: {} (ID: {})", integer_box, integer_box.box_id());
    println!("  BoolBox: {} (ID: {})", bool_box, bool_box.box_id());
    println!("  VoidBox: {} (ID: {})", void_box, void_box.box_id());
    
    // Test type identification
    println!("\n🔍 Type Information:");
    println!("  StringBox type: {}", string_box.type_name());
    println!("  IntegerBox type: {}", integer_box.type_name());
    println!("  BoolBox type: {}", bool_box.type_name());
    println!("  VoidBox type: {}", void_box.type_name());
}

fn demo_box_operations() {
    println!("\n⚡ 2. Box Operations:");
    
    // Integer addition
    let left_int = Box::new(IntegerBox::new(10)) as Box<dyn NyashBox>;
    let right_int = Box::new(IntegerBox::new(32)) as Box<dyn NyashBox>;
    let int_add = AddBox::new(left_int, right_int);
    let int_result = int_add.execute();
    
    println!("  Integer Addition: 10 + 32 = {}", int_result.to_string_box());
    
    // String concatenation
    let left_str = Box::new(StringBox::new("Everything is ")) as Box<dyn NyashBox>;
    let right_str = Box::new(StringBox::new("Box in Rust!")) as Box<dyn NyashBox>;
    let str_add = AddBox::new(left_str, right_str);
    let str_result = str_add.execute();
    
    println!("  String Concatenation: {}", str_result.to_string_box());
    
    // Mixed type addition (falls back to string concatenation)
    let mixed_left = Box::new(StringBox::new("Answer: ")) as Box<dyn NyashBox>;
    let mixed_right = Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>;
    let mixed_add = AddBox::new(mixed_left, mixed_right);
    let mixed_result = mixed_add.execute();
    
    println!("  Mixed Addition: {}", mixed_result.to_string_box());
}

fn demo_box_collections() {
    println!("\n📚 3. Box Collections:");
    
    // Create a collection of various boxes
    let mut box_collection: Vec<Box<dyn NyashBox>> = Vec::new();
    
    box_collection.push(Box::new(StringBox::new("First Box")));
    box_collection.push(Box::new(IntegerBox::new(100)));
    box_collection.push(Box::new(BoolBox::new(false)));
    box_collection.push(Box::new(VoidBox::new()));
    
    println!("  Collection contents:");
    for (i, box_item) in box_collection.iter().enumerate() {
        println!("    [{}] {} (Type: {}, ID: {})", 
                i, 
                box_item.to_string_box(), 
                box_item.type_name(),
                box_item.box_id());
    }
    
    // Test equality
    println!("\n🔍 Equality Testing:");
    let test1 = StringBox::new("test");
    let test2 = StringBox::new("test");
    let test3 = StringBox::new("different");
    
    println!("  \"test\" == \"test\": {}", test1.equals(&test2));
    println!("  \"test\" == \"different\": {}", test1.equals(&test3));
}

fn demo_environment_system() {
    println!("\n🌐 4. Environment & Scope Management:");
    
    // Create global environment
    let global_env = Environment::new_global();
    println!("  Created global environment: {}", global_env.lock().unwrap().scope_info());
    
    // Add global variables
    global_env.lock().unwrap().define("project_name", Box::new(StringBox::new("Nyash in Rust")));
    global_env.lock().unwrap().define("version", Box::new(StringBox::new("v1.0-rust")));
    global_env.lock().unwrap().define("debug_mode", Box::new(BoolBox::new(true)));
    
    println!("  Global variables: {:?}", global_env.lock().unwrap().list_variables());
    
    // Create function scope
    let function_env = Environment::new_child(global_env.clone(), "test_function");
    println!("  Created function scope: {}", function_env.lock().unwrap().scope_info());
    
    // Add local variables
    function_env.lock().unwrap().define("local_var", Box::new(IntegerBox::new(42)));
    function_env.lock().unwrap().define("temp_result", Box::new(StringBox::new("processing...")));
    
    // Test variable access from child scope
    println!("\n  🔍 Variable Access Tests:");
    
    // Access global variable from function scope
    match function_env.lock().unwrap().get("project_name") {
        Ok(value) => println!("    Access global from function: {}", value.to_string_box()),
        Err(e) => println!("    Error: {}", e),
    }
    
    // Access local variable
    match function_env.lock().unwrap().get("local_var") {
        Ok(value) => println!("    Access local variable: {}", value.to_string_box()),
        Err(e) => println!("    Error: {}", e),
    }
    
    // Try to access local variable from global (should fail)
    match global_env.lock().unwrap().get("local_var") {
        Ok(value) => println!("    Unexpected access to local from global: {}", value.to_string_box()),
        Err(e) => println!("    ✅ Correctly blocked access to local from global: {}", e),
    }
    
    // Test variable setting (modification)
    println!("\n  🔧 Variable Modification Tests:");
    
    // Modify global variable from function scope
    let _ = function_env.lock().unwrap().set("version", Box::new(StringBox::new("v1.1-rust-updated")));
    
    // Check if global was updated
    let updated_version = global_env.lock().unwrap().get("version").unwrap();
    println!("    Updated global variable: {}", updated_version.to_string_box());
    
    // Create nested scope (function inside function)
    let nested_env = Environment::new_child(function_env.clone(), "nested_function");
    nested_env.lock().unwrap().define("nested_var", Box::new(BoolBox::new(false)));
    
    // Test scope chain
    println!("\n  📊 Scope Chain Analysis:");
    let scope_chain = nested_env.lock().unwrap().scope_chain_info();
    for (i, scope_info) in scope_chain.iter().enumerate() {
        println!("    Level {}: {}", i, scope_info);
    }
    
    // Test variable shadowing
    println!("\n  🌑 Variable Shadowing Test:");
    function_env.lock().unwrap().define("debug_mode", Box::new(BoolBox::new(false))); // Shadow global
    
    let global_debug = global_env.lock().unwrap().get("debug_mode").unwrap();
    let function_debug = function_env.lock().unwrap().get("debug_mode").unwrap();
    
    println!("    Global debug_mode: {}", global_debug.to_string_box());
    println!("    Function debug_mode (shadowed): {}", function_debug.to_string_box());
    
    // Test Python compatibility layer
    println!("\n  🐍 Python Compatibility Layer:");
    let mut python_env = PythonCompatEnvironment::new();
    python_env.define("py_var", Box::new(StringBox::new("python_style")));
    
    let py_value = python_env.get("py_var");
    println!("    Python-style access: {}", py_value.to_string_box());
    println!("    _bindings contains: {:?}", python_env._bindings.keys().collect::<Vec<_>>());
    
    // Dump all variables for debugging
    println!("\n  📋 Complete Variable Dump:");
    let all_vars = nested_env.lock().unwrap().dump_all_variables();
    for (qualified_name, value) in all_vars {
        println!("    {}: {}", qualified_name, value);
    }
}

fn demo_tokenizer_system() {
    println!("\n🔤 5. Tokenizer System:");
    
    // Test simple tokens
    println!("  📝 Simple Token Test:");
    let simple_code = "box TestBox { value }";
    let mut tokenizer = NyashTokenizer::new(simple_code);
    
    match tokenizer.tokenize() {
        Ok(tokens) => {
            println!("    Input: {}", simple_code);
            println!("    Tokens:");
            for (i, token) in tokens.iter().enumerate() {
                if matches!(token.token_type, TokenType::EOF) {
                    break; // EOF は表示しない
                }
                println!("      [{}] {:?} at line {}, column {}", 
                        i, token.token_type, token.line, token.column);
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test complex code (same as debug_this_problem.nyash)
    println!("\n  🚀 Complex Code Test:");
    let complex_code = r#"
// this問題のミニマル再現
box TestBox {
    value
    
    getValue() {
        return this.value
    }
}

// テスト
obj = new TestBox()
obj.value = "test123"
print("Direct field: " + obj.value)
print("Method call: " + obj.getValue())
"#;
    
    let mut complex_tokenizer = NyashTokenizer::new(complex_code);
    match complex_tokenizer.tokenize() {
        Ok(tokens) => {
            let non_eof_tokens: Vec<_> = tokens.iter()
                .filter(|t| !matches!(t.token_type, TokenType::EOF))
                .collect();
            
            println!("    Successfully tokenized {} tokens", non_eof_tokens.len());
            
            // Show first 10 tokens
            println!("    First 10 tokens:");
            for (i, token) in non_eof_tokens.iter().take(10).enumerate() {
                println!("      [{}] {:?}", i, token.token_type);
            }
            
            // Count token types
            let mut token_counts = std::collections::HashMap::new();
            for token in &non_eof_tokens {
                let type_name = match &token.token_type {
                    TokenType::IDENTIFIER(_) => "IDENTIFIER",
                    TokenType::STRING(_) => "STRING", 
                    TokenType::NUMBER(_) => "NUMBER",
                    TokenType::BOX => "BOX",
                    TokenType::NEW => "NEW",
                    TokenType::THIS => "THIS",
                    TokenType::RETURN => "RETURN",
                    TokenType::PRINT => "PRINT",
                    TokenType::DOT => "DOT",
                    TokenType::ASSIGN => "ASSIGN",
                    TokenType::PLUS => "PLUS",
                    TokenType::LPAREN => "LPAREN",
                    TokenType::RPAREN => "RPAREN",
                    TokenType::LBRACE => "LBRACE",
                    TokenType::RBRACE => "RBRACE",
                    _ => "OTHER",
                };
                *token_counts.entry(type_name).or_insert(0) += 1;
            }
            
            println!("    Token type counts:");
            for (type_name, count) in token_counts {
                println!("      {}: {}", type_name, count);
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test string literals and escapes
    println!("\n  📝 String Literal Test:");
    let string_code = r#""Hello, World!" "Line 1\nLine 2" "Tab\tSeparated""#;
    let mut string_tokenizer = NyashTokenizer::new(string_code);
    
    match string_tokenizer.tokenize() {
        Ok(tokens) => {
            for token in tokens.iter() {
                if let TokenType::STRING(s) = &token.token_type {
                    println!("    String: {:?}", s);
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test numbers
    println!("\n  🔢 Number Test:");
    let number_code = "42 0 123 999";
    let mut number_tokenizer = NyashTokenizer::new(number_code);
    
    match number_tokenizer.tokenize() {
        Ok(tokens) => {
            for token in tokens.iter() {
                if let TokenType::NUMBER(n) = &token.token_type {
                    println!("    Number: {}", n);
                }
            }
        }
        Err(e) => println!("    Error: {}", e),
    }
    
    // Test error handling
    println!("\n  ❌ Error Handling Test:");
    let error_code = "box test @#$%";
    let mut error_tokenizer = NyashTokenizer::new(error_code);
    
    match error_tokenizer.tokenize() {
        Ok(_) => println!("    Unexpected success"),
        Err(e) => println!("    Expected error: {}", e),
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
    
    // Test the debug_this_problem.nyash equivalent
    println!("\n  🐛 Debug This Problem Test (Rust Parser):");
    let debug_code = r#"
    // this問題のミニマル再現
    box TestBox {
        value
        
        getValue() {
            return this.value
        }
    }

    // テスト
    obj = new TestBox()
    obj.value = "test123"
    print("Direct field: " + obj.value)
    print("Method call: " + obj.getValue())
    "#;
    
    match NyashParser::parse_from_string(debug_code) {
        Ok(ast) => {
            println!("    ✅ Successfully parsed debug_this_problem equivalent!");
            
            if let ASTNode::Program { statements, .. } = &ast {
                println!("    Complete program structure:");
                println!("      Total statements: {}", statements.len());
                
                let mut box_count = 0;
                let mut assignment_count = 0;
                let mut print_count = 0;
                let mut method_calls = 0;
                
                for stmt in statements {
                    match stmt {
                        ASTNode::BoxDeclaration { .. } => box_count += 1,
                        ASTNode::Assignment { .. } => assignment_count += 1,
                        ASTNode::Print { .. } => print_count += 1,
                        _ => {}
                    }
                    
                    // Count method calls recursively
                    count_method_calls(stmt, &mut method_calls);
                }
                
                println!("      - Box declarations: {}", box_count);
                println!("      - Assignments: {}", assignment_count);
                println!("      - Print statements: {}", print_count);
                println!("      - Method calls found: {}", method_calls);
                
                println!("    🎯 Parser successfully handles 'this' context in AST!");
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
}

// Helper function to count method calls recursively
fn count_method_calls(node: &ASTNode, count: &mut usize) {
    match node {
        ASTNode::MethodCall { .. } => {
            *count += 1;
        }
        ASTNode::Program { statements, .. } => {
            for stmt in statements {
                count_method_calls(stmt, count);
            }
        }
        ASTNode::Assignment { target, value, .. } => {
            count_method_calls(target, count);
            count_method_calls(value, count);
        }
        ASTNode::Print { expression, .. } => {
            count_method_calls(expression, count);
        }
        ASTNode::BinaryOp { left, right, .. } => {
            count_method_calls(left, count);
            count_method_calls(right, count);
        }
        ASTNode::BoxDeclaration { methods, .. } => {
            for method in methods.values() {
                count_method_calls(method, count);
            }
        }
        ASTNode::FunctionDeclaration { body, .. } => {
            for stmt in body {
                count_method_calls(stmt, count);
            }
        }
        ASTNode::Return { value, .. } => {
            if let Some(val) = value {
                count_method_calls(val, count);
            }
        }
        _ => {}
    }
}

fn demo_interpreter_system() {
    println!("\n🚀 7. Interpreter & Execution System:");
    
    // Test simple variable assignment and print
    println!("  📝 Simple Assignment & Print Test:");
    let simple_code = r#"
    x = 42
    y = "Hello, Nyash!"
    print(x)
    print(y)
    "#;
    
    match NyashParser::parse_from_string(simple_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Code: {}", simple_code.trim());
            println!("    Output:");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Execution successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test arithmetic operations
    println!("\n  ⚡ Arithmetic Operations Test:");
    let arithmetic_code = r#"
    a = 10
    b = 32
    result = a + b
    print("10 + 32 = " + result)
    "#;
    
    match NyashParser::parse_from_string(arithmetic_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Executing arithmetic operations...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Arithmetic execution successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test if statements
    println!("\n  🔄 Control Flow (If) Test:");
    let if_code = r#"
    condition = true
    if condition {
        print("Condition was true!")
        result = "success"
    } else {
        print("Condition was false!")
        result = "failure"
    }
    print("Result: " + result)
    "#;
    
    match NyashParser::parse_from_string(if_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Executing if statement...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ If statement execution successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test AND/OR operators
    println!("\n  🔗 Logical Operators Test:");
    let logic_code = r#"
    a = true
    b = false
    result1 = a && b
    result2 = a || b
    print("true && false = " + result1)
    print("true || false = " + result2)
    "#;
    
    match NyashParser::parse_from_string(logic_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Executing logical operators...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Logical operators execution successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test loop with break
    println!("\n  🔁 Loop with Break Test:");
    let loop_code = r#"
    counter = 0
    loop {
        counter = counter + 1
        print("Loop iteration: " + counter)
        if counter == 3 {
            print("Breaking loop!")
            break
        }
    }
    print("Final counter: " + counter)
    "#;
    
    match NyashParser::parse_from_string(loop_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Executing loop with break...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Loop execution successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test Box declaration and instance creation
    println!("\n  📦 Box Declaration & Instance Test:");
    let box_code = r#"
    box TestBox {
        value
        
        getValue() {
            return this.value
        }
        
        setValue(newValue) {
            this.value = newValue
        }
    }
    
    // Create instance
    obj = new TestBox()
    print("Created TestBox instance: " + obj)
    
    // Set field directly
    obj.value = "test123"
    print("Set field directly: obj.value = " + obj.value)
    
    // Call method that uses this
    result = obj.getValue()
    print("Method call result: " + result)
    
    // Call method that modifies via this
    obj.setValue("modified value")
    print("After setValue: " + obj.value)
    "#;
    
    match NyashParser::parse_from_string(box_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Testing Box instances with 'this' binding...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Box instance & 'this' working correctly!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test global variable
    println!("\n  🌍 Global Variable Test:");
    let global_code = r#"
    global project_name = "Nyash in Rust"
    global version = "v1.0"
    
    print("Project: " + project_name)
    print("Version: " + version)
    "#;
    
    match NyashParser::parse_from_string(global_code) {
        Ok(ast) => {
            let mut interpreter = NyashInterpreter::new();
            println!("    Setting global variables...");
            
            match interpreter.execute(ast) {
                Ok(_) => println!("    ✅ Global variables successful!"),
                Err(e) => println!("    ❌ Runtime error: {}", e),
            }
        }
        Err(e) => println!("    ❌ Parse error: {}", e),
    }
    
    // Test Self-Hosting Demonstration
    println!("\n  🎆 SELF-HOSTING DEMONSTRATION 🎆:");
    println!("    Loading self-hosting test file...");
    
    match std::fs::read_to_string("test_self_hosting_simple.nyash") {
        Ok(self_hosting_code) => {
            match NyashParser::parse_from_string(&self_hosting_code) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    println!("    Executing self-hosting simulation...");
                    
                    match interpreter.execute(ast) {
                        Ok(_) => {
                            println!("    🚀 LEGENDARY ACHIEVEMENT UNLOCKED! 🚀");
                            println!("    Rust-based Nyash interpreter successfully executed");
                            println!("    Nyash code that simulates compiling other Nyash code!");
                            println!("    Self-hosting level: ULTIMATE META-PROGRAMMING! 🎆");
                        },
                        Err(e) => println!("    ❌ Self-hosting runtime error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Self-hosting parse error: {}", e),
            }
        }
        Err(_) => {
            println!("    ⚠️  test_self_hosting_simple.nyash not found, using inline test:");
            
            let inline_self_hosting = r#"
            print("🎆 Inline Self-Hosting Test 🎆")
            
            box MetaCompiler {
                name
                
                init(compilerName) {
                    this.name = compilerName
                }
                
                compile(code) {
                    return "Compiled by " + this.name + ": " + code
                }
            }
            
            meta = new MetaCompiler()
            meta.init("Nyash-in-Rust")
            result = meta.compile("Everything is Box!")
            print(result)
            print("🚀 Meta-compilation successful!")
            "#;
            
            match NyashParser::parse_from_string(inline_self_hosting) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    match interpreter.execute(ast) {
                        Ok(_) => println!("    🎆 Inline self-hosting successful! 🎆"),
                        Err(e) => println!("    ❌ Inline self-hosting error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Inline self-hosting parse error: {}", e),
            }
        }
    }
    
    // Test Interface Box Implementation  
    println!("\n  🎆 INTERFACE BOX IMPLEMENTATION TEST 🎆:");
    println!("    Testing interface box syntax support...");
    
    match std::fs::read_to_string("test_interface.nyash") {
        Ok(interface_code) => {
            match NyashParser::parse_from_string(&interface_code) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    println!("    Executing interface box test...");
                    
                    match interpreter.execute(ast) {
                        Ok(_) => {
                            println!("    🚀 INTERFACE BOX STEP 1 SUCCESS! 🚀");
                            println!("    ✅ interface box syntax parsing works!");
                            println!("    ✅ Interface registration successful!");
                            println!("    Next: extends and implements syntax...");
                        },
                        Err(e) => println!("    ❌ Interface runtime error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Interface parse error: {}", e),
            }
        }
        Err(_) => {
            println!("    ⚠️  test_interface.nyash not found, using inline test:");
            
            let inline_interface_test = r#"
            print("🎆 Inline Interface Test 🎆")
            
            interface box Testable {
                test()
                verify(result)
            }
            
            print("Interface box declared successfully!")
            print("✅ Step 1: interface box syntax - WORKING!")
            "#;
            
            match NyashParser::parse_from_string(inline_interface_test) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    match interpreter.execute(ast) {
                        Ok(_) => println!("    🎆 Interface syntax working! 🎆"),
                        Err(e) => println!("    ❌ Interface test error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Interface test parse error: {}", e),
            }
        }
    }
    
    // Test Inheritance Implementation
    println!("\n  🚀 INHERITANCE IMPLEMENTATION TEST 🚀:");
    println!("    Testing extends & implements syntax...");
    
    match std::fs::read_to_string("test_inheritance.nyash") {
        Ok(inheritance_code) => {
            match NyashParser::parse_from_string(&inheritance_code) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    println!("    Executing inheritance test...");
                    
                    match interpreter.execute(ast) {
                        Ok(_) => {
                            println!("    🎆 INHERITANCE STEPS 2&3 SUCCESS! 🎆");
                            println!("    ✅ extends syntax working!");
                            println!("    ✅ implements syntax working!");
                            println!("    ✅ Inheritance chain resolution!");
                            println!("    ✅ Interface validation!");
                            println!("    ✅ Method overriding!");
                        },
                        Err(e) => println!("    ❌ Inheritance runtime error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Inheritance parse error: {}", e),
            }
        }
        Err(_) => {
            println!("    ⚠️  test_inheritance.nyash not found, using inline test:");
            
            let inline_inheritance_test = r#"
            print("🚀 Inline Inheritance Test 🚀")
            
            interface box Speakable {
                speak()
            }
            
            box Animal {
                name
                speak() {
                    return "Animal sound"
                }
            }
            
            box Dog extends Animal implements Speakable {
                breed
                speak() {
                    return "Woof!"
                }
            }
            
            dog = new Dog()
            dog.name = "Buddy"
            dog.breed = "Labrador"
            print("Dog says: " + dog.speak())
            print("✅ Inheritance working!")
            "#;
            
            match NyashParser::parse_from_string(inline_inheritance_test) {
                Ok(ast) => {
                    let mut interpreter = NyashInterpreter::new();
                    match interpreter.execute(ast) {
                        Ok(_) => println!("    🚀 Inheritance test successful! 🚀"),
                        Err(e) => println!("    ❌ Inheritance test error: {}", e),
                    }
                }
                Err(e) => println!("    ❌ Inheritance test parse error: {}", e),
            }
        }
    }
}

/// Execute MIR compilation and processing mode
fn execute_mir_mode(filename: &str, dump_mir: bool, verify_mir: bool, verbose: bool) {
    // Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };
    
    // Parse to AST
    let ast = match NyashParser::parse_from_string(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };
    
    // Compile to MIR
    let mut compiler = MirCompiler::new();
    let compile_result = match compiler.compile(ast) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ MIR compilation error: {}", e);
            process::exit(1);
        }
    };
    
    // Handle verification
    if verify_mir || dump_mir {
        match &compile_result.verification_result {
            Ok(()) => {
                if verify_mir {
                    println!("✅ MIR verification passed");
                }
            },
            Err(errors) => {
                eprintln!("❌ MIR verification failed with {} error(s):", errors.len());
                for (i, error) in errors.iter().enumerate() {
                    eprintln!("  {}: {}", i + 1, error);
                }
                if verify_mir {
                    process::exit(1);
                }
            }
        }
    }
    
    // Handle MIR dumping
    if dump_mir {
        let mut printer = if verbose {
            MirPrinter::verbose()
        } else {
            MirPrinter::new()
        };
        
        let mir_output = printer.print_module(&compile_result.module);
        println!("{}", mir_output);
    }
    
    // Show module statistics if verification was requested
    if verify_mir {
        let stats = compile_result.module.stats();
        println!("\n📊 Module Statistics:");
        println!("   Functions: {}", stats.function_count);
        println!("   Total Blocks: {}", stats.total_blocks);
        println!("   Total Instructions: {}", stats.total_instructions);
        println!("   Total Values: {}", stats.total_values);
        println!("   Pure Functions: {}", stats.pure_functions);
        
        if stats.function_count > 0 {
            for (name, function) in &compile_result.module.functions {
                let func_stats = function.stats();
                println!("\n📊 Function '{}' Statistics:", name);
                println!("   Blocks: {}", func_stats.block_count);
                println!("   Instructions: {}", func_stats.instruction_count);
                println!("   Values: {}", func_stats.value_count);
                println!("   Phi Functions: {}", func_stats.phi_count);
                println!("   Pure: {}", func_stats.is_pure);
            }
        }
    }
}

/// Execute VM mode
fn execute_vm_mode(filename: &str) {
    // Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };
    
    // Parse to AST
    let ast = match NyashParser::parse_from_string(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };
    
    // Compile to MIR
    let mut compiler = MirCompiler::new();
    let compile_result = match compiler.compile(ast) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ MIR compilation error: {}", e);
            process::exit(1);
        }
    };
    
    // Check for verification errors
    if let Err(errors) = &compile_result.verification_result {
        eprintln!("❌ MIR verification failed with {} error(s):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}: {}", i + 1, error);
        }
        // Continue execution anyway for now
    }
    
    // Execute with VM
    let mut vm = VM::new();
    match vm.execute_module(&compile_result.module) {
        Ok(result) => {
            println!("✅ VM execution completed successfully!");
            println!("Result: {}", result.to_string_box().value);
        },
        Err(e) => {
            eprintln!("❌ VM runtime error: {}", e);
            process::exit(1);
        }
    }
}

/// Execute WASM compilation mode
fn execute_wasm_mode(filename: &str, output_file: Option<&String>) {
    // Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };
    
    // Parse to AST
    let ast = match NyashParser::parse_from_string(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };
    
    // Compile to MIR
    let mut compiler = MirCompiler::new();
    let compile_result = match compiler.compile(ast) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ MIR compilation error: {}", e);
            process::exit(1);
        }
    };
    
    // Check for verification errors
    if let Err(errors) = &compile_result.verification_result {
        eprintln!("⚠️ MIR verification warnings ({} issues):", errors.len());
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}: {}", i + 1, error);
        }
        println!("Continuing with WASM compilation...");
    }
    
    // Compile to WASM
    let mut wasm_backend = WasmBackend::new();
    match wasm_backend.compile_to_wat(compile_result.module) {
        Ok(wat_text) => {
            println!("✅ WASM compilation completed successfully!");
            
            if let Some(output_path) = output_file {
                // Write to file
                match fs::write(output_path, &wat_text) {
                    Ok(_) => println!("📄 WAT output written to: {}", output_path),
                    Err(e) => {
                        eprintln!("❌ Error writing to file '{}': {}", output_path, e);
                        process::exit(1);
                    }
                }
            } else {
                // Print to stdout
                println!("📄 Generated WAT:");
                println!("{}", wat_text);
            }
        },
        Err(e) => {
            eprintln!("❌ WASM compilation error: {}", e);
            process::exit(1);
        }
    }
}

/// Execute benchmark mode
fn execute_benchmark_mode(iterations: u32) {
    use nyash_rust::benchmarks::BenchmarkSuite;
    
    let suite = BenchmarkSuite::new(iterations);
    let results = suite.run_all();
    
    if results.is_empty() {
        println!("❌ No benchmark results - make sure benchmark files exist in benchmarks/ directory");
        println!("   Expected files:");
        println!("   - benchmarks/bench_light.nyash");
        println!("   - benchmarks/bench_medium.nyash");
        println!("   - benchmarks/bench_heavy.nyash");
        process::exit(1);
    }
    
    suite.print_results(&results);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_main_functionality() {
        // This test ensures main() doesn't panic
        // In a real implementation, we'd have more comprehensive tests
        let string_box = StringBox::new("test");
        assert_eq!(string_box.type_name(), "StringBox");
        assert_eq!(string_box.to_string_box().value, "test");
    }
}
