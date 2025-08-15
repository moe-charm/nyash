/*!
 * Nyash Parser - Expression Parsing Module
 * 
 * 式（Expression）の解析を担当するモジュール
 * 演算子処理はoperators.rsに分離済み
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, LiteralValue, Span};
use super::{NyashParser, ParseError};

// ===== 🔥 Debug Macros (copied from parent module) =====

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

impl NyashParser {
    /// 式をパース (演算子優先順位あり)
    pub(super) fn parse_expression(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_or()
    }
    
    
    
    /// 基本式をパース: リテラル、変数、括弧、this、new
    pub(super) fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::STRING(s) => {
                let value = s.clone();
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::String(value),
                    span: Span::unknown(),
                })
            }
            
            TokenType::NUMBER(n) => {
                let value = *n;
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Integer(value),
                    span: Span::unknown(),
                })
            }
            
            TokenType::FLOAT(f) => {
                let value = *f;
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Float(value),
                    span: Span::unknown(),
                })
            }
            
            TokenType::TRUE => {
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Bool(true),
                    span: Span::unknown(),
                })
            }
            
            TokenType::FALSE => {
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Bool(false),
                    span: Span::unknown(),
                })
            }
            
            TokenType::NULL => {
                self.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Null,
                    span: Span::unknown(),
                })
            }
            
            TokenType::THIS => {
                self.advance();
                Ok(ASTNode::This { span: Span::unknown() })
            }
            
            TokenType::ME => {
                self.advance();
                Ok(ASTNode::Me { span: Span::unknown() })
            }
            
            TokenType::NEW => {
                self.advance();
                
                if let TokenType::IDENTIFIER(class_name) = &self.current_token().token_type {
                    let class_name = class_name.clone();
                    self.advance();
                    
                    // 🔥 ジェネリクス型引数のパース (<IntegerBox, StringBox>)
                    let type_arguments = if self.match_token(&TokenType::LESS) {
                        self.advance(); // consume '<'
                        let mut args = Vec::new();
                        
                        loop {
                            if let TokenType::IDENTIFIER(type_name) = &self.current_token().token_type {
                                args.push(type_name.clone());
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
                                    expected: "type argument".to_string(),
                                    line,
                                });
                            }
                        }
                        
                        self.consume(TokenType::GREATER)?; // consume '>'
                        args
                    } else {
                        Vec::new()
                    };
                    
                    self.consume(TokenType::LPAREN)?;
                    let mut arguments = Vec::new();
                    
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "new expression argument parsing");
                        
                        arguments.push(self.parse_expression()?);
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    
                    self.consume(TokenType::RPAREN)?;
                    
                    Ok(ASTNode::New {
                        class: class_name,
                        arguments,
                        type_arguments,
                        span: Span::unknown(),
                    })
                } else {
                    let line = self.current_token().line;
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "class name".to_string(),
                        line,
                    })
                }
            }
            
            TokenType::FROM => {
                // from構文をパース: from Parent.method(arguments)
                self.parse_from_call()
            }
            
            TokenType::IDENTIFIER(name) => {
                let name = name.clone();
                self.advance();
                Ok(ASTNode::Variable { name, span: Span::unknown() })
            }
            
            TokenType::LPAREN => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.consume(TokenType::RPAREN)?;
                Ok(expr)
            }
            
            _ => {
                let line = self.current_token().line;
                Err(ParseError::InvalidExpression { line })
            }
        }
    }
    
}