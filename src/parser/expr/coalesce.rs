use crate::ast::{ASTNode, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

#[inline]
fn is_sugar_enabled() -> bool {
    crate::parser::sugar_gate::is_enabled()
}

impl NyashParser {
    pub(crate) fn expr_parse_coalesce(&mut self) -> Result<ASTNode, ParseError> {
        let mut expr = self.expr_parse_or()?;
        while self.match_token(&TokenType::QmarkQmark) {
            if !is_sugar_enabled() {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "enable NYASH_SYNTAX_SUGAR_LEVEL=basic|full for '??'".to_string(),
                    line,
                });
            }
            self.advance();
            let rhs = self.expr_parse_or()?;
            let scr = expr;
            expr = ASTNode::MatchExpr {
                scrutinee: Box::new(scr.clone()),
                arms: vec![(crate::ast::LiteralValue::Null, rhs)],
                else_expr: Box::new(scr),
                span: Span::unknown(),
            };
        }
        Ok(expr)
    }
}
