/*!
 * Nyash Parser - Expression Parsing Module
 *
 * 式（Expression）の解析を担当するモジュール
 * 演算子の優先順位に従った再帰下降パーサー実装
 */

use super::common::ParserUtils;
use super::{NyashParser, ParseError};
use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span, UnaryOperator};
use crate::tokenizer::TokenType;

// Debug macros are now imported from the parent module via #[macro_export]
use crate::must_advance;

#[inline]
fn is_sugar_enabled() -> bool {
    crate::parser::sugar_gate::is_enabled()
}

impl NyashParser {
    /// 式をパース (演算子優先順位あり)
    pub(super) fn parse_expression(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_pipeline()
    }

    /// パイプライン演算子: lhs |> f(a,b) / lhs |> obj.m(a)
    /// 基本方針: 右辺が関数呼び出しなら先頭に lhs を挿入。メソッド呼び出しなら引数の先頭に lhs を挿入。
    fn parse_pipeline(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_coalesce()?;

        while self.match_token(&TokenType::PipeForward) {
            if !is_sugar_enabled() {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full for '|>'".to_string(),
                    line,
                });
            }
            // consume '|>'
            self.advance();

            // 右辺は「呼び出し系」の式を期待
            let rhs = self.parse_call()?;

            // 変換: rhs の形に応じて lhs を先頭引数として追加
            expr = match rhs {
                ASTNode::FunctionCall {
                    name,
                    mut arguments,
                    span,
                } => {
                    let mut new_args = Vec::with_capacity(arguments.len() + 1);
                    new_args.push(expr);
                    new_args.append(&mut arguments);
                    ASTNode::FunctionCall {
                        name,
                        arguments: new_args,
                        span,
                    }
                }
                ASTNode::MethodCall {
                    object,
                    method,
                    mut arguments,
                    span,
                } => {
                    let mut new_args = Vec::with_capacity(arguments.len() + 1);
                    new_args.push(expr);
                    new_args.append(&mut arguments);
                    ASTNode::MethodCall {
                        object,
                        method,
                        arguments: new_args,
                        span,
                    }
                }
                ASTNode::Variable { name, .. } => ASTNode::FunctionCall {
                    name,
                    arguments: vec![expr],
                    span: Span::unknown(),
                },
                ASTNode::FieldAccess { object, field, .. } => ASTNode::MethodCall {
                    object,
                    method: field,
                    arguments: vec![expr],
                    span: Span::unknown(),
                },
                ASTNode::Call {
                    callee,
                    mut arguments,
                    span,
                } => {
                    let mut new_args = Vec::with_capacity(arguments.len() + 1);
                    new_args.push(expr);
                    new_args.append(&mut arguments);
                    ASTNode::Call {
                        callee,
                        arguments: new_args,
                        span,
                    }
                }
                other => {
                    // 許容外: 関数/メソッド/変数/フィールド以外には適用不可
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: format!("callable after '|>' (got {})", other.info()),
                        line: self.current_token().line,
                    });
                }
            };
        }

        Ok(expr)
    }

    /// デフォルト値（??）: x ?? y => peek x { null => y, else => x }
    fn parse_coalesce(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_or()?;
        while self.match_token(&TokenType::QmarkQmark) {
            if !is_sugar_enabled() {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full for '??'".to_string(),
                    line,
                });
            }
            self.advance(); // consume '??'
            let rhs = self.parse_or()?;
            let scr = expr;
            expr = ASTNode::PeekExpr {
                scrutinee: Box::new(scr.clone()),
                arms: vec![(crate::ast::LiteralValue::Null, rhs)],
                else_expr: Box::new(scr),
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }

    /// OR演算子をパース: ||
    fn parse_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_and()?;

        while self.match_token(&TokenType::OR) {
            let operator = BinaryOperator::Or;
            self.advance();
            let right = self.parse_and()?;
            // Non-invasive syntax diff: record binop
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop("or");
                if !ok {
                    eprintln!("[GRAMMAR-DIFF][Parser] binop 'or' not allowed by syntax rules");
                }
            }
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
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop("and");
                if !ok {
                    eprintln!("[GRAMMAR-DIFF][Parser] binop 'and' not allowed by syntax rules");
                }
            }
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
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let name = match operator {
                    BinaryOperator::Equal => "eq",
                    BinaryOperator::NotEqual => "ne",
                    _ => "cmp",
                };
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop(name);
                if !ok {
                    eprintln!(
                        "[GRAMMAR-DIFF][Parser] binop '{}' not allowed by syntax rules",
                        name
                    );
                }
            }
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
        let mut expr = self.parse_range()?;

        while self.match_token(&TokenType::LESS)
            || self.match_token(&TokenType::LessEquals)
            || self.match_token(&TokenType::GREATER)
            || self.match_token(&TokenType::GreaterEquals)
        {
            let operator = match &self.current_token().token_type {
                TokenType::LESS => BinaryOperator::Less,
                TokenType::LessEquals => BinaryOperator::LessEqual,
                TokenType::GREATER => BinaryOperator::Greater,
                TokenType::GreaterEquals => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_range()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }

        Ok(expr)
    }

    /// 範囲演算子: a .. b => Range(a,b)
    fn parse_range(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_term()?;
        while self.match_token(&TokenType::RANGE) {
            if !is_sugar_enabled() {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full for '..'".to_string(),
                    line,
                });
            }
            self.advance(); // consume '..'
            let rhs = self.parse_term()?;
            expr = ASTNode::FunctionCall {
                name: "Range".to_string(),
                arguments: vec![expr, rhs],
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }

    /// 項をパース: + - >>
    fn parse_term(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&TokenType::PLUS)
            || self.match_token(&TokenType::MINUS)
            || self.match_token(&TokenType::ARROW)
        {
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
                if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                    let name = match operator {
                        BinaryOperator::Add => "add",
                        BinaryOperator::Subtract => "sub",
                        _ => "term",
                    };
                    let ok = crate::grammar::engine::get().syntax_is_allowed_binop(name);
                    if !ok {
                        eprintln!(
                            "[GRAMMAR-DIFF][Parser] binop '{}' not allowed by syntax rules",
                            name
                        );
                    }
                }
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

        while self.match_token(&TokenType::MULTIPLY)
            || self.match_token(&TokenType::DIVIDE)
            || self.match_token(&TokenType::MODULO)
        {
            let operator = match &self.current_token().token_type {
                TokenType::MULTIPLY => BinaryOperator::Multiply,
                TokenType::DIVIDE => BinaryOperator::Divide,
                TokenType::MODULO => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let name = match operator {
                    BinaryOperator::Multiply => "mul",
                    BinaryOperator::Divide => "div",
                    _ => "mod",
                };
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop(name);
                if !ok {
                    eprintln!(
                        "[GRAMMAR-DIFF][Parser] binop '{}' not allowed by syntax rules",
                        name
                    );
                }
            }
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
        // peek式の先読み
        if self.match_token(&TokenType::PEEK) {
            return self.parse_peek_expr();
        }
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

    /// peek式: peek <expr> { lit => arm ... else => arm }
    /// P1: arm は 式 または ブロック（{ ... } 最後の式が値）
    fn parse_peek_expr(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'peek'
        let scrutinee = self.parse_expression()?;
        self.consume(TokenType::LBRACE)?;

        let mut arms: Vec<(crate::ast::LiteralValue, ASTNode)> = Vec::new();
        let mut else_expr: Option<ASTNode> = None;

        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            while self.match_token(&TokenType::COMMA) || self.match_token(&TokenType::NEWLINE) {
                self.advance();
                self.skip_newlines();
            }
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // else or literal
            let is_else = matches!(self.current_token().token_type, TokenType::ELSE);
            if is_else {
                self.advance(); // consume 'else'
                self.consume(TokenType::FatArrow)?;
                // else アーム: ブロック or 式
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
                    self.parse_expression()?
                };
                else_expr = Some(expr);
            } else {
                // リテラルのみ許可（P0）
                let lit = self.parse_literal_only()?;
                self.consume(TokenType::FatArrow)?;
                // アーム: ブロック or 式
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
                    self.parse_expression()?
                };
                arms.push((lit, expr));
            }

            // 区切り（カンマや改行を許可）
            if self.match_token(&TokenType::COMMA) {
                self.advance();
            }
            if self.match_token(&TokenType::NEWLINE) {
                self.advance();
            }
        }

        self.consume(TokenType::RBRACE)?;
        let else_expr = else_expr.ok_or(ParseError::UnexpectedToken {
            found: self.current_token().token_type.clone(),
            expected: "else => <expr> in peek".to_string(),
            line: self.current_token().line,
        })?;

        Ok(ASTNode::PeekExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            else_expr: Box::new(else_expr),
            span: Span::unknown(),
        })
    }

    fn parse_literal_only(&mut self) -> Result<crate::ast::LiteralValue, ParseError> {
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
                        let mut _arg_count = 0;

                        while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                            must_advance!(self, _unused, "method call argument parsing");

                            arguments.push(self.parse_expression()?);
                            _arg_count += 1;

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
                // 後置 ?（Result伝播）
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

    /// 基本式をパース: リテラル、変数、括弧、this、new
    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::INCLUDE => {
                // Allow include as an expression: include "path"
                self.parse_include()
            }
            TokenType::STRING(s) => {
                let value = s.clone();
                self.advance();
                // Use plain literal to keep primitives simple in interpreter/VM paths
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
                // Deprecation: normalize 'this' to 'me'
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
                    let class_name = class_name.clone();
                    self.advance();

                    // 🔥 ジェネリクス型引数のパース (<IntegerBox, StringBox>)
                    let type_arguments = if self.match_token(&TokenType::LESS) {
                        self.advance(); // consume '<'
                        let mut args = Vec::new();

                        loop {
                            if let TokenType::IDENTIFIER(type_name) =
                                &self.current_token().token_type
                            {
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
                let parent = name.clone();
                self.advance();
                if self.match_token(&TokenType::DoubleColon) {
                    // Parent::method(args)
                    self.advance(); // consume '::'
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
                        must_advance!(self, _unused, "Parent::method call argument parsing");
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
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.consume(TokenType::RPAREN)?;
                Ok(expr)
            }

            TokenType::FN => {
                // 無名関数: fn (params?) { body }
                self.advance(); // consume 'fn'
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
                suggestion: format!(
                    "Use 'from {}.birth()' instead of 'from {}()'",
                    parent, parent
                ),
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
