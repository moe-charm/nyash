use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
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

        enum MatchArm {
            Lit(Vec<LiteralValue>, ASTNode),
            Type { ty: String, bind: String, body: ASTNode },
            Default(ASTNode),
        }

        let mut arms_any: Vec<MatchArm> = Vec::new();
        let mut saw_type_arm = false;
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

            // default '_' or type/literal arm
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
                default_expr = Some(expr.clone());
                arms_any.push(MatchArm::Default(expr));
            } else {
                // Type pattern? IDENT '(' IDENT ')'
                let mut handled = false;
                if let TokenType::IDENTIFIER(type_name) = self.current_token().token_type.clone() {
                    if self.peek_token() == &TokenType::LPAREN
                        && matches!(self.peek_nth_token(2), TokenType::IDENTIFIER(_))
                        && self.peek_nth_token(3) == &TokenType::RPAREN
                    {
                        // consume TypeName ( IDENT ), capture binding name
                        let ty = type_name.clone();
                        self.advance(); // TypeName
                        self.consume(TokenType::LPAREN)?;
                        let bind = match self.current_token().token_type.clone() {
                            TokenType::IDENTIFIER(s) => {
                                self.advance();
                                s
                            }
                            other => {
                                return Err(ParseError::UnexpectedToken {
                                    found: other,
                                    expected: "identifier".to_string(),
                                    line: self.current_token().line,
                                })
                            }
                        };
                        self.consume(TokenType::RPAREN)?;
                        self.consume(TokenType::FatArrow)?;
                        let body = if self.match_token(&TokenType::LBRACE) {
                            self.advance(); // consume '{'
                            let mut stmts: Vec<ASTNode> = Vec::new();
                            while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                                self.skip_newlines();
                                if !self.match_token(&TokenType::RBRACE) {
                                    stmts.push(self.parse_statement()?);
                                }
                            }
                            self.consume(TokenType::RBRACE)?;
                            ASTNode::Program { statements: stmts, span: Span::unknown() }
                        } else {
                            self.expr_parse_primary()?
                        };
                        // type arm parsed
                        arms_any.push(MatchArm::Type { ty, bind, body });
                        saw_type_arm = true;
                        handled = true;
                    }
                }
                if !handled {
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
                        ASTNode::Program { statements: stmts, span: Span::unknown() }
                    } else {
                        self.expr_parse_primary()?
                    };
                    arms_any.push(MatchArm::Lit(lits, expr));
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

        if !saw_type_arm {
            // 既存の Lower を活用するため PeekExpr に落とす（型パターンが無い場合のみ）
            let mut lit_arms: Vec<(LiteralValue, ASTNode)> = Vec::new();
            for arm in arms_any.into_iter() {
                match arm {
                    MatchArm::Lit(lits, expr) => {
                        for lit in lits.into_iter() {
                            lit_arms.push((lit, expr.clone()));
                        }
                    }
                    MatchArm::Default(_) => { /* handled via else_expr above */ }
                    MatchArm::Type { .. } => unreachable!(),
                }
            }
            return Ok(ASTNode::PeekExpr {
                scrutinee: Box::new(scrutinee),
                arms: lit_arms,
                else_expr: Box::new(else_expr),
                span: Span::unknown(),
            });
        }

        // 型パターンを含む: ASTで if 連鎖へ合成
        // 1) scrutinee を一度だけ評価しローカルに束縛
        let scr_var = "__ny_match_scrutinee".to_string();
        let scr_local = ASTNode::Local {
            variables: vec![scr_var.clone()],
            initial_values: vec![Some(Box::new(scrutinee))],
            span: Span::unknown(),
        };

        // 2) アーム順に If 連鎖を構築
        let mut else_node: ASTNode = else_expr;
        // Wrap else body in Program for uniformity
        else_node = ASTNode::Program { statements: vec![else_node], span: Span::unknown() };

        // Process arms in reverse to build nested If
        for arm in arms_any.into_iter().rev() {
            match arm {
                MatchArm::Default(_) => {
                    // already handled as else_node
                }
                MatchArm::Lit(lits, body) => {
                    // condition: (scr == lit1) || (scr == lit2) || ...
                    let mut cond: Option<ASTNode> = None;
                    for lit in lits.into_iter() {
                        let eq = ASTNode::BinaryOp {
                            operator: BinaryOperator::Equal,
                            left: Box::new(ASTNode::Variable { name: scr_var.clone(), span: Span::unknown() }),
                            right: Box::new(ASTNode::Literal { value: lit, span: Span::unknown() }),
                            span: Span::unknown(),
                        };
                        cond = Some(match cond {
                            None => eq,
                            Some(prev) => ASTNode::BinaryOp {
                                operator: BinaryOperator::Or,
                                left: Box::new(prev),
                                right: Box::new(eq),
                                span: Span::unknown(),
                            },
                        });
                    }
                    let then_prog = ASTNode::Program { statements: vec![body], span: Span::unknown() };
                    else_node = ASTNode::If {
                        condition: Box::new(cond.expect("literal arm must have at least one literal")),
                        then_body: match then_prog { ASTNode::Program { statements, .. } => statements, _ => unreachable!() },
                        else_body: Some(match else_node.clone() { ASTNode::Program { statements, .. } => statements, other => vec![other] }),
                        span: Span::unknown(),
                    };
                }
                MatchArm::Type { ty, bind, body } => {
                    // condition: scr.is("Type")
                    let is_call = ASTNode::MethodCall {
                        object: Box::new(ASTNode::Variable { name: scr_var.clone(), span: Span::unknown() }),
                        method: "is".to_string(),
                        arguments: vec![ASTNode::Literal { value: LiteralValue::String(ty.clone()), span: Span::unknown() }],
                        span: Span::unknown(),
                    };
                    // then: local bind = scr.as("Type"); <body>
                    let cast = ASTNode::MethodCall {
                        object: Box::new(ASTNode::Variable { name: scr_var.clone(), span: Span::unknown() }),
                        method: "as".to_string(),
                        arguments: vec![ASTNode::Literal { value: LiteralValue::String(ty.clone()), span: Span::unknown() }],
                        span: Span::unknown(),
                    };
                    let bind_local = ASTNode::Local {
                        variables: vec![bind.clone()],
                        initial_values: vec![Some(Box::new(cast))],
                        span: Span::unknown(),
                    };
                    let then_prog = ASTNode::Program { statements: vec![bind_local, body], span: Span::unknown() };
                    else_node = ASTNode::If {
                        condition: Box::new(is_call),
                        then_body: match then_prog { ASTNode::Program { statements, .. } => statements, _ => unreachable!() },
                        else_body: Some(match else_node.clone() { ASTNode::Program { statements, .. } => statements, other => vec![other] }),
                        span: Span::unknown(),
                    };
                }
            }
        }

        // 3) 全体を Program で包み、scrutinee の一回評価を保証
        Ok(ASTNode::Program {
            statements: vec![scr_local, else_node],
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
