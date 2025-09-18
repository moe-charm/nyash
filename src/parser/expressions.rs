#![allow(dead_code)]
/*!
 * Nyash Parser - Expression Parsing Module
 *
 * 式（Expression）の解析を担当するモジュール
 * 演算子の優先順位に従った再帰下降パーサー実装
 */

use super::common::ParserUtils;
use super::{NyashParser, ParseError};
use crate::ast::{ASTNode, Span, UnaryOperator};
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
        let mut expr = self.parse_ternary()?;

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

    /// 三項演算子: cond ? then : else
    /// Grammar (Phase 12.7): TernaryExpr = NullsafeExpr ( "?" Expr ":" Expr )?
    /// 実装: coalesce の上に差し込み、`cond ? a : b` を If式に変換する。
    fn parse_ternary(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_ternary()
    }

    /// デフォルト値（??）: x ?? y => peek x { null => y, else => x }
    fn parse_coalesce(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_coalesce()
    }

    /// OR演算子をパース: ||
    fn parse_or(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_or()
    }

    /// AND演算子をパース: &&
    fn parse_and(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_and()
    }

    /// ビットOR: |
    pub(crate) fn parse_bit_or(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_bit_or()
    }

    /// ビットXOR: ^
    fn parse_bit_xor(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_bit_xor()
    }

    /// ビットAND: &
    fn parse_bit_and(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_bit_and()
    }

    /// 等値演算子をパース: == !=
    pub(crate) fn parse_equality(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_equality()
    }

    /// 比較演算子をパース: < <= > >=
    fn parse_comparison(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_comparison()
    }

    /// 範囲演算子: a .. b => Range(a,b)
    fn parse_range(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_range()
    }

    /// 項をパース: + -
    fn parse_term(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_term()
    }

    /// シフトをパース: << >>
    fn parse_shift(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_shift()
    }

    /// 因子をパース: * /
    fn parse_factor(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_factor()
    }

    /// 単項演算子をパース
    pub(crate) fn parse_unary(&mut self) -> Result<ASTNode, ParseError> {
        // match式（peek置換）の先読み
        if self.match_token(&TokenType::MATCH) {
            return self.expr_parse_match();
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

    // parse_match_expr moved to expr/match_expr.rs as expr_parse_match

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
        self.expr_parse_call()
    }

    /// 基本式をパース: リテラル、変数、括弧、this、new、配列リテラル（糖衣）
    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        self.expr_parse_primary()
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
