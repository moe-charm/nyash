/*!
 * @repeat macro (parser-level syntactic sugar)
 *
 * Syntax:
 *   @repeat(n) { body }
 *
 * Lowering:
 *   {
 *     local __ny_n = n;
 *     local __ny_i = 0;
 *     loop(__ny_i < __ny_n) {
 *       body
 *       __ny_i = __ny_i + 1;
 *     }
 *   }
 */

use crate::ast::{ASTNode, LiteralValue, Span};
use crate::ast::BinaryOperator;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    pub(super) fn parse_repeat_macro(&mut self) -> Result<ASTNode, ParseError> {
        // Current token is '@'
        self.advance(); // consume '@'
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) if s == "repeat" => { self.advance(); }
            other => return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "'repeat'".into(), line: self.current_token().line }),
        }
        self.consume(TokenType::LPAREN)?;
        let n_expr = self.parse_expression()?;
        self.consume(TokenType::RPAREN)?;
        let body_user = self.parse_block_statements()?;

        // local __ny_n = n_expr;
        let local_n = ASTNode::Local {
            variables: vec!["__ny_n".into()],
            initial_values: vec![Some(Box::new(n_expr))],
            span: Span::unknown(),
        };
        // local __ny_i = 0;
        let local_i = ASTNode::Local {
            variables: vec!["__ny_i".into()],
            initial_values: vec![Some(Box::new(ASTNode::Literal { value: LiteralValue::Integer(0), span: Span::unknown() }))],
            span: Span::unknown(),
        };
        // condition: __ny_i < __ny_n
        let cond = ASTNode::BinaryOp {
            operator: BinaryOperator::Less,
            left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }),
            right: Box::new(ASTNode::Variable { name: "__ny_n".into(), span: Span::unknown() }),
            span: Span::unknown(),
        };
        // increment: __ny_i = __ny_i + 1
        let inc = ASTNode::Assignment {
            target: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }),
            value: Box::new(ASTNode::BinaryOp {
                operator: BinaryOperator::Add,
                left: Box::new(ASTNode::Variable { name: "__ny_i".into(), span: Span::unknown() }),
                right: Box::new(ASTNode::Literal { value: LiteralValue::Integer(1), span: Span::unknown() }),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };

        let mut loop_body = Vec::<ASTNode>::new();
        loop_body.extend(body_user);
        loop_body.push(inc);
        let loop_stmt = ASTNode::Loop { condition: Box::new(cond), body: loop_body, span: Span::unknown() };

        Ok(ASTNode::ScopeBox { body: vec![local_n, local_i, loop_stmt], span: Span::unknown() })
    }
}

