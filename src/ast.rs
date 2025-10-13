/*!
 * Nyash AST (Abstract Syntax Tree) - Rust Implementation
 *
 * Python版nyashc_v4.pyのAST構造をRustで完全再実装
 * Everything is Box哲学に基づく型安全なAST設計
 */

use crate::box_trait::NyashBox;
use std::collections::HashMap;
use std::fmt;
mod span;
pub use span::Span;
mod utils;
mod nodes;
pub use nodes::*;

// Span は src/ast/span.rs へ分離（re-export で後方互換維持）

/// 🌟 AST分類システム - ChatGPTアドバイス統合による3層アーキテクチャ
/// Structure/Expression/Statement の明確な分離による型安全性向上

/// ASTノードの種類分類
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNodeType {
    Structure,  // 構造定義: box, function, if, loop, try/catch
    Expression, // 式: リテラル, 変数, 演算, 呼び出し
    Statement,  // 文: 代入, return, break, include
}

/// 構造ノード - 言語の基本構造を定義
#[derive(Debug, Clone)]
pub enum StructureNode {
    BoxDeclaration {
        name: String,
        fields: Vec<String>,
        methods: Vec<ASTNode>,
        constructors: Vec<ASTNode>,
        init_fields: Vec<String>,
        weak_fields: Vec<String>, // 🔗 weak修飾子が付いたフィールドのリスト
        is_interface: bool,
        extends: Vec<String>, // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
        implements: Vec<String>,
        /// 🔥 ジェネリクス型パラメータ (例: ["T", "U"])
        type_parameters: Vec<String>,
        /// 🔥 Static boxかどうかのフラグ
        is_static: bool,
        /// 🔥 Static初期化ブロック (static { ... })
        static_init: Option<Vec<ASTNode>>,
        span: Span,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
        is_static: bool,   // 🔥 静的メソッドフラグ
        is_override: bool, // 🔥 オーバーライドフラグ
        span: Span,
    },
    IfStructure {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
        span: Span,
    },
    LoopStructure {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
        span: Span,
    },
    TryCatchStructure {
        try_body: Vec<ASTNode>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<ASTNode>>,
        span: Span,
    },
}

/// 式ノード - 値を生成する表現
#[derive(Debug, Clone)]
pub enum ExpressionNode {
    Literal {
        value: LiteralValue,
        span: Span,
    },
    Variable {
        name: String,
        span: Span,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        span: Span,
    },
    UnaryOperation {
        operator: UnaryOperator,
        operand: Box<ASTNode>,
        span: Span,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
        span: Span,
    },
    MethodCall {
        object: Box<ASTNode>,
        method: String,
        arguments: Vec<ASTNode>,
        span: Span,
    },
    FieldAccess {
        object: Box<ASTNode>,
        field: String,
        span: Span,
    },
    NewExpression {
        class: String,
        arguments: Vec<ASTNode>,
        /// 🔥 ジェネリクス型引数 (例: ["IntegerBox", "StringBox"])
        type_arguments: Vec<String>,
        span: Span,
    },
    ThisExpression {
        span: Span,
    },
    MeExpression {
        span: Span,
    },
    /// match式: match <expr> { lit => expr, ... else => expr }
    MatchExpr {
        scrutinee: Box<ASTNode>,
        arms: Vec<(LiteralValue, ASTNode)>,
        else_expr: Box<ASTNode>,
        span: Span,
    },
    // (Stage‑2 sugar for literals is represented in unified ASTNode, not here)
}

/// 文ノード - 実行可能なアクション  
#[derive(Debug, Clone)]
pub enum StatementNode {
    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
        span: Span,
    },
    Print {
        expression: Box<ASTNode>,
        span: Span,
    },
    Return {
        value: Option<Box<ASTNode>>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Include {
        filename: String,
        span: Span,
    },
    Local {
        variables: Vec<String>,
        span: Span,
    },
    Throw {
        exception_type: String,
        message: Box<ASTNode>,
        span: Span,
    },
    Expression {
        expr: Box<ASTNode>,
        span: Span,
    },
}

/// Catch節の構造体
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub exception_type: Option<String>, // None = catch-all
    pub variable_name: Option<String>,  // 例外を受け取る変数名
    pub body: Vec<ASTNode>,             // catch本体
    pub span: Span,                     // ソースコード位置
}

/// Enum variant for @enum macro - e.g., Ok(value), None
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,           // Variant name (e.g., "Ok", "Some", "None")
    pub fields: Vec<String>,    // Field names (empty for None, single for Ok(value))
    pub span: Span,
}

/// リテラル値の型 (Clone可能)
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Integer(i64),
    Float(f64), // 浮動小数点数サポート追加
    Bool(bool),
    Null, // null値
    Void,
}

impl LiteralValue {
    /// LiteralValueをNyashBoxに変換
    pub fn to_nyash_box(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::{BoolBox, IntegerBox, StringBox, VoidBox};
        #[cfg(feature = "legacy-boxes")]
        use crate::boxes::FloatBox;

        match self {
            LiteralValue::String(s) => Box::new(StringBox::new(s)),
            LiteralValue::Integer(i) => Box::new(IntegerBox::new(*i)),
            #[cfg(feature = "legacy-boxes")]
            LiteralValue::Float(f) => Box::new(FloatBox::new(*f)),
            #[cfg(not(feature = "legacy-boxes"))]
            LiteralValue::Float(f) => Box::new(StringBox::new(f.to_string())),
            LiteralValue::Bool(b) => Box::new(BoolBox::new(*b)),
            // Represent Null as Void to avoid legacy NullBox dependency in plugin-only builds
            LiteralValue::Null => Box::new(VoidBox::new()),
            LiteralValue::Void => Box::new(VoidBox::new()),
        }
    }

    /// NyashBoxからLiteralValueに変換
    pub fn from_nyash_box(box_val: &dyn NyashBox) -> Option<LiteralValue> {
        use crate::box_trait::{BoolBox, IntegerBox, StringBox, VoidBox};
        #[cfg(feature = "legacy-boxes")]
        use crate::boxes::FloatBox;
        #[allow(unused_imports)] use std::any::Any;

        if let Some(string_box) = box_val.as_any().downcast_ref::<StringBox>() {
            return Some(LiteralValue::String(string_box.value.clone()));
        }
        if let Some(int_box) = box_val.as_any().downcast_ref::<IntegerBox>() {
            return Some(LiteralValue::Integer(int_box.value));
        }
        #[cfg(feature = "legacy-boxes")]
        if let Some(float_box) = box_val.as_any().downcast_ref::<FloatBox>() {
            return Some(LiteralValue::Float(float_box.value));
        }
        if let Some(bool_box) = box_val.as_any().downcast_ref::<BoolBox>() {
            return Some(LiteralValue::Bool(bool_box.value));
        }
        if box_val.as_any().downcast_ref::<VoidBox>().is_some() {
            // Treat Void as Null on the AST roundtrip path (plugin-only compatibility)
            return Some(LiteralValue::Null);
        }
        None
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::Integer(i) => write!(f, "{}", i),
            LiteralValue::Float(fl) => write!(f, "{}", fl),
            LiteralValue::Bool(b) => write!(f, "{}", b),
            LiteralValue::Null => write!(f, "null"),
            LiteralValue::Void => write!(f, "void"),
        }
    }
}

/// 単項演算子の種類
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,  // -x
    Not,    // not x / !x
    BitNot, // ~x
}

/// 二項演算子の種類
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    BitAnd,
    BitOr,
    BitXor,
    Shl, // << shift-left (Phase 1)
    Shr,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            UnaryOperator::Minus => "-",
            UnaryOperator::Not => "not",
            UnaryOperator::BitNot => "~",
        };
        write!(f, "{}", symbol)
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::BitAnd => "&",
            BinaryOperator::BitOr => "|",
            BinaryOperator::BitXor => "^",
            BinaryOperator::Shl => "<<",
            BinaryOperator::Shr => ">>",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
        };
        write!(f, "{}", symbol)
    }
}

/// AST Node - Everything is Box哲学に基づく統一構造
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    /// プログラム全体 - 文のリスト
    Program {
        statements: Vec<ASTNode>,
        span: Span,
    },

    // ===== 文 (Statements) =====
    /// 代入文: target = value
    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
        span: Span,
    },

    /// print文: print(expression)
    Print {
        expression: Box<ASTNode>,
        span: Span,
    },

    /// if文: if condition { then_body } else { else_body }
    If {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
        span: Span,
    },

    /// loop文: loop(condition) { body } のみ
    Loop {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
        span: Span,
    },

    /// return文: return value
    Return {
        value: Option<Box<ASTNode>>,
        span: Span,
    },

    /// break文
    Break { span: Span },
    /// continue文
    Continue { span: Span },

    /// using文: using namespace_name
    UsingStatement { namespace_name: String, span: Span },
    /// import文: import "path" (as Alias)?
    ImportStatement {
        path: String,
        alias: Option<String>,
        span: Span,
    },

    /// nowait文: nowait variable = expression
    Nowait {
        variable: String,
        expression: Box<ASTNode>,
        span: Span,
    },

    /// await式: await expression
    AwaitExpression {
        expression: Box<ASTNode>,
        span: Span,
    },

    /// result伝播: expr? （ResultBoxなら isOk/getValue or 早期return）
    QMarkPropagate {
        expression: Box<ASTNode>,
        span: Span,
    },

    /// match式: match <expr> { lit => expr, ... else => expr }
    MatchExpr {
        scrutinee: Box<ASTNode>,
        arms: Vec<(LiteralValue, ASTNode)>,
        else_expr: Box<ASTNode>,
        span: Span,
    },
    /// 配列リテラル（糖衣）: [e1, e2, ...]
    ArrayLiteral { elements: Vec<ASTNode>, span: Span },
    /// マップリテラル（糖衣）: { "k": v, ... } （Stage‑2: 文字列キー限定）
    MapLiteral {
        entries: Vec<(String, ASTNode)>,
        span: Span,
    },

    /// 無名関数（最小P1: 値としてのみ。呼び出しは未対応）
    Lambda {
        params: Vec<String>,
        body: Vec<ASTNode>,
        span: Span,
    },

    /// arrow文: (sender >> receiver).method(args)
    Arrow {
        sender: Box<ASTNode>,
        receiver: Box<ASTNode>,
        span: Span,
    },

    /// try/catch/finally文: try { ... } catch (Type e) { ... } finally { ... }
    TryCatch {
        try_body: Vec<ASTNode>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<ASTNode>>,
        span: Span,
    },

    /// throw文: throw expression
    Throw {
        expression: Box<ASTNode>,
        span: Span,
    },

    // ===== 宣言 (Declarations) =====
    /// box宣言: box Name { fields... methods... }
    BoxDeclaration {
        name: String,
        fields: Vec<String>,
        /// 公開フィールド（public { ... }）
        public_fields: Vec<String>,
        /// 非公開フィールド（private { ... }）
        private_fields: Vec<String>,
        methods: HashMap<String, ASTNode>, // method_name -> FunctionDeclaration
        constructors: HashMap<String, ASTNode>, // constructor_key -> FunctionDeclaration
        init_fields: Vec<String>,          // initブロック内のフィールド定義
        weak_fields: Vec<String>,          // 🔗 weak修飾子が付いたフィールドのリスト
        is_interface: bool,                // interface box かどうか
        extends: Vec<String>, // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
        implements: Vec<String>, // 実装するinterface名のリスト
        type_parameters: Vec<String>, // 🔥 ジェネリクス型パラメータ (例: ["T", "U"])
        /// 🔥 Static boxかどうかのフラグ
        is_static: bool,
        /// 🔥 Static初期化ブロック (static { ... })
        static_init: Option<Vec<ASTNode>>,
        span: Span,
    },

    /// 関数宣言: functionName(params) { body }
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
        is_static: bool,   // 🔥 静的メソッドフラグ
        is_override: bool, // 🔥 オーバーライドフラグ
        span: Span,
    },

    /// @enum macro declaration: @enum Name { Variant1(field) Variant2 ... }
    /// Generates EnumNameBox + static box EnumName with constructors
    EnumDeclaration {
        name: String,
        variants: Vec<EnumVariant>,
        span: Span,
    },

    /// グローバル変数: global name = value
    GlobalVar {
        name: String,
        value: Box<ASTNode>,
        span: Span,
    },

    // ===== 式 (Expressions) =====
    /// リテラル値: "string", 42, true, etc
    Literal { value: LiteralValue, span: Span },

    /// 変数参照: variableName
    Variable { name: String, span: Span },

    /// 単項演算: operator operand
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<ASTNode>,
        span: Span,
    },

    /// 二項演算: left operator right
    BinaryOp {
        operator: BinaryOperator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        span: Span,
    },

    /// メソッド呼び出し: object.method(arguments)
    MethodCall {
        object: Box<ASTNode>,
        method: String,
        arguments: Vec<ASTNode>,
        span: Span,
    },

    /// フィールドアクセス: object.field
    FieldAccess {
        object: Box<ASTNode>,
        field: String,
        span: Span,
    },

    /// コンストラクタ呼び出し: new ClassName(arguments)
    New {
        class: String,
        arguments: Vec<ASTNode>,
        type_arguments: Vec<String>, // 🔥 ジェネリクス型引数 (例: ["IntegerBox", "StringBox"])
        span: Span,
    },

    /// this参照
    This { span: Span },

    /// me参照
    Me { span: Span },

    /// 🔥 from呼び出し: from Parent.method(arguments) or from Parent.constructor(arguments)
    FromCall {
        parent: String,          // Parent名
        method: String,          // method名またはconstructor
        arguments: Vec<ASTNode>, // 引数
        span: Span,
    },

    /// thisフィールドアクセス: this.field
    ThisField { field: String, span: Span },

    /// meフィールドアクセス: me.field
    MeField { field: String, span: Span },


    /// ローカル変数宣言: local x, y, z
    Local {
        variables: Vec<String>,
        /// 初期化値（変数と同じ順序、Noneは初期化なし）
        initial_values: Vec<Option<Box<ASTNode>>>,
        span: Span,
    },

    /// ScopeBox（オプション）: 診断/マクロ可視性のためのno-opスコープ。
    /// 正規化で注入され、MIRビルダがブロックとして処理（意味不変）。
    ScopeBox {
        body: Vec<ASTNode>,
        span: Span,
    },

    /// Outbox変数宣言: outbox x, y, z (static関数内専用)
    Outbox {
        variables: Vec<String>,
        /// 初期化値（変数と同じ順序、Noneは初期化なし）
        initial_values: Vec<Option<Box<ASTNode>>>,
        span: Span,
    },

    /// 関数呼び出し: functionName(arguments)
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
        span: Span,
    },

    /// 一般式呼び出し: (callee)(arguments)
    Call {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
        span: Span,
    },
}

// Tests moved to integration tests to keep this file lean
