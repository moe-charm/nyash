use crate::ast::{ASTNode, BinaryOperator, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(crate) fn expr_parse_shift(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_factor()?;
        loop {
            if self.match_token(&TokenType::ShiftLeft) {
                self.advance();
                let rhs = self.expr_parse_factor()?;
                expr = ASTNode::BinaryOp {
                    operator: BinaryOperator::Shl,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span: Span::unknown(),
                };
                continue;
            }
            if self.match_token(&TokenType::ShiftRight) {
                self.advance();
                let rhs = self.expr_parse_factor()?;
                expr = ASTNode::BinaryOp {
                    operator: BinaryOperator::Shr,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span: Span::unknown(),
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }
}
