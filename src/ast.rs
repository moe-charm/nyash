/*!
 * Nyash AST (Abstract Syntax Tree) - Rust Implementation
 * 
 * Python版nyashc_v4.pyのAST構造をRustで完全再実装
 * Everything is Box哲学に基づく型安全なAST設計
 */

use crate::box_trait::NyashBox;
use std::collections::HashMap;
use std::fmt;

/// ソースコード位置情報 - エラー報告とデバッグの革命
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,     // 開始位置（バイトオフセット）
    pub end: usize,       // 終了位置（バイトオフセット）
    pub line: usize,      // 行番号（1から開始）
    pub column: usize,    // 列番号（1から開始）
}

impl Span {
    /// 新しいSpanを作成
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
    
    /// デフォルトのSpan（不明な位置）
    pub fn unknown() -> Self {
        Self { start: 0, end: 0, line: 1, column: 1 }
    }
    
    /// 2つのSpanを結合（開始位置から終了位置まで）
    pub fn merge(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
    
    /// ソースコードから該当箇所を抽出してエラー表示用文字列を生成
    pub fn error_context(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        if self.line == 0 || self.line > lines.len() {
            return format!("line {}, column {}", self.line, self.column);
        }
        
        let line_content = lines[self.line - 1];
        let mut context = String::new();
        
        // 行番号とソース行を表示
        context.push_str(&format!("   |\n{:3} | {}\n", self.line, line_content));
        
        // カーソル位置を表示（簡易版）
        if self.column > 0 && self.column <= line_content.len() + 1 {
            context.push_str("   | ");
            for _ in 1..self.column {
                context.push(' ');
            }
            let span_length = if self.end > self.start { 
                (self.end - self.start).min(line_content.len() - self.column + 1)
            } else { 
                1 
            };
            for _ in 0..span_length.max(1) {
                context.push('^');
            }
            context.push('\n');
        }
        
        context
    }
    
    /// 位置情報の文字列表現
    pub fn location_string(&self) -> String {
        format!("line {}, column {}", self.line, self.column)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// 🌟 AST分類システム - ChatGPTアドバイス統合による3層アーキテクチャ
/// Structure/Expression/Statement の明確な分離による型安全性向上

/// ASTノードの種類分類
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNodeType {
    Structure,    // 構造定義: box, function, if, loop, try/catch
    Expression,   // 式: リテラル, 変数, 演算, 呼び出し
    Statement,    // 文: 代入, return, break, include
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
        weak_fields: Vec<String>,  // 🔗 weak修飾子が付いたフィールドのリスト
        is_interface: bool,
        extends: Vec<String>,  // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
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
        is_static: bool,     // 🔥 静的メソッドフラグ
        is_override: bool,   // 🔥 オーバーライドフラグ
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
    /// peek式: peek <expr> { lit => expr, ... else => expr }
    PeekExpr {
        scrutinee: Box<ASTNode>,
        arms: Vec<(LiteralValue, ASTNode)>,
        else_expr: Box<ASTNode>,
        span: Span,
    },
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
    pub exception_type: Option<String>,  // None = catch-all
    pub variable_name: Option<String>,   // 例外を受け取る変数名
    pub body: Vec<ASTNode>,             // catch本体
    pub span: Span,                     // ソースコード位置
}

/// リテラル値の型 (Clone可能)
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Integer(i64),
    Float(f64),  // 浮動小数点数サポート追加
    Bool(bool),
    Null,        // null値
    Void,
}

impl LiteralValue {
    /// LiteralValueをNyashBoxに変換
    pub fn to_nyash_box(&self) -> Box<dyn NyashBox> {
        use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
        use crate::boxes::FloatBox;
        
        match self {
            LiteralValue::String(s) => Box::new(StringBox::new(s)),
            LiteralValue::Integer(i) => Box::new(IntegerBox::new(*i)),
            LiteralValue::Float(f) => Box::new(FloatBox::new(*f)),
            LiteralValue::Bool(b) => Box::new(BoolBox::new(*b)),
            LiteralValue::Null => Box::new(crate::boxes::null_box::NullBox::new()),
            LiteralValue::Void => Box::new(VoidBox::new()),
        }
    }
    
    /// NyashBoxからLiteralValueに変換
    pub fn from_nyash_box(box_val: &dyn NyashBox) -> Option<LiteralValue> {
        #[allow(unused_imports)]
        use std::any::Any;
        use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
        use crate::boxes::FloatBox;
        
        if let Some(string_box) = box_val.as_any().downcast_ref::<StringBox>() {
            Some(LiteralValue::String(string_box.value.clone()))
        } else if let Some(int_box) = box_val.as_any().downcast_ref::<IntegerBox>() {
            Some(LiteralValue::Integer(int_box.value))
        } else if let Some(float_box) = box_val.as_any().downcast_ref::<FloatBox>() {
            Some(LiteralValue::Float(float_box.value))
        } else if let Some(bool_box) = box_val.as_any().downcast_ref::<BoolBox>() {
            Some(LiteralValue::Bool(bool_box.value))
        } else if box_val.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
            Some(LiteralValue::Null)
        } else if box_val.as_any().downcast_ref::<VoidBox>().is_some() {
            Some(LiteralValue::Void)
        } else {
            None
        }
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
    Not,    // not x
}

/// 二項演算子の種類
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract, 
    Multiply,
    Divide,
    Modulo,
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
    Break {
        span: Span,
    },
    /// continue文
    Continue {
        span: Span,
    },
    
    /// using文: using namespace_name
    UsingStatement {
        namespace_name: String,
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
    
    /// peek式: peek <expr> { lit => expr, ... else => expr }
    PeekExpr {
        scrutinee: Box<ASTNode>,
        arms: Vec<(LiteralValue, ASTNode)>,
        else_expr: Box<ASTNode>,
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
        init_fields: Vec<String>,         // initブロック内のフィールド定義
        weak_fields: Vec<String>,         // 🔗 weak修飾子が付いたフィールドのリスト
        is_interface: bool,               // interface box かどうか
        extends: Vec<String>,             // 🚀 Multi-delegation: Changed from Option<String> to Vec<String>
        implements: Vec<String>,          // 実装するinterface名のリスト
        type_parameters: Vec<String>,     // 🔥 ジェネリクス型パラメータ (例: ["T", "U"])
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
        is_static: bool,     // 🔥 静的メソッドフラグ
        is_override: bool,   // 🔥 オーバーライドフラグ
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
    Literal {
        value: LiteralValue,
        span: Span,
    },
    
    /// 変数参照: variableName
    Variable {
        name: String,
        span: Span,
    },
    
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
        type_arguments: Vec<String>,      // 🔥 ジェネリクス型引数 (例: ["IntegerBox", "StringBox"])
        span: Span,
    },
    
    /// this参照
    This {
        span: Span,
    },
    
    /// me参照
    Me {
        span: Span,
    },
    
    /// 🔥 from呼び出し: from Parent.method(arguments) or from Parent.constructor(arguments)
    FromCall {
        parent: String,        // Parent名
        method: String,        // method名またはconstructor
        arguments: Vec<ASTNode>, // 引数
        span: Span,
    },
    
    /// thisフィールドアクセス: this.field
    ThisField {
        field: String,
        span: Span,
    },
    
    /// meフィールドアクセス: me.field
    MeField {
        field: String,
        span: Span,
    },
    
    /// ファイル読み込み: include "filename.nyash"
    Include {
        filename: String,
        span: Span,
    },
    
    /// ローカル変数宣言: local x, y, z
    Local {
        variables: Vec<String>,
        /// 初期化値（変数と同じ順序、Noneは初期化なし）
        initial_values: Vec<Option<Box<ASTNode>>>,
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

impl ASTNode {
    /// AST nodeの種類を文字列で取得 (デバッグ用)
    pub fn node_type(&self) -> &'static str {
        match self {
            ASTNode::Program { .. } => "Program",
            ASTNode::Assignment { .. } => "Assignment",
            ASTNode::Print { .. } => "Print",
            ASTNode::If { .. } => "If",
            ASTNode::Loop { .. } => "Loop",
            ASTNode::Return { .. } => "Return",
            ASTNode::Break { .. } => "Break",
            ASTNode::Continue { .. } => "Continue",
            ASTNode::UsingStatement { .. } => "UsingStatement",
            ASTNode::BoxDeclaration { .. } => "BoxDeclaration",
            ASTNode::FunctionDeclaration { .. } => "FunctionDeclaration",
            ASTNode::GlobalVar { .. } => "GlobalVar",
            ASTNode::Literal { .. } => "Literal",
            ASTNode::Variable { .. } => "Variable",
            ASTNode::UnaryOp { .. } => "UnaryOp",
            ASTNode::BinaryOp { .. } => "BinaryOp",
            ASTNode::MethodCall { .. } => "MethodCall",
            ASTNode::FieldAccess { .. } => "FieldAccess",
            ASTNode::New { .. } => "New",
            ASTNode::This { .. } => "This",
            ASTNode::Me { .. } => "Me",
            ASTNode::FromCall { .. } => "FromCall",
            ASTNode::ThisField { .. } => "ThisField",
            ASTNode::MeField { .. } => "MeField",
            ASTNode::Include { .. } => "Include",
            ASTNode::Local { .. } => "Local",
            ASTNode::Outbox { .. } => "Outbox",
            ASTNode::FunctionCall { .. } => "FunctionCall",
            ASTNode::Call { .. } => "Call",
            ASTNode::Nowait { .. } => "Nowait",
            ASTNode::Arrow { .. } => "Arrow",
            ASTNode::TryCatch { .. } => "TryCatch",
            ASTNode::Throw { .. } => "Throw",
            ASTNode::AwaitExpression { .. } => "AwaitExpression",
            ASTNode::QMarkPropagate { .. } => "QMarkPropagate",
            ASTNode::PeekExpr { .. } => "PeekExpr",
            ASTNode::Lambda { .. } => "Lambda",
        }
    }
    
    /// 🌟 AST分類 - ChatGPTアドバイス統合による革新的分類システム
    /// Structure/Expression/Statement の明確な分離
    pub fn classify(&self) -> ASTNodeType {
        match self {
            // Structure nodes - 言語の基本構造
            ASTNode::BoxDeclaration { .. } => ASTNodeType::Structure,
            ASTNode::FunctionDeclaration { .. } => ASTNodeType::Structure,
            ASTNode::If { .. } => ASTNodeType::Structure,
            ASTNode::Loop { .. } => ASTNodeType::Structure,
            ASTNode::TryCatch { .. } => ASTNodeType::Structure,
            
            // Expression nodes - 値を生成する表現
            ASTNode::Literal { .. } => ASTNodeType::Expression,
            ASTNode::Variable { .. } => ASTNodeType::Expression,
            ASTNode::BinaryOp { .. } => ASTNodeType::Expression,
            ASTNode::UnaryOp { .. } => ASTNodeType::Expression,
            ASTNode::FunctionCall { .. } => ASTNodeType::Expression,
            ASTNode::Call { .. } => ASTNodeType::Expression,
            ASTNode::MethodCall { .. } => ASTNodeType::Expression,
            ASTNode::FieldAccess { .. } => ASTNodeType::Expression,
            ASTNode::New { .. } => ASTNodeType::Expression,
            ASTNode::This { .. } => ASTNodeType::Expression,
            ASTNode::Me { .. } => ASTNodeType::Expression,
            ASTNode::FromCall { .. } => ASTNodeType::Expression,
            ASTNode::ThisField { .. } => ASTNodeType::Expression,
            ASTNode::MeField { .. } => ASTNodeType::Expression,
            ASTNode::PeekExpr { .. } => ASTNodeType::Expression,
            ASTNode::QMarkPropagate { .. } => ASTNodeType::Expression,
            ASTNode::Lambda { .. } => ASTNodeType::Expression,
            
            // Statement nodes - 実行可能なアクション
            ASTNode::Program { .. } => ASTNodeType::Statement, // プログラム全体
            ASTNode::Assignment { .. } => ASTNodeType::Statement,
            ASTNode::Print { .. } => ASTNodeType::Statement,
            ASTNode::Return { .. } => ASTNodeType::Statement,
            ASTNode::Break { .. } => ASTNodeType::Statement,
            ASTNode::Continue { .. } => ASTNodeType::Statement,
            ASTNode::UsingStatement { .. } => ASTNodeType::Statement,
            ASTNode::GlobalVar { .. } => ASTNodeType::Statement,
            ASTNode::Include { .. } => ASTNodeType::Statement,
            ASTNode::Local { .. } => ASTNodeType::Statement,
            ASTNode::Outbox { .. } => ASTNodeType::Statement,
            ASTNode::Nowait { .. } => ASTNodeType::Statement,
            ASTNode::Arrow { .. } => ASTNodeType::Statement,
            ASTNode::Throw { .. } => ASTNodeType::Statement,
            ASTNode::AwaitExpression { .. } => ASTNodeType::Expression,
        }
    }
    
    /// 🎯 構造パターンチェック - 2段階パーサー用
    pub fn is_structure(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Structure)
    }
    
    /// ⚡ 式パターンチェック - 評価エンジン用
    pub fn is_expression(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Expression)
    }
    
    /// 📝 文パターンチェック - 実行エンジン用
    pub fn is_statement(&self) -> bool {
        matches!(self.classify(), ASTNodeType::Statement)
    }
    
    /// AST nodeの詳細情報を取得 (デバッグ用)
    pub fn info(&self) -> String {
        match self {
            ASTNode::Program { statements, .. } => {
                format!("Program({} statements)", statements.len())
            }
            ASTNode::Assignment { target, .. } => {
                format!("Assignment(target: {})", target.info())
            }
            ASTNode::Print { .. } => "Print".to_string(),
            ASTNode::If { .. } => "If".to_string(),
            ASTNode::Loop { condition: _, body, .. } => {
                format!("Loop({} statements)", body.len())
            }
            ASTNode::Return { value, .. } => {
                if value.is_some() {
                    "Return(with value)".to_string()
                } else {
                    "Return(void)".to_string()
                }
            }
            ASTNode::Break { .. } => "Break".to_string(),
            ASTNode::Continue { .. } => "Continue".to_string(),
            ASTNode::UsingStatement { namespace_name, .. } => {
                format!("UsingStatement({})", namespace_name)
            }
            ASTNode::BoxDeclaration { name, fields, methods, constructors,  is_interface, extends, implements, .. } => {
                let mut desc = if *is_interface {
                    format!("InterfaceBox({}, {} methods", name, methods.len())
                } else {
                    format!("BoxDeclaration({}, {} fields, {} methods, {} constructors", name, fields.len(), methods.len(), constructors.len())
                };
                
                if !extends.is_empty() {
                    desc.push_str(&format!(", extends [{}]", extends.join(", ")));
                }
                
                if !implements.is_empty() {
                    desc.push_str(&format!(", implements [{}]", implements.join(", ")));
                }
                
                desc.push(')');
                desc
            }
            ASTNode::FunctionDeclaration { name, params, body, is_static, is_override, .. } => {
                let static_str = if *is_static { "static " } else { "" };
                let override_str = if *is_override { "override " } else { "" };
                format!("FunctionDeclaration({}{}{}({}), {} statements)", 
                        override_str, static_str, name, params.join(", "), body.len())
            }
            ASTNode::GlobalVar { name, .. } => {
                format!("GlobalVar({})", name)
            }
            ASTNode::Literal { .. } => "Literal".to_string(),
            ASTNode::Variable { name, .. } => {
                format!("Variable({})", name)
            }
            ASTNode::UnaryOp { operator, .. } => {
                format!("UnaryOp({})", operator)
            }
            ASTNode::BinaryOp { operator, .. } => {
                format!("BinaryOp({})", operator)
            }
            ASTNode::MethodCall { method, arguments, .. } => {
                format!("MethodCall({}, {} args)", method, arguments.len())
            }
            ASTNode::FieldAccess { field, .. } => {
                format!("FieldAccess({})", field)
            }
            ASTNode::New { class, arguments, type_arguments, .. } => {
                if type_arguments.is_empty() {
                    format!("New({}, {} args)", class, arguments.len())
                } else {
                    format!("New({}<{}>, {} args)", class, type_arguments.join(", "), arguments.len())
                }
            }
            ASTNode::This { .. } => "This".to_string(),
            ASTNode::Me { .. } => "Me".to_string(),
            ASTNode::FromCall { parent, method, arguments, .. } => {
                format!("FromCall({}.{}, {} args)", parent, method, arguments.len())
            }
            ASTNode::ThisField { field, .. } => {
                format!("ThisField({})", field)
            }
            ASTNode::MeField { field, .. } => {
                format!("MeField({})", field)
            }
            ASTNode::Include { filename, .. } => {
                format!("Include({})", filename)
            }
            ASTNode::Local { variables, .. } => {
                format!("Local({})", variables.join(", "))
            }
            ASTNode::Outbox { variables, .. } => {
                format!("Outbox({})", variables.join(", "))
            }
            ASTNode::FunctionCall { name, arguments, .. } => {
                format!("FunctionCall({}, {} args)", name, arguments.len())
            }
            ASTNode::Call { .. } => "Call".to_string(),
            ASTNode::Nowait { variable, .. } => {
                format!("Nowait({})", variable)
            }
            ASTNode::Arrow { .. } => {
                "Arrow(>>)".to_string()
            }
            ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                let mut desc = format!("TryCatch({} try statements, {} catch clauses", 
                                      try_body.len(), catch_clauses.len());
                if finally_body.is_some() {
                    desc.push_str(", has finally");
                }
                desc.push(')');
                desc
            }
            ASTNode::Throw { .. } => "Throw".to_string(),
            ASTNode::AwaitExpression { expression, .. } => {
                format!("Await({:?})", expression)
            }
            ASTNode::PeekExpr { .. } => "PeekExpr".to_string(),
            ASTNode::QMarkPropagate { .. } => "QMarkPropagate".to_string(),
            ASTNode::Lambda { params, body, .. } => {
                format!("Lambda({} params, {} statements)", params.len(), body.len())
            }
        }
    }
    
    /// ASTノードからSpan情報を取得
    pub fn span(&self) -> Span {
        match self {
            ASTNode::Program { span, .. } => *span,
            ASTNode::Assignment { span, .. } => *span,
            ASTNode::Print { span, .. } => *span,
            ASTNode::If { span, .. } => *span,
            ASTNode::Loop { span, .. } => *span,
            ASTNode::Return { span, .. } => *span,
            ASTNode::Break { span, .. } => *span,
            ASTNode::Continue { span, .. } => *span,
            ASTNode::UsingStatement { span, .. } => *span,
            ASTNode::Nowait { span, .. } => *span,
            ASTNode::Arrow { span, .. } => *span,
            ASTNode::TryCatch { span, .. } => *span,
            ASTNode::Throw { span, .. } => *span,
            ASTNode::BoxDeclaration { span, .. } => *span,
            ASTNode::FunctionDeclaration { span, .. } => *span,
            ASTNode::GlobalVar { span, .. } => *span,
            ASTNode::Literal { span, .. } => *span,
            ASTNode::Variable { span, .. } => *span,
            ASTNode::UnaryOp { span, .. } => *span,
            ASTNode::BinaryOp { span, .. } => *span,
            ASTNode::MethodCall { span, .. } => *span,
            ASTNode::FieldAccess { span, .. } => *span,
            ASTNode::New { span, .. } => *span,
            ASTNode::This { span, .. } => *span,
            ASTNode::Me { span, .. } => *span,
            ASTNode::FromCall { span, .. } => *span,
            ASTNode::ThisField { span, .. } => *span,
            ASTNode::MeField { span, .. } => *span,
            ASTNode::Include { span, .. } => *span,
            ASTNode::Local { span, .. } => *span,
            ASTNode::Outbox { span, .. } => *span,
            ASTNode::FunctionCall { span, .. } => *span,
            ASTNode::Call { span, .. } => *span,
            ASTNode::AwaitExpression { span, .. } => *span,
            ASTNode::PeekExpr { span, .. } => *span,
            ASTNode::QMarkPropagate { span, .. } => *span,
            ASTNode::Lambda { span, .. } => *span,
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info())
    }
}

impl ASTNode {
    /// FunctionDeclarationのパラメータ数を取得
    pub fn get_param_count(&self) -> usize {
        match self {
            ASTNode::FunctionDeclaration { params, .. } => params.len(),
            _ => 0,
        }
    }
}

// Tests moved to integration tests to keep this file lean
