/*! 
 * Nyash Programming Language - Rust Implementation Library
 * 
 * Everything is Box philosophy implemented in memory-safe Rust
 */

// 🌐 WebAssembly support
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod box_trait;
pub mod boxes;
// pub mod stdlib;
pub mod environment;
pub mod tokenizer;
pub mod ast;  // Using old ast.rs for now
pub mod parser;  // Using old parser.rs for now
pub mod interpreter;
pub mod instance;
pub mod channel_box;
pub mod finalization;
pub mod exception_box;
pub mod method_box;
pub mod type_box;  // 🌟 TypeBox revolutionary system
pub mod operator_traits; // 🚀 Rust-style trait-based operator overloading
pub mod box_operators; // 🚀 Operator implementations for basic Box types

// 🌐 P2P Communication Infrastructure (NEW!)
pub mod messaging;
pub mod transport;

#[cfg(target_arch = "wasm32")]
pub mod wasm_test;

#[cfg(test)]
pub mod tests;

// Re-export main types for easy access
pub use box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox};
pub use environment::{Environment, PythonCompatEnvironment};
pub use tokenizer::{NyashTokenizer, TokenType, Token};
pub use type_box::{TypeBox, TypeRegistry, MethodSignature};  // 🌟 TypeBox exports
pub use ast::{ASTNode, BinaryOperator, LiteralValue};
pub use parser::{NyashParser, ParseError};
pub use interpreter::{NyashInterpreter, RuntimeError};
pub use instance::InstanceBox;
pub use channel_box::{ChannelBox, MessageBox};
pub use boxes::math_box::{MathBox, FloatBox, RangeBox};
pub use boxes::time_box::{TimeBox, DateTimeBox, TimerBox};
pub use boxes::map_box::MapBox;
pub use boxes::random_box::RandomBox;
pub use boxes::sound_box::SoundBox;
pub use boxes::debug_box::DebugBox;
pub use boxes::console_box::ConsoleBox;
pub use method_box::{MethodBox, BoxType, FunctionDefinition, EphemeralInstance};
pub use boxes::null_box::{NullBox, null};

// Direct canvas test export
#[cfg(target_arch = "wasm32")]
pub use wasm_test::wasm_test::test_direct_canvas_draw;

// 🌐 WebAssembly exports for browser usage
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct NyashWasm {
    interpreter: NyashInterpreter,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NyashWasm {
    /// Create a new Nyash interpreter instance for browser use
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Setup panic handling for better browser debugging
        console_error_panic_hook::set_once();
        
        // Create interpreter with browser-specific setup
        let interpreter = NyashInterpreter::new();
        
        // Register browser-specific boxes
        // ConsoleBox is available as a constructor: console = new ConsoleBox()
        // TODO: Also register DOMBox, CanvasBox etc.
        
        Self { interpreter }
    }
    
    /// Evaluate Nyash code and return result as string
    #[wasm_bindgen]
    pub fn eval(&mut self, code: &str) -> String {
        // Handle empty or whitespace-only input
        let trimmed_code = code.trim();
        if trimmed_code.is_empty() {
            return String::new();
        }
        
        // Split multiline code into logical statements for better WASM handling
        let lines: Vec<&str> = trimmed_code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect();
        
        // If single line or looks like a complete static box/box definition, parse as-is
        if lines.len() == 1 || trimmed_code.contains("static box") || trimmed_code.contains("box ") {
            return self.eval_single_block(trimmed_code);
        }
        
        // For multiple lines, try to execute line by line
        let mut results = Vec::new();
        let mut accumulated_code = String::new();
        
        for line in lines {
            // Accumulate lines for block structures
            accumulated_code.push_str(line);
            accumulated_code.push('\n');
            
            // Check if we have a complete statement
            if self.is_complete_statement(&accumulated_code) {
                let result = self.eval_single_block(accumulated_code.trim());
                if result.starts_with("Parse Error:") {
                    return result; // Stop on parse error
                }
                if !result.is_empty() && result != "void" {
                    results.push(result);
                }
                accumulated_code.clear();
            }
        }
        
        // Execute any remaining accumulated code
        if !accumulated_code.trim().is_empty() {
            let result = self.eval_single_block(accumulated_code.trim());
            if !result.is_empty() && result != "void" {
                results.push(result);
            }
        }
        
        // Return the most relevant result
        results.into_iter()
            .filter(|r| !r.starts_with("Parse Error:") && !r.starts_with("Runtime Error:"))
            .last()
            .unwrap_or_else(|| "void".to_string())
    }
    
    /// Evaluate a single block of code
    fn eval_single_block(&mut self, code: &str) -> String {
        // First parse the code into an AST
        let ast = match NyashParser::parse_from_string(code) {
            Ok(ast) => ast,
            Err(e) => return format!("Parse Error: {}", e),
        };
        
        // Then execute the AST
        match self.interpreter.execute(ast) {
            Ok(result_box) => {
                // Format the result for browser display
                let result_str = result_box.to_string_box().value;
                if result_str == "void" || result_str.is_empty() {
                    "void".to_string()
                } else {
                    result_str
                }
            }
            Err(e) => format!("Runtime Error: {}", e),
        }
    }
    
    /// Check if code represents a complete statement (heuristic)
    fn is_complete_statement(&self, code: &str) -> bool {
        let trimmed = code.trim();
        
        // Always complete: assignments, function calls, simple expressions
        if trimmed.contains('=') && !trimmed.ends_with('=') {
            return true;
        }
        
        // Block structures need closing braces
        let open_braces = trimmed.chars().filter(|&c| c == '{').count();
        let close_braces = trimmed.chars().filter(|&c| c == '}').count();
        
        // Complete if braces are balanced or no braces at all
        open_braces == 0 || open_braces == close_braces
    }
    
    /// Get the current version info
    #[wasm_bindgen]
    pub fn version() -> String {
        String::from("Nyash WASM v0.1.0 - Everything is Box in Browser!")
    }
}