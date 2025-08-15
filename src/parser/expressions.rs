/*!
 * Nyash Parser - Expression Parsing Module
 * 
 * 式（Expression）の解析を担当するモジュール
 * 演算子の優先順位に従った再帰下降パーサー実装
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, BinaryOperator, LiteralValue, UnaryOperator, Span};
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
    
    /// OR演算子をパース: ||
    fn parse_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&TokenType::OR) {
            let operator = BinaryOperator::Or;
            self.advance();
            let right = self.parse_and()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// AND演算子をパース: &&
    fn parse_and(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&TokenType::AND) {
            let operator = BinaryOperator::And;
            self.advance();
            let right = self.parse_equality()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 等値演算子をパース: == !=
    fn parse_equality(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_comparison()?;
        
        while self.match_token(&TokenType::EQUALS) || self.match_token(&TokenType::NotEquals) {
            let operator = match &self.current_token().token_type {
                TokenType::EQUALS => BinaryOperator::Equal,
                TokenType::NotEquals => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 比較演算子をパース: < <= > >=
    fn parse_comparison(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_term()?;
        
        while self.match_token(&TokenType::LESS) || 
              self.match_token(&TokenType::LessEquals) ||
              self.match_token(&TokenType::GREATER) ||
              self.match_token(&TokenType::GreaterEquals) {
            let operator = match &self.current_token().token_type {
                TokenType::LESS => BinaryOperator::Less,
                TokenType::LessEquals => BinaryOperator::LessEqual,
                TokenType::GREATER => BinaryOperator::Greater,
                TokenType::GreaterEquals => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 項をパース: + - >>
    fn parse_term(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_factor()?;
        
        while self.match_token(&TokenType::PLUS) || self.match_token(&TokenType::MINUS) || self.match_token(&TokenType::ARROW) {
            if self.match_token(&TokenType::ARROW) {
                // >> Arrow演算子
                self.advance();
                let right = self.parse_factor()?;
                expr = ASTNode::Arrow {
                    sender: Box::new(expr),
                    receiver: Box::new(right),
                    span: Span::unknown(),
                };
            } else {
                let operator = match &self.current_token().token_type {
                    TokenType::PLUS => BinaryOperator::Add,
                    TokenType::MINUS => BinaryOperator::Subtract,
                    _ => unreachable!(),
                };
                self.advance();
                let right = self.parse_factor()?;
                expr = ASTNode::BinaryOp {
                    operator,
                    left: Box::new(expr),
                    right: Box::new(right),
                    span: Span::unknown(),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// 因子をパース: * /
    fn parse_factor(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_unary()?;
        
        while self.match_token(&TokenType::MULTIPLY) || self.match_token(&TokenType::DIVIDE) || self.match_token(&TokenType::MODULO) {
            let operator = match &self.current_token().token_type {
                TokenType::MULTIPLY => BinaryOperator::Multiply,
                TokenType::DIVIDE => BinaryOperator::Divide,
                TokenType::MODULO => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        
        Ok(expr)
    }
    
    /// 単項演算子をパース
    fn parse_unary(&mut self) -> Result<ASTNode, ParseError> {
        if self.match_token(&TokenType::MINUS) {
            self.advance(); // consume '-'
            let operand = self.parse_unary()?; // 再帰的に単項演算をパース
            return Ok(ASTNode::UnaryOp {
                operator: UnaryOperator::Minus,
                operand: Box::new(operand),
                span: Span::unknown(),
            });
        }
        
        if self.match_token(&TokenType::NOT) {
            self.advance(); // consume 'not'
            let operand = self.parse_unary()?; // 再帰的に単項演算をパース
            return Ok(ASTNode::UnaryOp {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
                span: Span::unknown(),
            });
        }
        
        if self.match_token(&TokenType::AWAIT) {
            self.advance(); // consume 'await'
            let expression = self.parse_unary()?; // 再帰的にパース
            return Ok(ASTNode::AwaitExpression {
                expression: Box::new(expression),
                span: Span::unknown(),
            });
        }
        
        self.parse_call()
    }
    
    /// 関数・メソッド呼び出しをパース
    fn parse_call(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&TokenType::DOT) {
                self.advance(); // consume '.'
                
                if let TokenType::IDENTIFIER(method_name) = &self.current_token().token_type {
                    let method_name = method_name.clone();
                    self.advance();
                    
                    if self.match_token(&TokenType::LPAREN) {
                        // メソッド呼び出し: obj.method(args)
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
                        
                        expr = ASTNode::MethodCall {
                            object: Box::new(expr),
                            method: method_name,
                            arguments,
                            span: Span::unknown(),
                        };
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
                    
                    expr = ASTNode::FunctionCall { name, arguments, span: Span::unknown() };
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// 基本式をパース: リテラル、変数、括弧、this、new
    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
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
    
    /// from構文をパース: from Parent.method(arguments)
    pub(super) fn parse_from_call(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'from'
        
        // Parent名を取得
        let parent = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "parent class name".to_string(),
                line,
            });
        };
        
        // DOT とmethod名は任意（pack透明化対応）
        let method = if self.match_token(&TokenType::DOT) {
            // DOTがある場合: from Parent.method() 形式
            self.advance(); // consume DOT
            
            // method名を取得 (IDENTIFIERまたはINITを受け入れ)
            match &self.current_token().token_type {
                TokenType::IDENTIFIER(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                TokenType::INIT => {
                    self.advance();
                    "init".to_string()
                }
                TokenType::PACK => {
                    self.advance();
                    "pack".to_string()
                }
                TokenType::BIRTH => {
                    self.advance();
                    "birth".to_string()
                }
                _ => {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "method name".to_string(),
                        line,
                    });
                }
            }
        } else {
            // DOTがない場合: from Parent() 形式 - 透明化システム廃止
            // Phase 8.9: 明示的birth()構文を強制
            let line = self.current_token().line;
            return Err(ParseError::TransparencySystemRemoved {
                suggestion: format!("Use 'from {}.birth()' instead of 'from {}()'", parent, parent),
                line,
            });
        };
        
        // 引数リストをパース
        self.consume(TokenType::LPAREN)?;
        let mut arguments = Vec::new();
        
        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
            must_advance!(self, _unused, "from call argument parsing");
            
            arguments.push(self.parse_expression()?);
            
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                // カンマの後の trailing comma をチェック
            }
        }
        
        self.consume(TokenType::RPAREN)?;
        
        Ok(ASTNode::FromCall {
            parent,
            method,
            arguments,
            span: Span::unknown(),
        })
    }
}