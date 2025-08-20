/*!
 * Static Box Definition Parser
 * 
 * static box宣言と関連ヘルパー関数
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use std::collections::HashMap;

impl NyashParser {
    /// static box宣言をパース: static box Name { ... }
    pub fn parse_static_box(&mut self) -> Result<ASTNode, ParseError> {
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
            public_fields: vec![],
            private_fields: vec![],
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
}
