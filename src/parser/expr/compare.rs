use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, BinaryOperator, Span};

impl NyashParser {
    pub(crate) fn expr_parse_equality(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_comparison()?;
        while self.match_token(&TokenType::EQUALS) || self.match_token(&TokenType::NotEquals) {
            let operator = match &self.current_token().token_type {
                TokenType::EQUALS => BinaryOperator::Equal,
                TokenType::NotEquals => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.expr_parse_comparison()?;
            if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
                let name = match operator { BinaryOperator::Equal=>"eq", BinaryOperator::NotEqual=>"ne", _=>"cmp" };
                let ok = crate::grammar::engine::get().syntax_is_allowed_binop(name);
                if !ok { eprintln!("[GRAMMAR-DIFF][Parser] binop '{}' not allowed by syntax rules", name); }
            }
            expr = ASTNode::BinaryOp { operator, left: Box::new(expr), right: Box::new(right), span: Span::unknown() };
        }
        Ok(expr)
    }

    pub(crate) fn expr_parse_comparison(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_range()?;
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
            let right = self.expr_parse_range()?;
            expr = ASTNode::BinaryOp { operator, left: Box::new(expr), right: Box::new(right), span: Span::unknown() };
        }
        Ok(expr)
    }
}

