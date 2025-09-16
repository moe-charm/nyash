use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, BinaryOperator, Span};

impl NyashParser {
    pub(crate) fn expr_parse_factor(&mut self) -> Result<ASTNode, ParseError> {
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
                let name = match operator { BinaryOperator::Multiply=>"mul", BinaryOperator::Divide=>"div", _=>"mod" };
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop(name);
                if !ok { eprintln!("[GRAMMAR-DIFF][Parser] binop '{}' not allowed by syntax rules", name); }
            }
            expr = ASTNode::BinaryOp { operator, left: Box::new(expr), right: Box::new(right), span: Span::unknown() };
        }
        Ok(expr)
    }
}

