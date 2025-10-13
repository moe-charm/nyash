/*!
 Nyash Programming Language — Rust library crate.
 Provides parser, MIR, backends, runner, and supporting runtime.
*/

// Allow referring to this crate as `nyash_rust` from within the crate, matching external paths.
extern crate self as nyash_rust;

// WebAssembly support
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod ast; // using historical ast.rs
pub mod box_arithmetic;
pub mod box_core; // Core trait definitions (split from box_trait)
pub mod box_factory; // unified Box Factory
pub mod box_operators; // operator implementations for basic Box types
pub mod box_registry; // Built-in box registry (split from box_trait)
pub mod box_trait; // Re-exports for backward compatibility
#[cfg(feature = "legacy-boxes")]
pub mod boxes;
#[cfg(not(feature = "legacy-boxes"))]
pub mod compat {
    pub mod basic_box_shim;
}
pub mod channel_box;
pub mod core; // core models shared by backends
pub mod environment;
pub mod exception_box;
pub mod finalization;
pub mod instance_v2; // simplified InstanceBox implementation
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
// JIT functionality archived to archive/jit-cranelift/
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
// VM ops families (thin boxes): compare/call/boxcall helpers
pub mod vm_ops;
// Unified Grammar scaffolding
pub mod grammar;
pub mod syntax; // syntax sugar config and helpers
// Execution runner (CLI coordinator)
pub mod runner;
pub mod guards;
pub mod using; // using resolver scaffolding (Phase 15)
pub mod common; // shared policies/helpers (builder/VM)

// Expose the macro engine module under a raw identifier; the source lives under `src/macro/`.
#[path = "macro/mod.rs"]
pub mod r#macro;

#[cfg(target_arch = "wasm32")]
pub mod wasm_test;

#[cfg(test)]
pub mod tests;

// Re-export main types for easy access
pub use ast::{ASTNode, BinaryOperator, LiteralValue};
#[cfg(feature = "legacy-boxes")]
pub use box_arithmetic::{AddBox, CompareBox, DivideBox, ModuloBox, MultiplyBox, SubtractBox};
pub use box_trait::{BoolBox, IntegerBox, NyashBox, StringBox, VoidBox};
pub use environment::{Environment, PythonCompatEnvironment};
pub use box_factory::RuntimeError;
pub use parser::{NyashParser, ParseError};
pub use tokenizer::{NyashTokenizer, Token, TokenType};
pub use type_box::{MethodSignature, TypeBox, TypeRegistry}; // 🌟 TypeBox exports
#[cfg(feature = "legacy-boxes")]
pub use boxes::console_box::ConsoleBox;
#[cfg(feature = "legacy-boxes")]
pub use boxes::debug_box::DebugBox;
#[cfg(feature = "legacy-boxes")]
pub use boxes::map_box::MapBox;
#[cfg(feature = "legacy-boxes")]
pub use boxes::math_box::{FloatBox, MathBox, RangeBox};
#[cfg(feature = "legacy-boxes")]
pub use boxes::null_box::{null, NullBox};
#[cfg(feature = "legacy-boxes")]
pub use boxes::random_box::RandomBox;
#[cfg(feature = "legacy-boxes")]
pub use boxes::sound_box::SoundBox;
#[cfg(feature = "legacy-boxes")]
pub use boxes::time_box::{DateTimeBox, TimeBox, TimerBox};
pub use channel_box::{ChannelBox, MessageBox};
pub use instance_v2::InstanceBox; // 🎯 新実装テスト（nyash_rustパス使用）
pub use method_box::{BoxType, EphemeralInstance, FunctionDefinition, MethodBox};

pub use value::NyashValue;

// WASM support to be reimplemented with VM/LLVM backends
