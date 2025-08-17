/*!
 * Nyash Interpreter - Modular Rust Implementation
 * 
 * Refactored from massive 2,633-line interpreter.rs into logical modules
 * Everything is Box philosophy with clean separation of concerns
 */

// Import all necessary dependencies
use crate::ast::{ASTNode, CatchClause};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, ArrayBox, ResultBox, ErrorBox, BoxCore};
#[cfg(not(feature = "dynamic-file"))]
use crate::box_trait::FileBox;
use crate::boxes::FutureBox;
use crate::instance::InstanceBox;
use crate::channel_box::ChannelBox;
use crate::boxes::math_box::{MathBox, RangeBox};
use crate::boxes::time_box::{TimeBox, TimerBox};
use crate::boxes::map_box::MapBox;
use crate::boxes::random_box::RandomBox;
use crate::boxes::sound_box::SoundBox;
use crate::boxes::debug_box::DebugBox;
use crate::method_box::MethodBox;

// WASM-specific Box types (conditionally included)
#[cfg(target_arch = "wasm32")]
use crate::boxes::web::{WebDisplayBox, WebConsoleBox, WebCanvasBox};
use crate::finalization;
use crate::exception_box;
use std::collections::HashMap;

// Module declarations  
mod async_methods;
mod box_methods;
mod core;
mod expressions;
mod statements;
mod functions;
mod objects;
mod io;
mod methods;
mod math_methods;
mod system_methods;
mod web_methods;
mod special_methods;
#[cfg(feature = "dynamic-file")]
mod plugin_loader;

// Main interpreter implementation - will be moved from interpreter.rs
pub use core::NyashInterpreter;

// Dynamic plugin support
#[cfg(feature = "dynamic-file")]
pub use plugin_loader::{PluginLoader, FileBoxProxy};


/// 実行制御フロー
#[derive(Debug)]
pub enum ControlFlow {
    None,
    Break,
    Return(Box<dyn NyashBox>),
    Throw(Box<dyn NyashBox>),
}

/// コンストラクタ実行コンテキスト
#[derive(Debug, Clone)]
pub struct ConstructorContext {
    pub class_name: String,
    pub parent_class: Option<String>,
}

/// Box宣言を保持する構造体
#[derive(Debug, Clone)]
pub struct BoxDeclaration {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: HashMap<String, ASTNode>,
    pub constructors: HashMap<String, ASTNode>,
    pub init_fields: Vec<String>,
    pub weak_fields: Vec<String>,  // 🔗 weak修飾子が付いたフィールドのリスト
    pub is_interface: bool,
    pub extends: Vec<String>,  // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
    pub implements: Vec<String>,
    pub type_parameters: Vec<String>,  // 🔥 ジェネリクス型パラメータ
}

/// 🔥 Static Box定義を保持する構造体
#[derive(Debug, Clone)]
pub struct StaticBoxDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: HashMap<String, ASTNode>,
    pub init_fields: Vec<String>,
    pub weak_fields: Vec<String>,  // 🔗 weak修飾子が付いたフィールドのリスト
    pub static_init: Option<Vec<ASTNode>>,  // static { } ブロック
    pub extends: Vec<String>,  // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
    pub implements: Vec<String>,
    pub type_parameters: Vec<String>,
    /// 初期化状態
    pub initialization_state: StaticBoxState,
}

/// 🔥 Static Box初期化状態
#[derive(Debug, Clone, PartialEq)]
pub enum StaticBoxState {
    NotInitialized,     // 未初期化
    Initializing,       // 初期化中（循環参照検出用）
    Initialized,        // 初期化完了
}

/// 関数宣言を保持する構造体
#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<ASTNode>,
}

// Re-export core interpreter types
pub use core::*;

// Import and re-export stdlib for interpreter modules  
pub use crate::stdlib::BuiltinStdlib;