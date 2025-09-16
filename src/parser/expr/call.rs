use crate::ast::{ASTNode, Span};
use crate::must_advance;
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

#[inline]
fn is_sugar_enabled() -> bool {
    crate::parser::sugar_gate::is_enabled()
}

impl NyashParser {
    pub(crate) fn expr_parse_call(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_primary()?;

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
                        let mut _arg_count = 0;

                        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                            must_advance!(self, _unused, "method call argument parsing");

                            arguments.push(self.parse_expression()?);
                            _arg_count += 1;

                            if self.match_token(&TokenType::COMMA) {
                                self.advance();
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
            } else if self.match_token(&TokenType::QmarkDot) {
                if !is_sugar_enabled() {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full for '?.'".to_string(),
                        line,
                    });
                }
                self.advance(); // consume '?.'
                                // ident then optional call
                let name = match &self.current_token().token_type {
                    TokenType::IDENTIFIER(s) => {
                        let v = s.clone();
                        self.advance();
                        v
                    }
                    _ => {
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "identifier after '?.'".to_string(),
                            line,
                        });
                    }
                };
                let access = if self.match_token(&TokenType::LPAREN) {
                    // method call
                    self.advance();
                    let mut arguments = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "safe method call arg parsing");
                        arguments.push(self.parse_expression()?);
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    self.consume(TokenType::RPAREN)?;
                    ASTNode::MethodCall {
                        object: Box::new(expr.clone()),
                        method: name,
                        arguments,
                        span: Span::unknown(),
                    }
                } else {
                    // field access
                    ASTNode::FieldAccess {
                        object: Box::new(expr.clone()),
                        field: name,
                        span: Span::unknown(),
                    }
                };

                // Wrap with peek: peek expr { null => null, else => access(expr) }
                expr = ASTNode::PeekExpr {
                    scrutinee: Box::new(expr.clone()),
                    arms: vec![(
                        crate::ast::LiteralValue::Null,
                        ASTNode::Literal {
                            value: crate::ast::LiteralValue::Null,
                            span: Span::unknown(),
                        },
                    )],
                    else_expr: Box::new(access),
                    span: Span::unknown(),
                };
            } else if self.match_token(&TokenType::LPAREN) {
                // 関数呼び出し: function(args) または 一般式呼び出し: (callee)(args)
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

                if let ASTNode::Variable { name, .. } = expr.clone() {
                    expr = ASTNode::FunctionCall {
                        name,
                        arguments,
                        span: Span::unknown(),
                    };
                } else {
                    expr = ASTNode::Call {
                        callee: Box::new(expr),
                        arguments,
                        span: Span::unknown(),
                    };
                }
            } else if self.match_token(&TokenType::QUESTION) {
                let nt = self.peek_token();
                let is_ender = matches!(
                    nt,
                    TokenType::NEWLINE
                        | TokenType::EOF
                        | TokenType::RPAREN
                        | TokenType::COMMA
                        | TokenType::RBRACE
                );
                if !is_ender {
                    break;
                }
                self.advance();
                expr = ASTNode::QMarkPropagate {
                    expression: Box::new(expr),
                    span: Span::unknown(),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }
}
