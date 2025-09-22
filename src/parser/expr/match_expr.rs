use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// match式: match <expr> { lit[ '|' lit ]* => <expr|block>, ..., _ => <expr|block> }
    /// MVP: リテラルパターン＋OR＋デフォルト(_) のみ。アーム本体は式またはブロック。
    pub(crate) fn expr_parse_match(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'match'
        // Scrutinee: 通常の式を受理（演算子優先順位を含む）
        let scrutinee = self.parse_expression()?;
        self.consume(TokenType::LBRACE)?;

        enum MatchArm {
            Lit { lits: Vec<LiteralValue>, guard: Option<ASTNode>, body: ASTNode },
            Type { ty: String, bind: String, guard: Option<ASTNode>, body: ASTNode },
            Default,
        }

        let mut arms_any: Vec<MatchArm> = Vec::new();
        let mut saw_type_arm = false;
        let mut saw_guard = false;
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
                // MVP: default '_' does not accept guard
                if self.match_token(&TokenType::IF) {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "'=>' (guard is not allowed for default arm)".to_string(),
                        line,
                    });
                }
                self.consume(TokenType::FatArrow)?;
                let expr = if self.match_token(&TokenType::LBRACE) {
                    if self.is_object_literal() {
                        // オブジェクトリテラルとして処理
                        self.parse_expression()?
                    } else {
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
                    }
                } else {
                    // 値アームは通常の式全体を受理
                    self.parse_expression()?
                };
                default_expr = Some(expr.clone());
                arms_any.push(MatchArm::Default);
            } else {
                // arm head
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
                        // Optional guard
                        let guard = if self.match_token(&TokenType::IF) {
                            self.advance();
                            let g = self.parse_expression()?;
                            saw_guard = true;
                            Some(g)
                        } else { None };
                        self.consume(TokenType::FatArrow)?;
                        let body = if self.match_token(&TokenType::LBRACE) {
                            if self.is_object_literal() {
                                // オブジェクトリテラルとして処理
                                self.parse_expression()?
                            } else {
                                self.advance(); // consume '{'
                                let mut stmts: Vec<ASTNode> = Vec::new();
                                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                                    self.skip_newlines();
                                    if !self.match_token(&TokenType::RBRACE) {
                                        let st = self.parse_statement()?;
                                        stmts.push(st);
                                    }
                                }
                                self.consume(TokenType::RBRACE)?;
                                ASTNode::Program { statements: stmts, span: Span::unknown() }
                            }
                        } else {
                            // 値アームは通常の式全体を受理
                            self.parse_expression()?
                        };
                        // type arm parsed
                        arms_any.push(MatchArm::Type { ty, bind, guard, body });
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
                    // Optional guard before '=>'
                    let guard = if self.match_token(&TokenType::IF) {
                        self.advance();
                        let g = self.parse_expression()?;
                        saw_guard = true;
                        Some(g)
                    } else { None };
                    self.consume(TokenType::FatArrow)?;
                    let expr = if self.match_token(&TokenType::LBRACE) {
                        if self.is_object_literal() {
                            // オブジェクトリテラルとして処理
                            self.parse_expression()?
                        } else {
                            self.advance(); // consume '{'
                            let mut stmts: Vec<ASTNode> = Vec::new();
                            while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                                self.skip_newlines();
                                if !self.match_token(&TokenType::RBRACE) {
                                    let st = self.parse_statement()?;
                                    stmts.push(st);
                                }
                            }
                            self.consume(TokenType::RBRACE)?;
                            ASTNode::Program { statements: stmts, span: Span::unknown() }
                        }
                    } else {
                        // 値アームは通常の式全体を受理
                        self.parse_expression()?
                    };
                    arms_any.push(MatchArm::Lit { lits, guard, body: expr });
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

        if !saw_type_arm && !saw_guard {
            // 既存の Lower を活用するため PeekExpr に落とす（型パターンが無い場合のみ）
            let mut lit_arms: Vec<(LiteralValue, ASTNode)> = Vec::new();
            for arm in arms_any.into_iter() {
                match arm {
                    MatchArm::Lit { lits, guard: _, body } => {
                        for lit in lits.into_iter() {
                            lit_arms.push((lit, body.clone()));
                        }
                    }
                    MatchArm::Default => { /* handled via else_expr above */ }
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
        // 1) scrutinee を一度だけ評価しローカルに束縛（衝突回避のため gensym 風の名前を付与）
        let scr_var = format!("__ny_match_scrutinee_{}", self.current());
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
                MatchArm::Default => {
                    // already handled as else_node
                }
                MatchArm::Lit { lits, guard, body } => {
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
                    let else_statements = match else_node.clone() { ASTNode::Program { statements, .. } => statements, other => vec![other] };
                    let then_body_statements = if let Some(g) = guard {
                        // Nested guard: if g then body else else_node
                        let guard_if = ASTNode::If {
                            condition: Box::new(g),
                            then_body: vec![body],
                            else_body: Some(else_statements.clone()),
                            span: Span::unknown(),
                        };
                        vec![guard_if]
                    } else {
                        vec![body]
                    };
                    else_node = ASTNode::If {
                        condition: Box::new(cond.expect("literal arm must have at least one literal")),
                        then_body: then_body_statements,
                        else_body: Some(else_statements),
                        span: Span::unknown(),
                    };
                }
                MatchArm::Type { ty, bind, guard, body } => {
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
                    let else_statements = match else_node.clone() { ASTNode::Program { statements, .. } => statements, other => vec![other] };
                    let then_body_statements = if let Some(g) = guard {
                        // After binding, check guard then branch to body else fallthrough to else_node
                        let guard_if = ASTNode::If {
                            condition: Box::new(g),
                            then_body: vec![body],
                            else_body: Some(else_statements.clone()),
                            span: Span::unknown(),
                        };
                        vec![bind_local, guard_if]
                    } else {
                        vec![bind_local, body]
                    };
                    else_node = ASTNode::If {
                        condition: Box::new(is_call),
                        then_body: then_body_statements,
                        else_body: Some(else_statements),
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

    /// オブジェクトリテラル判定: { IDENTIFIER : または { STRING : の場合はtrue
    fn is_object_literal(&self) -> bool {
        // 副作用を避けるためcurrent_token()を使用
        if !matches!(self.current_token().token_type, TokenType::LBRACE) {
            return false;
        }
        match self.peek_token() {
            TokenType::IDENTIFIER(_) | TokenType::STRING(_) => {
                matches!(self.peek_nth_token(2), TokenType::COLON)
            }
            _ => false
        }
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
