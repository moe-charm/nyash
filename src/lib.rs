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
// pub mod interpreter removed - legacy interpreter deleted
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
// pub mod jit; // ARCHIVED: Cranelift JIT subsystem moved to archive/jit-cranelift/
pub mod jit_stub; // Temporary JIT stub for Phase 15 compilation compatibility
pub use jit_stub as jit; // Alias for compatibility
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
pub mod using; // using resolver scaffolding (Phase 15)

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
pub use box_factory::RuntimeError;
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

// WASM support temporarily disabled - legacy interpreter removed
// TODO: Implement WASM support using VM or LLVM backends
