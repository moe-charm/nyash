use crate::ast::{ASTNode, BinaryOperator, LiteralValue, Span};
use crate::parser::cursor::TokenCursor;
use crate::parser::ParseError;
use crate::tokenizer::TokenType;
use crate::parser::sugar_gate;

/// TokenCursorを使用した式パーサー（実験的実装）
pub struct ExprParserWithCursor;

impl ExprParserWithCursor {
    /// 式をパース（TokenCursor版）
    pub fn parse_expression(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        // 式モードで実行（改行を自動的にスキップ）
        cursor.with_expr_mode(|c| {
            Self::parse_or_expr(c)
        })
    }

    /// OR式をパース
    fn parse_or_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_and_expr(cursor)?;

        while cursor.match_token(&TokenType::OR) {
            let op_line = cursor.current().line;
            cursor.advance();
            let right = Self::parse_and_expr(cursor)?;
            left = ASTNode::BinaryOp {
                operator: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(op_line, 0, op_line, 0),
            };
        }

        Ok(left)
    }

    /// AND式をパース
    fn parse_and_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_comparison_expr(cursor)?;

        while cursor.match_token(&TokenType::AND) {
            let op_line = cursor.current().line;
            cursor.advance();
            let right = Self::parse_comparison_expr(cursor)?;
            left = ASTNode::BinaryOp {
                operator: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(op_line, 0, op_line, 0),
            };
        }

        Ok(left)
    }

    /// 比較式をパース
    fn parse_comparison_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_additive_expr(cursor)?;

        while let Some(op) = Self::match_comparison_op(cursor) {
            let op_line = cursor.current().line;
            cursor.advance();
            let right = Self::parse_additive_expr(cursor)?;
            left = ASTNode::BinaryOp {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(op_line, 0, op_line, 0),
            };
        }

        Ok(left)
    }

    /// 比較演算子をチェック
    fn match_comparison_op(cursor: &TokenCursor) -> Option<BinaryOperator> {
        match &cursor.current().token_type {
            TokenType::EQUALS => Some(BinaryOperator::Equal),
            TokenType::NotEquals => Some(BinaryOperator::NotEqual),
            TokenType::LESS => Some(BinaryOperator::Less),
            TokenType::LessEquals => Some(BinaryOperator::LessEqual),
            TokenType::GREATER => Some(BinaryOperator::Greater),
            TokenType::GreaterEquals => Some(BinaryOperator::GreaterEqual),
            _ => None,
        }
    }

    /// 加算式をパース
    fn parse_additive_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_multiplicative_expr(cursor)?;

        while let Some(op) = Self::match_additive_op(cursor) {
            let op_line = cursor.current().line;
            cursor.advance();
            let right = Self::parse_multiplicative_expr(cursor)?;
            left = ASTNode::BinaryOp {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(op_line, 0, op_line, 0),
            };
        }

        Ok(left)
    }

    /// 加算演算子をチェック
    fn match_additive_op(cursor: &TokenCursor) -> Option<BinaryOperator> {
        match &cursor.current().token_type {
            TokenType::PLUS => Some(BinaryOperator::Add),
            TokenType::MINUS => Some(BinaryOperator::Subtract),
            _ => None,
        }
    }

    /// 乗算式をパース
    fn parse_multiplicative_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        let mut left = Self::parse_primary_expr(cursor)?;

        while let Some(op) = Self::match_multiplicative_op(cursor) {
            let op_line = cursor.current().line;
            cursor.advance();
            let right = Self::parse_primary_expr(cursor)?;
            left = ASTNode::BinaryOp {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(op_line, 0, op_line, 0),
            };
        }

        Ok(left)
    }

    /// 乗算演算子をチェック
    fn match_multiplicative_op(cursor: &TokenCursor) -> Option<BinaryOperator> {
        match &cursor.current().token_type {
            TokenType::MULTIPLY => Some(BinaryOperator::Multiply),
            TokenType::DIVIDE => Some(BinaryOperator::Divide),
            TokenType::MODULO => Some(BinaryOperator::Modulo),
            _ => None,
        }
    }

    /// プライマリ式をパース
    fn parse_primary_expr(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        match &cursor.current().token_type.clone() {
            TokenType::LBRACK => {
                // Array literal (sugar gated)
                let sugar_on = sugar_gate::is_enabled()
                    || std::env::var("NYASH_ENABLE_ARRAY_LITERAL").ok().as_deref() == Some("1");
                if !sugar_on {
                    let line = cursor.current().line;
                    return Err(ParseError::UnexpectedToken {
                        found: cursor.current().token_type.clone(),
                        expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full or NYASH_ENABLE_ARRAY_LITERAL=1".to_string(),
                        line,
                    });
                }
                cursor.advance(); // consume '['
                let mut elements: Vec<ASTNode> = Vec::new();
                while !cursor.match_token(&TokenType::RBRACK) && !cursor.is_at_end() {
                    let el = Self::parse_expression(cursor)?;
                    elements.push(el);
                    if cursor.match_token(&TokenType::COMMA) {
                        cursor.advance();
                    }
                }
                cursor.consume(TokenType::RBRACK)?;
                Ok(ASTNode::ArrayLiteral { elements, span: Span::unknown() })
            }
            TokenType::NUMBER(n) => {
                let value = *n;
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Integer(value),
                    span: Span::unknown(),
                })
            }
            TokenType::STRING(s) => {
                let value = s.clone();
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::String(value),
                    span: Span::unknown(),
                })
            }
            TokenType::TRUE => {
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Bool(true),
                    span: Span::unknown(),
                })
            }
            TokenType::FALSE => {
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Bool(false),
                    span: Span::unknown(),
                })
            }
            TokenType::NULL => {
                cursor.advance();
                Ok(ASTNode::Literal {
                    value: LiteralValue::Null,
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
            TokenType::LPAREN => {
                cursor.advance();
                let expr = Self::parse_expression(cursor)?;
                cursor.consume(TokenType::RPAREN)?;
                Ok(expr)
            }
            TokenType::LBRACE => {
                // オブジェクトリテラル（改行対応済み）
                Self::parse_object_literal(cursor)
            }
            _ => {
                let line = cursor.current().line;
                Err(ParseError::InvalidExpression { line })
            }
        }
    }

    /// オブジェクトリテラルをパース（TokenCursor版）
    fn parse_object_literal(cursor: &mut TokenCursor) -> Result<ASTNode, ParseError> {
        cursor.consume(TokenType::LBRACE)?;
        let mut entries = Vec::new();

        while !cursor.match_token(&TokenType::RBRACE) && !cursor.is_at_end() {
            // キーをパース（STRING or IDENTIFIER）
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
            let value = Self::parse_expression(cursor)?;
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
}
