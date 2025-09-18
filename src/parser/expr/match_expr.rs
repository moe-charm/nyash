use crate::ast::{ASTNode, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// match式: match <expr> { lit[ '|' lit ]* => <expr|block>, ..., _ => <expr|block> }
    /// MVP: リテラルパターン＋OR＋デフォルト(_) のみ。アーム本体は式またはブロック。
    pub(crate) fn expr_parse_match(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'match'
        // Scrutinee: MVPでは primary/call に限定（表現力は十分）
        let scrutinee = self.expr_parse_primary()?;
        self.consume(TokenType::LBRACE)?;

        let mut arms: Vec<(crate::ast::LiteralValue, ASTNode)> = Vec::new();
        let mut default_expr: Option<ASTNode> = None;

        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            while self.match_token(&TokenType::COMMA) || self.match_token(&TokenType::NEWLINE) {
                self.advance();
                self.skip_newlines();
            }
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // default '_' or literal arm
            let is_default = matches!(self.current_token().token_type, TokenType::IDENTIFIER(ref s) if s == "_");
            if is_default {
                self.advance(); // consume '_'
                self.consume(TokenType::FatArrow)?;
                let expr = if self.match_token(&TokenType::LBRACE) {
                    // ブロックを式として扱う（最後の文の値が返る）
                    self.advance(); // consume '{'
                    let mut stmts: Vec<ASTNode> = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            stmts.push(self.parse_statement()?);
                        }
                    }
                    self.consume(TokenType::RBRACE)?;
                    ASTNode::Program {
                        statements: stmts,
                        span: Span::unknown(),
                    }
                } else {
                    // MVP: アームは primary/call を優先
                    self.expr_parse_primary()?
                };
                default_expr = Some(expr);
            } else {
                // リテラル（OR結合可）
                let mut lits: Vec<crate::ast::LiteralValue> = Vec::new();
                let first = self.lit_only_for_match()?;
                lits.push(first);
                while self.match_token(&TokenType::BitOr) {
                    self.advance(); // consume '|'
                    let nxt = self.lit_only_for_match()?;
                    lits.push(nxt);
                }
                self.consume(TokenType::FatArrow)?;
                let expr = if self.match_token(&TokenType::LBRACE) {
                    self.advance(); // consume '{'
                    let mut stmts: Vec<ASTNode> = Vec::new();
                    while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                        self.skip_newlines();
                        if !self.match_token(&TokenType::RBRACE) {
                            stmts.push(self.parse_statement()?);
                        }
                    }
                    self.consume(TokenType::RBRACE)?;
                    ASTNode::Program {
                        statements: stmts,
                        span: Span::unknown(),
                    }
                } else {
                    self.expr_parse_primary()?
                };
                for lit in lits {
                    arms.push((lit.clone(), expr.clone()));
                }
            }

            // 区切り（カンマや改行を許可）
            while self.match_token(&TokenType::COMMA) || self.match_token(&TokenType::NEWLINE) {
                self.advance();
            }
            self.skip_newlines();
        }

        self.consume(TokenType::RBRACE)?;
        let else_expr = default_expr.ok_or(ParseError::UnexpectedToken {
            found: self.current_token().token_type.clone(),
            expected: "_ => <expr> in match".to_string(),
            line: self.current_token().line,
        })?;

        // 既存の Lower を活用するため PeekExpr に落とす
        Ok(ASTNode::PeekExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            else_expr: Box::new(else_expr),
            span: Span::unknown(),
        })
    }

    // match 用の最小リテラルパーサ（式は受け付けない）
    fn lit_only_for_match(&mut self) -> Result<crate::ast::LiteralValue, ParseError> {
        match &self.current_token().token_type {
            TokenType::STRING(s) => {
                let v = crate::ast::LiteralValue::String(s.clone());
                self.advance();
                Ok(v)
            }
            TokenType::NUMBER(n) => {
                let v = crate::ast::LiteralValue::Integer(*n);
                self.advance();
                Ok(v)
            }
            TokenType::FLOAT(f) => {
                let v = crate::ast::LiteralValue::Float(*f);
                self.advance();
                Ok(v)
            }
            TokenType::TRUE => {
                self.advance();
                Ok(crate::ast::LiteralValue::Bool(true))
            }
            TokenType::FALSE => {
                self.advance();
                Ok(crate::ast::LiteralValue::Bool(false))
            }
            TokenType::NULL => {
                self.advance();
                Ok(crate::ast::LiteralValue::Null)
            }
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "literal".to_string(),
                    line,
                })
            }
        }
    }
}
