/*!
 Nyash Programming Language — Rust library crate.
 Provides parser, MIR, backends, runner, and supporting runtime.
*/

// Allow referring to this crate as `nyash_rust` from within the crate, matching external paths.
extern crate self as nyash_rust;

// WebAssembly support
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Legacy interpreter removed
mod interpreter_stub;

pub mod ast; // using historical ast.rs
pub mod box_arithmetic;
pub mod box_factory; // unified Box Factory
pub mod box_operators; // operator implementations for basic Box types
pub mod box_trait;
pub mod boxes;
pub mod channel_box;
pub mod core; // core models shared by backends
pub mod environment;
pub mod exception_box;
pub mod finalization;
pub mod instance_v2; // simplified InstanceBox implementation
pub mod interpreter { pub use crate::interpreter_stub::*; }
pub mod method_box;
pub mod operator_traits; // trait-based operator overloading
pub mod parser; // using historical parser.rs
pub mod scope_tracker; // Box lifecycle tracking for VM
pub mod stdlib;
pub mod tokenizer;
pub mod type_box; // TypeBox system (arithmetic moved from box_trait.rs)

pub mod value;

pub mod messaging;
pub mod transport;

// MIR (Mid-level Intermediate Representation)
pub mod mir;
#[cfg(feature = "aot-plan-import")]
pub mod mir_aot_plan_import {
    pub use crate::mir::aot_plan_import::*;
}

// Backends
pub mod backend;
pub mod jit; // Cranelift JIT subsystem (skeleton)
pub mod semantics; // Unified semantics trait for MIR evaluation/lowering

pub mod benchmarks;

// BID-FFI / Plugin system (prototype)
pub mod bid;

// Configuration system
pub mod config;

// CLI system
pub mod cli;

// Runtime system (plugins, registry, etc.)
pub mod debug;
pub mod runner_plugin_init;
pub mod runtime;
// Unified Grammar scaffolding
pub mod grammar;
pub mod syntax; // syntax sugar config and helpers
// Execution runner (CLI coordinator)
pub mod runner;

// Expose the macro engine module under a raw identifier; the source lives under `src/macro/`.
#[path = "macro/mod.rs"]
pub mod r#macro;

#[cfg(target_arch = "wasm32")]
pub mod wasm_test;

#[cfg(test)]
pub mod tests;

// Re-export main types for easy access
pub use ast::{ASTNode, BinaryOperator, LiteralValue};
pub use box_arithmetic::{AddBox, CompareBox, DivideBox, ModuloBox, MultiplyBox, SubtractBox};
pub use box_trait::{BoolBox, IntegerBox, NyashBox, StringBox, VoidBox};
pub use environment::{Environment, PythonCompatEnvironment};
#[cfg(feature = "interpreter-legacy")]
#[cfg(feature = "interpreter-legacy")]
pub use interpreter::{NyashInterpreter, RuntimeError};
pub use parser::{NyashParser, ParseError};
pub use tokenizer::{NyashTokenizer, Token, TokenType};
pub use type_box::{MethodSignature, TypeBox, TypeRegistry}; // 🌟 TypeBox exports
                                                            // pub use instance::InstanceBox;  // 旧実装
pub use boxes::console_box::ConsoleBox;
pub use boxes::debug_box::DebugBox;
pub use boxes::map_box::MapBox;
pub use boxes::math_box::{FloatBox, MathBox, RangeBox};
pub use boxes::null_box::{null, NullBox};
pub use boxes::random_box::RandomBox;
pub use boxes::sound_box::SoundBox;
pub use boxes::time_box::{DateTimeBox, TimeBox, TimerBox};
pub use channel_box::{ChannelBox, MessageBox};
pub use instance_v2::InstanceBox; // 🎯 新実装テスト（nyash_rustパス使用）
pub use method_box::{BoxType, EphemeralInstance, FunctionDefinition, MethodBox};

pub use value::NyashValue;

// Direct canvas test export
#[cfg(all(target_arch = "wasm32", feature = "interpreter-legacy"))]
pub use wasm_test::wasm_test::test_direct_canvas_draw;

// WebAssembly exports for browser usage
#[cfg(all(target_arch = "wasm32", feature = "interpreter-legacy"))]
#[wasm_bindgen]
pub struct NyashWasm {
    interpreter: NyashInterpreter,
}

#[cfg(all(target_arch = "wasm32", feature = "interpreter-legacy"))]
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
        let lines: Vec<&str> = trimmed_code
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect();

        // If single line or looks like a complete static box/box definition, parse as-is
        if lines.len() == 1 || trimmed_code.contains("static box") || trimmed_code.contains("box ")
        {
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
        results
            .into_iter()
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
