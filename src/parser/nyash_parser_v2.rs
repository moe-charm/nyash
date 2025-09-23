/*!
 * NyashParser v2 - TokenCursorベースの新パーサー
 *
 * 改行処理を完全自動化した次世代パーサー
 * skip_newlines()の手動呼び出しを完全排除
 */

use crate::ast::{ASTNode, Span};
use crate::parser::cursor::{TokenCursor, NewlineMode};
use crate::parser::ParseError;
use crate::tokenizer::{Token, TokenType};
use std::collections::{HashMap, HashSet};

/// TokenCursorベースの新パーサー
pub struct NyashParserV2<'a> {
    cursor: TokenCursor<'a>,
    static_box_dependencies: HashMap<String, HashSet<String>>,
    debug_fuel: Option<usize>,
}

impl<'a> NyashParserV2<'a> {
    /// 新しいパーサーを作成
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
            static_box_dependencies: HashMap::new(),
            debug_fuel: Some(100_000),
        }
    }

    /// プログラムをパース（エントリーポイント）
    pub fn parse_program(&mut self) -> Result<ASTNode, ParseError> {
        let mut statements = Vec::new();

        // 文モードでパース（改行が文の区切り）
        while !self.cursor.is_at_end() {
            statements.push(self.parse_statement()?);

            // 文の区切り（改行やセミコロン）は自動処理
            while self.cursor.match_token(&TokenType::NEWLINE)
                || self.cursor.match_token(&TokenType::SEMICOLON) {
                self.cursor.advance();
            }
        }

        Ok(ASTNode::Program {
            statements,
            span: Span::unknown(),
        })
    }

    /// 文をパース
    pub fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        // 文モードで実行（改行を文の区切りとして扱う）
        match &self.cursor.current().token_type {
            TokenType::LOCAL => self.parse_local_declaration(),
            TokenType::IF => self.parse_if_statement(),
            TokenType::LOOP => self.parse_loop_statement(),
            TokenType::RETURN => self.parse_return_statement(),
            TokenType::BREAK => self.parse_break_statement(),
            TokenType::CONTINUE => self.parse_continue_statement(),
            _ => {
                // 式文（代入や関数呼び出しなど）
                self.parse_expression_statement()
            }
        }
    }

    /// 式をパース
    pub fn parse_expression(&mut self) -> Result<ASTNode, ParseError> {
        // 式モードで実行（改行を自動的にスキップ）
        self.cursor.with_expr_mode(|c| {
            Self::parse_or_expression_internal(c)
        })
    }

    /// OR式をパース（内部実装）
    fn parse_or_expression_internal(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_and_expression_internal(cursor)?;

        while cursor.match_token(&TokenType::OR) {
            cursor.advance();
            let right = Self::parse_and_expression_internal(cursor)?;
            left = ASTNode::BinaryOp {
                operator: crate::ast::BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }

        Ok(left)
    }

    /// AND式をパース（内部実装）
    fn parse_and_expression_internal(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_primary_expression_internal(cursor)?;

        while cursor.match_token(&TokenType::AND) {
            cursor.advance();
            let right = Self::parse_primary_expression_internal(cursor)?;
            left = ASTNode::BinaryOp {
                operator: crate::ast::BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }

        Ok(left)
    }

    /// プライマリ式をパース（内部実装）
    fn parse_primary_expression_internal(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        match &cursor.current().token_type.clone() {
            TokenType::NUMBER(n) => {
                let value = *n;
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: crate::ast::LiteralValue::Integer(value),
                    span: Span::unknown(),
                })
            }
            TokenType::STRING(s) => {
                let value = s.clone();
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: crate::ast::LiteralValue::String(value),
                    span: Span::unknown(),
                })
            }
            TokenType::TRUE => {
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: crate::ast::LiteralValue::Bool(true),
                    span: Span::unknown(),
                })
            }
            TokenType::FALSE => {
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: crate::ast::LiteralValue::Bool(false),
                    span: Span::unknown(),
                })
            }
            TokenType::IDENTIFIER(name) => {
                let name = name.clone();
                cursor.advance();
                Ok(ASTNode::Variable {
                    name,
                    span: Span::unknown(),
                })
            }
            TokenType::LBRACE => {
                // オブジェクトリテラル（改行は自動処理）
                Self::parse_object_literal_internal(cursor)
            }
            TokenType::LPAREN => {
                cursor.advance();
                let expr = Self::parse_or_expression_internal(cursor)?;
                cursor.consume(TokenType::RPAREN)?;
                Ok(expr)
            }
            _ => {
                let line = cursor.current().line;
                Err(ParseError::InvalidExpression { line })
            }
        }
    }

    /// オブジェクトリテラルをパース（改行完全自動化）
    fn parse_object_literal_internal(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        cursor.consume(TokenType::LBRACE)?;
        let mut entries = Vec::new();

        // ブレース内は改行が自動的にスキップされる！
        while !cursor.match_token(&TokenType::RBRACE) && !cursor.is_at_end() {
            // キーをパース
            let key = match &cursor.current().token_type {
                TokenType::STRING(s) => {
                    let k = s.clone();
                    cursor.advance();
                    k
                }
                TokenType::IDENTIFIER(id) => {
                    let k = id.clone();
                    cursor.advance();
                    k
                }
                _ => {
                    let line = cursor.current().line;
                    return Err(ParseError::UnexpectedToken {
                        found: cursor.current().token_type.clone(),
                        expected: "string or identifier key".to_string(),
                        line,
                    });
                }
            };

            cursor.consume(TokenType::COLON)?;
            let value = Self::parse_or_expression_internal(cursor)?;
            entries.push((key, value));

            if cursor.match_token(&TokenType::COMMA) {
                cursor.advance();
            }
        }

        cursor.consume(TokenType::RBRACE)?;
        Ok(ASTNode::MapLiteral {
            entries,
            span: Span::unknown(),
        })
    }

    // 以下、各種文のパースメソッド（スタブ）
    fn parse_local_declaration(&mut self) -> Result<ASTNode, ParseError> {
        todo!("local宣言のパース実装")
    }

    fn parse_if_statement(&mut self) -> Result<ASTNode, ParseError> {
        todo!("if文のパース実装")
    }

    fn parse_loop_statement(&mut self) -> Result<ASTNode, ParseError> {
        todo!("loop文のパース実装")
    }

    fn parse_return_statement(&mut self) -> Result<ASTNode, ParseError> {
        todo!("return文のパース実装")
    }

    fn parse_break_statement(&mut self) -> Result<ASTNode, ParseError> {
        todo!("break文のパース実装")
    }

    fn parse_continue_statement(&mut self) -> Result<ASTNode, ParseError> {
        todo!("continue文のパース実装")
    }

    fn parse_expression_statement(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_expression()
    }
}