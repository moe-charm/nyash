use crate::ast::{ASTNode, BinaryOperator, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(crate) fn expr_parse_bit_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_bit_xor()?;
        while self.match_token(&TokenType::BitOr) {
            let operator = BinaryOperator::BitOr;
            self.advance();
            let right = self.expr_parse_bit_xor()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }

    pub(crate) fn expr_parse_bit_xor(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_bit_and()?;
        while self.match_token(&TokenType::BitXor) {
            let operator = BinaryOperator::BitXor;
            self.advance();
            let right = self.expr_parse_bit_and()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }

    pub(crate) fn expr_parse_bit_and(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_equality()?;
        while self.match_token(&TokenType::BitAnd) {
            let operator = BinaryOperator::BitAnd;
            self.advance();
            let right = self.expr_parse_equality()?;
            expr = ASTNode::BinaryOp {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }
}
