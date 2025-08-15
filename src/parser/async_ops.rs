/*!
 * Nyash Parser - Async Operations Module
 * 
 * 非同期処理（await）の解析を担当するモジュール
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, Span};
use super::{NyashParser, ParseError};

impl NyashParser {
    /// await式をパース
    pub(super) fn parse_await(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'await'
        let expression = self.parse_unary()?; // 再帰的にパース
        Ok(ASTNode::AwaitExpression {
            expression: Box::new(expression),
            span: Span::unknown(),
        })
    }
}