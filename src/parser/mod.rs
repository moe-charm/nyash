/*!
 * Nyash Parser - Rust Implementation
 * 
 * Python版nyashc_v4.pyのNyashParserをRustで完全再実装
 * Token列をAST (Abstract Syntax Tree) に変換
 * 
 * TODO: リファクタリング計画
 * - expressions.rs: 式パーサー (parse_expression, parse_or, parse_and等)
 * - statements.rs: 文パーサー (parse_statement, parse_if, parse_loop等)
 * - declarations.rs: 宣言パーサー (parse_box_declaration, parse_function_declaration等)
 * - errors.rs: エラー型定義とハンドリング
 */

// サブモジュール宣言
mod common;
mod expressions;
mod statements;
mod declarations;
mod items;
// mod errors;

use common::ParserUtils;

use crate::tokenizer::{Token, TokenType, TokenizeError};
use crate::ast::{ASTNode, Span};
use std::collections::HashMap;
use thiserror::Error;

// ===== 🔥 Debug Macros =====

/// Infinite loop detection macro - must be called in every loop that advances tokens
/// Prevents parser from hanging due to token consumption bugs
/// Uses parser's debug_fuel field for centralized fuel management
#[macro_export]
macro_rules! must_advance {
    ($parser:expr, $fuel:expr, $location:literal) => {
        // デバッグ燃料がSomeの場合のみ制限チェック
        if let Some(ref mut limit) = $parser.debug_fuel {
            if *limit == 0 {
                eprintln!("🚨 PARSER INFINITE LOOP DETECTED at {}", $location);
                eprintln!("🔍 Current token: {:?} at line {}", $parser.current_token().token_type, $parser.current_token().line);
                eprintln!("🔍 Parser position: {}/{}", $parser.current, $parser.tokens.len());
                return Err($crate::parser::ParseError::InfiniteLoop { 
                    location: $location.to_string(),
                    token: $parser.current_token().token_type.clone(),
                    line: $parser.current_token().line,
                });
            }
            *limit -= 1;
        }
        // None の場合は無制限なのでチェックしない
    };
}

/// Initialize debug fuel for loop monitoring
#[macro_export]
macro_rules! debug_fuel {
    () => {
        100_000 // Default: 100k iterations should be enough for any reasonable program
    };
}

// Two-phase parser structures are no longer needed - simplified to direct parsing

/// パースエラー
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token {found:?}, expected {expected} at line {line}")]
    UnexpectedToken { found: TokenType, expected: String, line: usize },
    
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    
    #[error("Invalid expression at line {line}")]
    InvalidExpression { line: usize },
    
    #[error("Invalid statement at line {line}")]
    InvalidStatement { line: usize },
    
    #[error("Circular dependency detected between static boxes: {cycle}")]
    CircularDependency { cycle: String },
    
    #[error("🚨 Infinite loop detected in parser at {location} - token: {token:?} at line {line}")]
    InfiniteLoop { location: String, token: TokenType, line: usize },
    
    #[error("🔥 Transparency system removed: {suggestion} at line {line}")]
    TransparencySystemRemoved { suggestion: String, line: usize },
    
    #[error("Unsupported namespace '{name}' at line {line}. Only 'nyashstd' is supported in Phase 0.")]
    UnsupportedNamespace { name: String, line: usize },
    
    #[error("Expected identifier at line {line}")]
    ExpectedIdentifier { line: usize },
    
    #[error("Tokenize error: {0}")]
    TokenizeError(#[from] TokenizeError),
}

/// Nyashパーサー - トークン列をASTに変換
pub struct NyashParser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    /// 🔥 Static box依存関係追跡（循環依存検出用）
    pub(super) static_box_dependencies: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// 🔥 デバッグ燃料：無限ループ検出用制限値 (None = 無制限)
    pub(super) debug_fuel: Option<usize>,
}

// Implement ParserUtils trait
impl ParserUtils for NyashParser {
    fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
    
    fn current(&self) -> usize {
        self.current
    }
    
    fn current_mut(&mut self) -> &mut usize {
        &mut self.current
    }
}

impl NyashParser {
    /// 新しいパーサーを作成
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            static_box_dependencies: std::collections::HashMap::new(),
            debug_fuel: Some(100_000), // デフォルト値
        }
    }
    
    /// 文字列からパース (トークナイズ + パース)
    pub fn parse_from_string(input: impl Into<String>) -> Result<ASTNode, ParseError> {
        Self::parse_from_string_with_fuel(input, Some(100_000))
    }
    
    /// 文字列からパース (デバッグ燃料指定版)
    /// fuel: Some(n) = n回まで、None = 無制限
    pub fn parse_from_string_with_fuel(input: impl Into<String>, fuel: Option<usize>) -> Result<ASTNode, ParseError> {
        let mut tokenizer = crate::tokenizer::NyashTokenizer::new(input);
        let tokens = tokenizer.tokenize()?;
        
        let mut parser = Self::new(tokens);
        parser.debug_fuel = fuel;
        let result = parser.parse();
        result
    }
    
    /// パース実行 - Program ASTを返す
    pub fn parse(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_program()
    }
    
    // ===== パース関数群 =====
    
    /// プログラム全体をパース
    fn parse_program(&mut self) -> Result<ASTNode, ParseError> {
        let mut statements = Vec::new();
        let mut statement_count = 0;
        
        while !self.is_at_end() {
            
            // EOF tokenはスキップ
            if matches!(self.current_token().token_type, TokenType::EOF) {
                break;
            }
            
            // NEWLINE tokenはスキップ（文の区切りとして使用）
            if matches!(self.current_token().token_type, TokenType::NEWLINE) {
                self.advance();
                continue;
            }
            
            let statement = self.parse_statement()?;
            statements.push(statement);
            statement_count += 1;
        }
        
        
        // 🔥 すべてのstatic box解析後に循環依存検出
        self.check_circular_dependencies()?;
        
        Ok(ASTNode::Program { statements, span: Span::unknown() })
    }
    // Statement parsing methods are now in statements.rs module
    
    /// 代入文または関数呼び出しをパース
    fn parse_assignment_or_function_call(&mut self) -> Result<ASTNode, ParseError> {
        
        // まず左辺を式としてパース
        let expr = self.parse_expression()?;
        
        // 次のトークンが = なら代入文
        if self.match_token(&TokenType::ASSIGN) {
            self.advance(); // consume '='
            let value = Box::new(self.parse_expression()?);
            
            // 左辺が代入可能な形式かチェック
            match &expr {
                ASTNode::Variable { .. } | 
                ASTNode::FieldAccess { .. } => {
                    Ok(ASTNode::Assignment {
                        target: Box::new(expr),
                        value,
                        span: Span::unknown(),
                    })
                }
                _ => {
                    let line = self.current_token().line;
                    Err(ParseError::InvalidStatement { line })
                }
            }
        } else {
            // 代入文でなければ式文として返す
            Ok(expr)
        }
    }
    
    // Expression parsing methods are now in expressions.rs module
    // Utility methods are now in common.rs module via ParserUtils trait
    // Item parsing methods are now in items.rs module
    
    // ===== 🔥 Static Box循環依存検出 =====
}
