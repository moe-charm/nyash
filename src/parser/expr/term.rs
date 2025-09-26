use crate::ast::{ASTNode, BinaryOperator, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(crate) fn expr_parse_term(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_shift()?;
        while self.match_token(&TokenType::PLUS) || self.match_token(&TokenType::MINUS) {
            let operator = match &self.current_token().token_type {
                TokenType::PLUS => BinaryOperator::Add,
                TokenType::MINUS => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.expr_parse_shift()?;
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
        Ok(expr)
    }
}
