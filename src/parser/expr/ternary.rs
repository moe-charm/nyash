use crate::ast::{ASTNode, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

#[inline]
#[allow(dead_code)]
fn is_sugar_enabled() -> bool {
    crate::parser::sugar_gate::is_enabled()
}

impl NyashParser {
    pub(crate) fn expr_parse_ternary(&mut self) -> Result<ASTNode, ParseError> {
        let cond = self.expr_parse_coalesce()?;
        if self.match_token(&TokenType::QUESTION) {
            self.advance();
            let then_expr = self.parse_expression()?;
            self.consume(TokenType::COLON)?;
            let else_expr = self.parse_expression()?;
            return Ok(ASTNode::If {
                condition: Box::new(cond),
                then_body: vec![then_expr],
                else_body: Some(vec![else_expr]),
                span: Span::unknown(),
            });
        }
        Ok(cond)
    }
}
