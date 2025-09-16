use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, BinaryOperator, Span};

impl NyashParser {
    pub(crate) fn expr_parse_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_and()?;
        while self.match_token(&TokenType::OR) {
            let operator = BinaryOperator::Or;
            self.advance();
            let right = self.expr_parse_and()?;
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop("or");
                if !ok { eprintln!("[GRAMMAR-DIFF][Parser] binop 'or' not allowed by syntax rules"); }
            }
            expr = ASTNode::BinaryOp { operator, left: Box::new(expr), right: Box::new(right), span: Span::unknown() };
        }
        Ok(expr)
    }

    pub(crate) fn expr_parse_and(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.parse_bit_or()?;
        while self.match_token(&TokenType::AND) {
            let operator = BinaryOperator::And;
            self.advance();
            let right = self.parse_equality()?;
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop("and");
                if !ok { eprintln!("[GRAMMAR-DIFF][Parser] binop 'and' not allowed by syntax rules"); }
            }
            expr = ASTNode::BinaryOp { operator, left: Box::new(expr), right: Box::new(right), span: Span::unknown() };
        }
        Ok(expr)
    }
}
