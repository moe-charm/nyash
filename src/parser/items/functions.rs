/*!
 * Function declaration parsing
 */

use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use crate::must_advance;

impl NyashParser {
    /// function宣言をパース: function name(params) { body }
    pub fn parse_function_declaration(&mut self) -> Result<ASTNode, ParseError> {
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
}