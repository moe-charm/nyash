use crate::ast::{ASTNode, LiteralValue, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(crate) fn expr_parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::LBRACK => {
                let sugar_on = crate::parser::sugar_gate::is_enabled()
                    || std::env::var("NYASH_ENABLE_ARRAY_LITERAL").ok().as_deref() == Some("1");
                if !sugar_on {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full or NYASH_ENABLE_ARRAY_LITERAL=1".to_string(),
                        line,
                    });
                }
                self.advance();
                let mut elems: Vec<ASTNode> = Vec::new();
                while !self.match_token(&TokenType::RBRACK) && !self.is_at_end() {
                    crate::must_advance!(self, _unused, "array literal element parsing");
                    let el = self.parse_expression()?;
                    elems.push(el);
                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                    }
                }
                self.consume(TokenType::RBRACK)?;
                Ok(ASTNode::ArrayLiteral {
                    elements: elems,
                    span: Span::unknown(),
                })
            }
            TokenType::LBRACE => {
                let sugar_on = crate::parser::sugar_gate::is_enabled()
                    || std::env::var("NYASH_ENABLE_MAP_LITERAL").ok().as_deref() == Some("1");
                if !sugar_on {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full or NYASH_ENABLE_MAP_LITERAL=1".to_string(), line });
                }
                self.advance();
                let mut entries: Vec<(String, ASTNode)> = Vec::new();
                let sugar_level = std::env::var("NYASH_SYNTAX_SUGAR_LEVEL").ok();
                // デフォルトでIDENTIFIERキーを許可（basicが明示的に指定された場合のみ無効）
                let ident_key_on = std::env::var("NYASH_ENABLE_MAP_IDENT_KEY").ok().as_deref()
                    == Some("1")
                    || sugar_level.as_deref() != Some("basic");  // basic以外は全て許可（デフォルト含む）
                // skip_newlines削除: brace_depth > 0なので自動スキップされる
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    // skip_newlines削除: brace_depth > 0なので自動スキップされる
                    let key = match &self.current_token().token_type {
                        TokenType::STRING(s) => {
                            let v = s.clone();
                            self.advance();
                            v
                        }
                        TokenType::IDENTIFIER(id) if ident_key_on => {
                            let v = id.clone();
                            self.advance();
                            v
                        }
                        _ => {
                            let line = self.current_token().line;
                            return Err(ParseError::UnexpectedToken {
                                found: self.current_token().token_type.clone(),
                                expected: if ident_key_on {
                                    "string or identifier key in map literal".to_string()
                                } else {
                                    "string key in map literal (set NYASH_SYNTAX_SUGAR_LEVEL=full for identifier keys)".to_string()
                                },
                                line,
                            });
                        }
                    };
                    // skip_newlines削除: brace_depth > 0なので自動スキップされる
                    self.consume(TokenType::COLON)?;
                    // skip_newlines削除: brace_depth > 0なので自動スキップされる
                    let value_expr = self.parse_expression()?;
                    entries.push((key, value_expr));
                    // skip_newlines削除: brace_depth > 0なので自動スキップされる
                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                        // skip_newlines削除: brace_depth > 0なので自動スキップされる
                    }
                }
                // skip_newlines削除: brace_depth > 0なので自動スキップされる
                self.consume(TokenType::RBRACE)?;
                Ok(ASTNode::MapLiteral {
                    entries,
                    span: Span::unknown(),
                })
            }
            TokenType::INCLUDE => self.parse_include(),
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
                if std::env::var("NYASH_DEPRECATE_THIS").ok().as_deref() == Some("1") {
                    eprintln!(
                        "[deprecate:this] 'this' is deprecated; use 'me' instead (line {})",
                        self.current_token().line
                    );
                }
                self.advance();
                Ok(ASTNode::Me {
                    span: Span::unknown(),
                })
            }
            TokenType::ME => {
                self.advance();
                Ok(ASTNode::Me {
                    span: Span::unknown(),
                })
            }
            TokenType::NEW => {
                self.advance();
                if let TokenType::IDENTIFIER(class_name) = &self.current_token().token_type {
                    let class = class_name.clone();
                    self.advance();
                    let mut type_arguments: Vec<String> = Vec::new();
                    if self.match_token(&TokenType::LESS) {
                        self.advance();
                        loop {
                            if let TokenType::IDENTIFIER(tn) = &self.current_token().token_type {
                                type_arguments.push(tn.clone());
                                self.advance();
                            } else {
                                let line = self.current_token().line;
                                return Err(ParseError::UnexpectedToken {
                                    found: self.current_token().token_type.clone(),
                                    expected: "type argument".to_string(),
                                    line,
                                });
                            }
                            if self.match_token(&TokenType::COMMA) {
                                self.advance();
                                continue;
                            }
                            self.consume(TokenType::GREATER)?;
                            break;
                        }
                    }
                    self.consume(TokenType::LPAREN)?;
                    let mut arguments = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        crate::must_advance!(self, _unused, "new expression argument parsing");
                        arguments.push(self.parse_expression()?);
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    self.consume(TokenType::RPAREN)?;
                    Ok(ASTNode::New {
                        class,
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
            TokenType::FROM => self.parse_from_call(),
            TokenType::IDENTIFIER(name) => {
                let parent = name.clone();
                self.advance();
                if self.match_token(&TokenType::DoubleColon) {
                    self.advance();
                    let method = match &self.current_token().token_type {
                        TokenType::IDENTIFIER(m) => {
                            let s = m.clone();
                            self.advance();
                            s
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
                    };
                    self.consume(TokenType::LPAREN)?;
                    let mut arguments = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        crate::must_advance!(self, _unused, "Parent::method call argument parsing");
                        arguments.push(self.parse_expression()?);
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }
                    self.consume(TokenType::RPAREN)?;
                    Ok(ASTNode::FromCall {
                        parent,
                        method,
                        arguments,
                        span: Span::unknown(),
                    })
                } else {
                    Ok(ASTNode::Variable {
                        name: parent,
                        span: Span::unknown(),
                    })
                }
            }
            TokenType::LPAREN => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenType::RPAREN)?;
                Ok(expr)
            }
            TokenType::FN => {
                self.advance();
                let mut params: Vec<String> = Vec::new();
                if self.match_token(&TokenType::LPAREN) {
                    self.advance();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        if let TokenType::IDENTIFIER(p) = &self.current_token().token_type {
                            params.push(p.clone());
                            self.advance();
                            if self.match_token(&TokenType::COMMA) {
                                self.advance();
                            }
                        } else {
                            let line = self.current_token().line;
                            return Err(ParseError::UnexpectedToken {
                                found: self.current_token().token_type.clone(),
                                expected: "parameter name".to_string(),
                                line,
                            });
                        }
                    }
                    self.consume(TokenType::RPAREN)?;
                }
                self.consume(TokenType::LBRACE)?;
                let mut body: Vec<ASTNode> = Vec::new();
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();
                    if !self.match_token(&TokenType::RBRACE) {
                        body.push(self.parse_statement()?);
                    }
                }
                self.consume(TokenType::RBRACE)?;
                Ok(ASTNode::Lambda {
                    params,
                    body,
                    span: Span::unknown(),
                })
            }
            _ => {
                let line = self.current_token().line;
                Err(ParseError::InvalidExpression { line })
            }
        }
    }
}
