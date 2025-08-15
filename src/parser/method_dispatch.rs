/*!
 * Nyash Parser - Method Dispatch Module
 * 
 * メソッド呼び出しと関数呼び出しの解析を担当するモジュール
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use super::{NyashParser, ParseError};

// Macro for infinite loop detection
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

impl NyashParser {
    /// 関数・メソッド呼び出しをパース
    pub(super) fn parse_call(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&TokenType::DOT) {
                self.advance(); // consume '.'
                
                if let TokenType::IDENTIFIER(method_name) = &self.current_token().token_type {
                    let method_name = method_name.clone();
                    self.advance();
                    
                    if self.match_token(&TokenType::LPAREN) {
                        // メソッド呼び出し: obj.method(args)
                        expr = self.parse_method_call(expr, method_name)?;
                    } else {
                        // フィールドアクセス: obj.field
                        expr = ASTNode::FieldAccess {
                            object: Box::new(expr),
                            field: method_name,
                            span: Span::unknown(),
                        };
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "identifier".to_string(),
                        line,
                    });
                }
            } else if self.match_token(&TokenType::LPAREN) {
                // 関数呼び出し: function(args)
                if let ASTNode::Variable { name, .. } = expr {
                    expr = self.parse_function_call(name)?;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// メソッド呼び出しをパース: obj.method(args)
    fn parse_method_call(&mut self, object: ASTNode, method_name: String) -> Result<ASTNode, ParseError> {
        self.advance(); // consume '('
        let mut arguments = Vec::new();
        let mut arg_count = 0;
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "method call argument parsing");
            
            arguments.push(self.parse_expression()?);
            arg_count += 1;
            
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                // カンマの後の trailing comma をチェック
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        Ok(ASTNode::MethodCall {
            object: Box::new(object),
            method: method_name,
            arguments,
            span: Span::unknown(),
        })
    }
    
    /// 関数呼び出しをパース: function(args)
    fn parse_function_call(&mut self, function_name: String) -> Result<ASTNode, ParseError> {
        self.advance(); // consume '('
        let mut arguments = Vec::new();
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "function call argument parsing");
            
            arguments.push(self.parse_expression()?);
            if self.match_token(&TokenType::COMMA) {
                self.advance();
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        Ok(ASTNode::FunctionCall { 
            name: function_name, 
            arguments, 
            span: Span::unknown() 
        })
    }
}