/*!
 * @assert macro — parser-level assert sugar
 * Syntax:
 *   @assert(cond)
 *   @assert(cond, msg)
 * Lowering:
 *   if (!cond) { throw msg_or_default }
 */

use crate::ast::{ASTNode, LiteralValue, Span, UnaryOperator};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(super) fn parse_assert_macro(&mut self) -> Result<ASTNode, ParseError> {
        // Current token is '@'
        self.advance(); // consume '@'
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) if s == "assert" => { self.advance(); }
            other => return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "'assert'".into(), line: self.current_token().line }),
        }
        self.consume(TokenType::LPAREN)?;
        let cond = self.parse_expression()?;
        let msg = if self.match_token(&TokenType::COMMA) {
            self.advance();
            self.parse_expression()?
        } else {
            ASTNode::Literal { value: LiteralValue::String("assertion failed".into()), span: Span::unknown() }
        };
        self.consume(TokenType::RPAREN)?;
        // Lower: if (!cond) { throw msg }
        let not_cond = ASTNode::UnaryOp { operator: UnaryOperator::Not, operand: Box::new(cond), span: Span::unknown() };
        let then_body = vec![ASTNode::Throw { expression: Box::new(msg), span: Span::unknown() }];
        Ok(ASTNode::If { condition: Box::new(not_cond), then_body, else_body: None, span: Span::unknown() })
    }
}

