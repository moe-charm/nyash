/*!
 * Static declaration parsing
 * Handles both static functions and static boxes
 */

use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use crate::must_advance;

impl NyashParser {
    /// 静的宣言をパース - 🔥 static function / static box 記法  
    pub fn parse_static_declaration(&mut self) -> Result<ASTNode, ParseError> {
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
            is_static: true,  // 🔥 静的関数フラグを設定
            is_override: false, // デフォルトは非オーバーライド
            span: Span::unknown(),
        })
    }
}