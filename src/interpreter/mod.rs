/*!
 * Nyash Interpreter - Modular Rust Implementation
 * 
 * Refactored from massive 2,633-line interpreter.rs into logical modules
 * Everything is Box philosophy with clean separation of concerns
 */

// Import all necessary dependencies
use crate::ast::{ASTNode, BinaryOperator, CatchClause};
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox, SubtractBox, MultiplyBox, DivideBox, CompareBox, ArrayBox, FileBox, ResultBox, ErrorBox, FutureBox};
use crate::instance::InstanceBox;
use crate::channel_box::{ChannelBox, MessageBox};
use crate::boxes::math_box::{MathBox, FloatBox, RangeBox};
use crate::boxes::time_box::{TimeBox, DateTimeBox, TimerBox};
use crate::boxes::map_box::MapBox;
use crate::boxes::random_box::RandomBox;
use crate::boxes::sound_box::SoundBox;
use crate::boxes::debug_box::DebugBox;
use crate::method_box::MethodBox;
use crate::finalization;
use crate::exception_box;
use std::collections::HashMap;

// Module declarations  
mod box_methods;
mod core;
mod expressions;
mod statements;
mod functions;
mod objects;
mod io;

// Main interpreter implementation - will be moved from interpreter.rs


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
    pub is_interface: bool,
    pub extends: Option<String>,
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
    pub static_init: Option<Vec<ASTNode>>,  // static { } ブロック
    pub extends: Option<String>,
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