/*!
 * Nyash Parser - Rust Implementation
 * 
 * Token列をAST (Abstract Syntax Tree) に変換するメインパーサー
 * 
 * アーキテクチャ:
 * - mod.rs: メインパーサー・宣言パース・ユーティリティ
 * - expressions.rs: 式解析コーディネーター (208行)
 * - operators.rs: 演算子処理 (208行)
 * - method_dispatch.rs: メソッド・関数呼び出し (128行)
 * - async_ops.rs: await処理 (20行)
 * - delegation.rs: from呼び出し (117行)
 * - statements.rs: 文解析
 * - dependency_analysis.rs: 循環依存検出 (106行)
 * - validation.rs: オーバーライド検証 (55行)
 */

// サブモジュール宣言
mod expressions;
mod operators;
mod method_dispatch;
mod async_ops;
mod delegation;
mod statements;
mod dependency_analysis;
mod validation;
// mod declarations;
// mod errors;

use crate::tokenizer::{Token, TokenType, TokenizeError};
use crate::ast::{ASTNode, Span};
use std::collections::HashMap;
use thiserror::Error;

// ===== 🔥 Debug Macros =====

/// Infinite loop detection macro - must be called in every loop that advances tokens
/// Prevents parser from hanging due to token consumption bugs
/// Uses parser's debug_fuel field for centralized fuel management
macro_rules! must_advance {
    ($parser:expr, $fuel:expr, $location:literal) => {
        // デバッグ燃料がSomeの場合のみ制限チェック
        if let Some(ref mut limit) = $parser.debug_fuel {
            if *limit == 0 {
                eprintln!("🚨 PARSER INFINITE LOOP DETECTED at {}", $location);
                eprintln!("🔍 Current token: {:?} at line {}", $parser.current_token().token_type, $parser.current_token().line);
                eprintln!("🔍 Parser position: {}/{}", $parser.current, $parser.tokens.len());
                return Err(ParseError::InfiniteLoop { 
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
    tokens: Vec<Token>,
    current: usize,
    /// 🔥 Static box依存関係追跡（循環依存検出用）
    static_box_dependencies: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// 🔥 デバッグ燃料：無限ループ検出用制限値 (None = 無制限)
    debug_fuel: Option<usize>,
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
    // === Statement parsing methods are in statements.rs module ===
    
    /// box宣言をパース: box Name { fields... methods... }
    fn parse_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::BOX)?;
        
        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };
        
        // 🔥 ジェネリクス型パラメータのパース (<T, U>)
        let type_parameters = if self.match_token(&TokenType::LESS) {
            self.advance(); // consume '<'
            let mut params = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(param_name) = &self.current_token().token_type {
                    params.push(param_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "type parameter name".to_string(),
                        line,
                    });
                }
            }
            
            self.consume(TokenType::GREATER)?; // consume '>'
            params
        } else {
            Vec::new()
        };
        
        // from句のパース（Multi-delegation）🚀
        let extends = if self.match_token(&TokenType::FROM) {
            self.advance(); // consume 'from'
            
            let mut parent_list = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(parent_name) = &self.current_token().token_type {
                    parent_list.push(parent_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "parent class name".to_string(),
                        line,
                    });
                }
            }
            
            parent_list
        } else {
            Vec::new()
        };
        
        // interface句のパース（インターフェース実装）
        let implements = if self.match_token(&TokenType::INTERFACE) {
            self.advance(); // consume 'interface'
            
            let mut interface_list = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(interface_name) = &self.current_token().token_type {
                    interface_list.push(interface_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "interface name".to_string(),
                        line,
                    });
                }
            }
            
            interface_list
        } else {
            vec![]
        };
        
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ
        
        let mut fields = Vec::new();
        let mut methods = HashMap::new();
        let mut constructors = HashMap::new();
        let mut init_fields = Vec::new();
        let mut weak_fields = Vec::new();  // 🔗 Track weak fields
        
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ
            
            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }
            
            // initブロックの処理（initメソッドではない場合のみ）
            if self.match_token(&TokenType::INIT) && self.peek_token() != &TokenType::LPAREN {
                self.advance(); // consume 'init'
                self.consume(TokenType::LBRACE)?;
                
                // initブロック内のフィールド定義を読み込み
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();
                    
                    if self.match_token(&TokenType::RBRACE) {
                        break;
                    }
                    
                    // Check for weak modifier
                    let is_weak = if self.match_token(&TokenType::WEAK) {
                        self.advance(); // consume 'weak'
                        true
                    } else {
                        false
                    };
                    
                    if let TokenType::IDENTIFIER(field_name) = &self.current_token().token_type {
                        init_fields.push(field_name.clone());
                        if is_weak {
                            weak_fields.push(field_name.clone()); // 🔗 Add to weak fields list
                        }
                        self.advance();
                        
                        // カンマがあればスキップ
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    } else {
                        // 不正なトークンがある場合はエラー
                        return Err(ParseError::UnexpectedToken {
                            expected: if is_weak { "field name after 'weak'" } else { "field name" }.to_string(),
                            found: self.current_token().token_type.clone(),
                            line: self.current_token().line,
                        });
                    }
                }
                
                self.consume(TokenType::RBRACE)?;
                continue;
            }
            
            // overrideキーワードをチェック
            let mut is_override = false;
            if self.match_token(&TokenType::OVERRIDE) {
                is_override = true;
                self.advance();
            }
            
            // initトークンをメソッド名として特別処理
            if self.match_token(&TokenType::INIT) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "init".to_string();
                self.advance(); // consume 'init'
                
                // コンストラクタとして処理
                if self.match_token(&TokenType::LPAREN) {
                    // initは常にコンストラクタ
                    if is_override {
                        return Err(ParseError::UnexpectedToken {
                            expected: "method definition, not constructor after override keyword".to_string(),
                            found: TokenType::INIT,
                            line: self.current_token().line,
                        });
                    }
                    // コンストラクタの処理
                    self.advance(); // consume '('
                    
                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "constructor parameter parsing");
                        
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }
                        
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    self.consume(TokenType::LBRACE)?;
                    
                    let mut body = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            body.push(self.parse_statement()?);
                        }
                    }
                    
                    self.consume(TokenType::RBRACE)?;
                    
                    let constructor = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params: params.clone(),
                        body,
                        is_static: false,
                        is_override: false,
                        span: Span::unknown(),
                    };
                    
                    // パラメータの数でコンストラクタを区別
                    let constructor_key = format!("{}/{}", field_or_method, params.len());
                    constructors.insert(constructor_key, constructor);
                }
            }
            
            // packトークンをメソッド名として特別処理
            else if self.match_token(&TokenType::PACK) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "pack".to_string();
                self.advance(); // consume 'pack'
                
                // コンストラクタとして処理
                if self.match_token(&TokenType::LPAREN) {
                    // packは常にコンストラクタ
                    if is_override {
                        return Err(ParseError::UnexpectedToken {
                            expected: "method definition, not constructor after override keyword".to_string(),
                            found: TokenType::PACK,
                            line: self.current_token().line,
                        });
                    }
                    // コンストラクタの処理
                    self.advance(); // consume '('
                    
                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "constructor parameter parsing");
                        
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }
                        
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    self.consume(TokenType::LBRACE)?;
                    
                    let mut body = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            body.push(self.parse_statement()?);
                        }
                    }
                    
                    self.consume(TokenType::RBRACE)?;
                    
                    let constructor = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params: params.clone(),
                        body,
                        is_static: false,
                        is_override: false,
                        span: Span::unknown(),
                    };
                    
                    // パラメータの数でコンストラクタを区別
                    let constructor_key = format!("{}/{}", field_or_method, params.len());
                    constructors.insert(constructor_key, constructor);
                }
            }
            
            // birthトークンをメソッド名として特別処理
            else if self.match_token(&TokenType::BIRTH) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "birth".to_string();
                self.advance(); // consume 'birth'
                
                // コンストラクタとして処理
                if self.match_token(&TokenType::LPAREN) {
                    // birthは常にコンストラクタ
                    if is_override {
                        return Err(ParseError::UnexpectedToken {
                            expected: "method definition, not constructor after override keyword".to_string(),
                            found: TokenType::BIRTH,
                            line: self.current_token().line,
                        });
                    }
                    // コンストラクタの処理
                    self.advance(); // consume '('
                    
                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "constructor parameter parsing");
                        
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }
                        
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    self.consume(TokenType::LBRACE)?;
                    
                    let mut body = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            body.push(self.parse_statement()?);
                        }
                    }
                    
                    self.consume(TokenType::RBRACE)?;
                    
                    let constructor = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params: params.clone(),
                        body,
                        is_static: false,
                        is_override: false,
                        span: Span::unknown(),
                    };
                    
                    // パラメータの数でコンストラクタを区別
                    let constructor_key = format!("{}/{}", field_or_method, params.len());
                    constructors.insert(constructor_key, constructor);
                }
            } else if let TokenType::IDENTIFIER(field_or_method) = &self.current_token().token_type {
                let field_or_method = field_or_method.clone();
                self.advance();
                
                // メソッド定義またはコンストラクタか？
                if self.match_token(&TokenType::LPAREN) {
                    // Box名と同じまたは"init"または"pack"または"birth"の場合はコンストラクタ
                    if field_or_method == name || field_or_method == "init" || field_or_method == "pack" || field_or_method == "birth" {
                        // コンストラクタはoverrideできない
                        if is_override {
                            return Err(ParseError::UnexpectedToken {
                                expected: "method definition, not constructor after override keyword".to_string(),
                                found: TokenType::IDENTIFIER(field_or_method.clone()),
                                line: self.current_token().line,
                            });
                        }
                        // コンストラクタの処理
                        self.advance(); // consume '('
                        
                        let mut params = Vec::new();
                        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                            must_advance!(self, _unused, "constructor parameter parsing");
                            
                            if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                                params.push(param.clone());
                                self.advance();
                            }
                            
                            if self.match_token(&TokenType::COMMA) {
                                self.advance();
                            }
                        }
                        
                        self.consume(TokenType::RPAREN)?;
                        self.consume(TokenType::LBRACE)?;
                        
                        let mut body = Vec::new();
                        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                            self.skip_newlines();
                            if !self.match_token(&TokenType::RBRACE) {
                                body.push(self.parse_statement()?);
                            }
                        }
                        
                        self.consume(TokenType::RBRACE)?;
                        
                        let constructor = ASTNode::FunctionDeclaration {
                            name: field_or_method.clone(),
                            params: params.clone(),
                            body,
                            is_static: false,  // コンストラクタは静的でない
                            is_override: false, // デフォルトは非オーバーライド
                            span: Span::unknown(),
                        };
                        
                        // パラメータの数でコンストラクタを区別
                        let constructor_key = format!("{}/{}", field_or_method, params.len());
                        constructors.insert(constructor_key, constructor);
                    } else {
                        // 通常のメソッド定義
                        self.advance(); // consume '('
                        
                        let mut params = Vec::new();
                        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                            must_advance!(self, _unused, "box method parameter parsing");
                            
                            if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                                params.push(param.clone());
                                self.advance();
                                
                                if self.match_token(&TokenType::COMMA) {
                                    self.advance();
                                    // カンマの後に閉じ括弧があるかチェック（trailing comma）
                                }
                            } else if !self.match_token(&TokenType::RPAREN) {
                                // IDENTIFIERでもRPARENでもない場合はエラー
                                let line = self.current_token().line;
                                return Err(ParseError::UnexpectedToken {
                                    found: self.current_token().token_type.clone(),
                                    expected: "parameter name or ')'".to_string(),
                                    line,
                                });
                            }
                        }
                        
                        self.consume(TokenType::RPAREN)?;
                        self.consume(TokenType::LBRACE)?;
                        
                        let mut body = Vec::new();
                        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                            self.skip_newlines(); // メソッド本体内の改行をスキップ
                            if !self.match_token(&TokenType::RBRACE) {
                                body.push(self.parse_statement()?);
                            }
                        }
                        
                        self.consume(TokenType::RBRACE)?;
                        
                        let method = ASTNode::FunctionDeclaration {
                            name: field_or_method.clone(),
                            params,
                            body,
                            is_static: false,  // メソッドは通常静的でない
                            is_override, // overrideキーワードの有無を反映
                            span: Span::unknown(),
                        };
                        
                        methods.insert(field_or_method, method);
                    }
                } else {
                    // フィールド定義
                    if is_override {
                        return Err(ParseError::UnexpectedToken {
                            expected: "method definition after override keyword".to_string(),
                            found: self.current_token().token_type.clone(),
                            line: self.current_token().line,
                        });
                    }
                    fields.push(field_or_method);
                }
                self.skip_newlines(); // フィールド/メソッド定義後の改行をスキップ
            } else {
                // 予期しないトークンの場合、詳細なエラー情報を出力してスキップ
                let line = self.current_token().line;
                eprintln!("Debug: Unexpected token {:?} at line {}", self.current_token().token_type, line);
                self.advance(); // トークンをスキップして続行
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        // 🔍 デリゲーションメソッドチェック：親Boxに存在しないメソッドのoverride検出
        if !extends.is_empty() {
            // For multi-delegation, validate against all parents
            for parent_name in &extends {
                self.validate_override_methods(&name, parent_name, &methods)?;
            }
        }
        
        Ok(ASTNode::BoxDeclaration {
            name,
            fields,
            methods,
            constructors,
            init_fields,
            weak_fields,  // 🔗 Add weak fields to the construction
            is_interface: false,
            extends,
            implements,
            type_parameters,
            is_static: false,
            static_init: None,
            span: Span::unknown(),
        })
    }
    
    /// インターフェースBox宣言をパース: interface box Name { method1() method2() }
    fn parse_interface_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::INTERFACE)?;
        self.consume(TokenType::BOX)?;
        
        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };
        
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ
        
        let mut methods = HashMap::new();
        
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ
            if let TokenType::IDENTIFIER(method_name) = &self.current_token().token_type {
                let method_name = method_name.clone();
                self.advance();
                
                // インターフェースメソッドはシグネチャのみ
                if self.match_token(&TokenType::LPAREN) {
                    self.advance(); // consume '('
                    
                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }
                        
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    
                    // インターフェースメソッドは実装なし（空のbody）
                    let method_decl = ASTNode::FunctionDeclaration {
                        name: method_name.clone(),
                        params,
                        body: vec![], // 空の実装
                        is_static: false,  // インターフェースメソッドは通常静的でない
                        is_override: false, // デフォルトは非オーバーライド
                        span: Span::unknown(),
                    };
                    
                    methods.insert(method_name, method_decl);
                    
                    // メソッド宣言後の改行をスキップ
                    self.skip_newlines();
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "(".to_string(),
                        line,
                    });
                }
            } else {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "method name".to_string(),
                    line,
                });
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        Ok(ASTNode::BoxDeclaration {
            name,
            fields: vec![], // インターフェースはフィールドなし
            methods,
            constructors: HashMap::new(), // インターフェースにコンストラクタなし
            init_fields: vec![], // インターフェースにinitブロックなし
            weak_fields: vec![], // 🔗 インターフェースにweak fieldsなし
            is_interface: true, // インターフェースフラグ
            extends: vec![],  // 🚀 Multi-delegation: Changed from None to vec![]
            implements: vec![],
            type_parameters: Vec::new(), // 🔥 インターフェースではジェネリクス未対応
            is_static: false, // インターフェースは非static
            static_init: None, // インターフェースにstatic initなし
            span: Span::unknown(),
        })
    }
    
    /// グローバル変数をパース: global name = value
    fn parse_global_var(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::GLOBAL)?;
        
        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };
        
        self.consume(TokenType::ASSIGN)?;
        let value = Box::new(self.parse_expression()?);
        
        Ok(ASTNode::GlobalVar { name, value, span: Span::unknown() })
    }
    // === Statement parsing methods are in statements.rs module ===
    
    /// function宣言をパース: function name(params) { body }
    fn parse_function_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::FUNCTION)?;
        
        // 関数名を取得
        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "function name".to_string(),
                line,
            });
        };
        
        // パラメータリストをパース
        self.consume(TokenType::LPAREN)?;
        let mut params = Vec::new();
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "function declaration parameter parsing");
            
            if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                params.push(param.clone());
                self.advance();
                
                if self.match_token(&TokenType::COMMA) {
                    self.advance();
                }
            } else if !self.match_token(&TokenType::RPAREN) {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "parameter name".to_string(),
                    line,
                });
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        // 関数本体をパース
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines();
        
        let mut body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if !self.match_token(&TokenType::RBRACE) {
                body.push(self.parse_statement()?);
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        Ok(ASTNode::FunctionDeclaration {
            name,
            params,
            body,
            is_static: false,  // 通常の関数は静的でない
            is_override: false, // デフォルトは非オーバーライド
            span: Span::unknown(),
        })
    }
    
    /// 静的宣言をパース - 🔥 static function / static box 記法  
    fn parse_static_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::STATIC)?;
        
        // 次のトークンで分岐: function か box か
        match &self.current_token().token_type {
            TokenType::FUNCTION => self.parse_static_function(),
            TokenType::BOX => self.parse_static_box(),
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "function or box after static".to_string(),
                    line,
                })
            }
        }
    }
    
    /// 静的関数宣言をパース - static function Name() { ... }
    fn parse_static_function(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::FUNCTION)?;
        
        // 関数名を取得（Box名.関数名の形式をサポート）
        let name = if let TokenType::IDENTIFIER(first_part) = &self.current_token().token_type {
            let mut full_name = first_part.clone();
            self.advance();
            
            // ドット記法をチェック（例：Math.min）
            if self.match_token(&TokenType::DOT) {
                self.advance(); // DOTを消費
                
                if let TokenType::IDENTIFIER(method_name) = &self.current_token().token_type {
                    full_name = format!("{}.{}", full_name, method_name);
                    self.advance();
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "method name after dot".to_string(),
                        line,
                    });
                }
            }
            
            full_name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "static function name".to_string(),
                line,
            });
        };
        
        // パラメータリストをパース
        self.consume(TokenType::LPAREN)?;
        let mut params = Vec::new();
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "static function parameter parsing");
            
            if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                params.push(param.clone());
                self.advance();
                
                if self.match_token(&TokenType::COMMA) {
                    self.advance();
                }
            } else if !self.match_token(&TokenType::RPAREN) {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "parameter name".to_string(),
                    line,
                });
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        // 関数本体をパース
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレースの後の改行をスキップ
        
        let mut body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時の改行をスキップ
            if !self.match_token(&TokenType::RBRACE) {
                body.push(self.parse_statement()?);
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        Ok(ASTNode::FunctionDeclaration {
            name,
            params,
            body,
            is_static: true,  // 🔥 静的関数フラグを設定
            is_override: false, // デフォルトは非オーバーライド
            span: Span::unknown(),
        })
    }
    
    /// 静的Box宣言をパース - static box Name { ... }
    fn parse_static_box(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::BOX)?;
        
        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };
        
        // 🔥 ジェネリクス型パラメータのパース (<T, U>)
        let type_parameters = if self.match_token(&TokenType::LESS) {
            self.advance(); // consume '<'
            let mut params = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(param_name) = &self.current_token().token_type {
                    params.push(param_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "type parameter name".to_string(),
                        line,
                    });
                }
            }
            
            self.consume(TokenType::GREATER)?; // consume '>'
            params
        } else {
            Vec::new()
        };
        
        // from句のパース（Multi-delegation）- static boxでもデリゲーション可能 🚀
        let extends = if self.match_token(&TokenType::FROM) {
            self.advance(); // consume 'from'
            
            let mut parent_list = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(parent_name) = &self.current_token().token_type {
                    parent_list.push(parent_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "parent class name".to_string(),
                        line,
                    });
                }
            }
            
            parent_list
        } else {
            Vec::new()
        };
        
        // interface句のパース（インターフェース実装）- static boxでもinterface実装可能
        let implements = if self.match_token(&TokenType::INTERFACE) {
            self.advance(); // consume 'interface'
            
            let mut interface_list = Vec::new();
            
            loop {
                if let TokenType::IDENTIFIER(interface_name) = &self.current_token().token_type {
                    interface_list.push(interface_name.clone());
                    self.advance();
                    
                    if self.match_token(&TokenType::COMMA) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "interface name".to_string(),
                        line,
                    });
                }
            }
            
            interface_list
        } else {
            vec![]
        };
        
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ
        
        let mut fields = Vec::new();
        let mut methods = HashMap::new();
        let constructors = HashMap::new();
        let mut init_fields = Vec::new();
        let mut weak_fields = Vec::new();  // 🔗 Track weak fields for static box
        let mut static_init = None;
        
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ
            
            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }
            
            // 🔥 static { } ブロックの処理
            if self.match_token(&TokenType::STATIC) {
                self.advance(); // consume 'static'
                self.consume(TokenType::LBRACE)?;
                
                let mut static_body = Vec::new();
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();
                    if !self.match_token(&TokenType::RBRACE) {
                        static_body.push(self.parse_statement()?);
                    }
                }
                
                self.consume(TokenType::RBRACE)?;
                static_init = Some(static_body);
                continue;
            }
            
            // initブロックの処理
            if self.match_token(&TokenType::INIT) {
                self.advance(); // consume 'init'
                self.consume(TokenType::LBRACE)?;
                
                // initブロック内のフィールド定義を読み込み
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();
                    
                    if self.match_token(&TokenType::RBRACE) {
                        break;
                    }
                    
                    // Check for weak modifier
                    let is_weak = if self.match_token(&TokenType::WEAK) {
                        self.advance(); // consume 'weak'
                        true
                    } else {
                        false
                    };
                    
                    if let TokenType::IDENTIFIER(field_name) = &self.current_token().token_type {
                        init_fields.push(field_name.clone());
                        if is_weak {
                            weak_fields.push(field_name.clone()); // 🔗 Add to weak fields list
                        }
                        self.advance();
                        
                        // カンマがあればスキップ
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    } else {
                        // 不正なトークンがある場合はエラー
                        return Err(ParseError::UnexpectedToken {
                            expected: if is_weak { "field name after 'weak'" } else { "field name" }.to_string(),
                            found: self.current_token().token_type.clone(),
                            line: self.current_token().line,
                        });
                    }
                }
                
                self.consume(TokenType::RBRACE)?;
                continue;
            }
            
            if let TokenType::IDENTIFIER(field_or_method) = &self.current_token().token_type {
                let field_or_method = field_or_method.clone();
                self.advance();
                
                // メソッド定義か？
                if self.match_token(&TokenType::LPAREN) {
                    // メソッド定義
                    self.advance(); // consume '('
                    
                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }
                        
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    self.consume(TokenType::LBRACE)?;
                    
                    let mut body = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            body.push(self.parse_statement()?);
                        }
                    }
                    
                    self.consume(TokenType::RBRACE)?;
                    
                    let method = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params,
                        body,
                        is_static: false,  // static box内のメソッドは通常メソッド
                        is_override: false, // デフォルトは非オーバーライド
                        span: Span::unknown(),
                    };
                    
                    methods.insert(field_or_method, method);
                } else {
                    // フィールド定義
                    fields.push(field_or_method);
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "method or field name".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        // 🔥 Static初期化ブロックから依存関係を抽出
        if let Some(ref init_stmts) = static_init {
            let dependencies = self.extract_dependencies_from_statements(init_stmts);
            self.static_box_dependencies.insert(name.clone(), dependencies);
        } else {
            self.static_box_dependencies.insert(name.clone(), std::collections::HashSet::new());
        }
        
        Ok(ASTNode::BoxDeclaration {
            name,
            fields,
            methods,
            constructors,
            init_fields,
            weak_fields,  // 🔗 Add weak fields to static box construction
            is_interface: false,
            extends,
            implements,
            type_parameters,
            is_static: true,  // 🔥 static boxフラグを設定
            static_init,      // 🔥 static初期化ブロック
            span: Span::unknown(),
        })
    }
    
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
    
    // === Expression parsing methods are in expressions.rs module ===
    
    // ===== ユーティリティメソッド =====
    
    /// 現在のトークンを取得
    fn current_token(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: 0,
            column: 0,
        })
    }
    
    /// 次のトークンを先読み（位置を進めない）
    fn peek_token(&self) -> &TokenType {
        if self.current + 1 < self.tokens.len() {
            &self.tokens[self.current + 1].token_type
        } else {
            &TokenType::EOF
        }
    }
    
    /// 位置を1つ進める
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    
    /// NEWLINEトークンをスキップ
    fn skip_newlines(&mut self) {
        let mut skip_count = 0;
        while matches!(self.current_token().token_type, TokenType::NEWLINE) && !self.is_at_end() {
            self.advance();
            skip_count += 1;
        }
        if skip_count > 0 {
        }
    }
    
    /// 指定されたトークンタイプを消費 (期待通りでなければエラー)
    fn consume(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        
        if std::mem::discriminant(&self.current_token().token_type) == 
           std::mem::discriminant(&expected) {
            let token = self.current_token().clone();
            self.advance();
            Ok(token)
        } else {
            let line = self.current_token().line;
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: format!("{:?}", expected),
                line,
            })
        }
    }
    
    /// 現在のトークンが指定されたタイプかチェック
    fn match_token(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current_token().token_type) == 
        std::mem::discriminant(token_type)
    }
    
    /// 終端に達したかチェック
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.current_token().token_type, TokenType::EOF)
    }
    // === Include, local, outbox, try/catch/throw parsing methods are in statements.rs module ===
    
    // === 🔥 Dependency Analysis moved to dependency_analysis.rs ===
    // === Override validation moved to validation.rs ===
}

