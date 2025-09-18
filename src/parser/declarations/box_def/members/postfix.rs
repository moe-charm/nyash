//! Postfix handlers (catch/cleanup) utilities for unified members
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

/// If Stage-3 gate allows, parse optional catch/cleanup after a block body and wrap it.
/// Returns a (possibly) wrapped body.
pub fn wrap_with_optional_postfix(
    p: &mut NyashParser,
    body: Vec<ASTNode>,
) -> Result<Vec<ASTNode>, ParseError> {
    if !(crate::config::env::parser_stage3()
        && (p.match_token(&TokenType::CATCH) || p.match_token(&TokenType::CLEANUP)))
    {
        return Ok(body);
    }

    let mut catch_clauses: Vec<crate::ast::CatchClause> = Vec::new();
    if p.match_token(&TokenType::CATCH) {
        p.advance();
        p.consume(TokenType::LPAREN)?;
        let (exc_ty, exc_var) = p.parse_catch_param()?;
        p.consume(TokenType::RPAREN)?;
        let catch_body = p.parse_block_statements()?;
        catch_clauses.push(crate::ast::CatchClause {
            exception_type: exc_ty,
            variable_name: exc_var,
            body: catch_body,
            span: Span::unknown(),
        });
        p.skip_newlines();
        if p.match_token(&TokenType::CATCH) {
            let line = p.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "single catch only after member body".to_string(),
                line,
            });
        }
    }
    let finally_body = if p.match_token(&TokenType::CLEANUP) {
        p.advance();
        Some(p.parse_block_statements()?)
    } else {
        None
    };
    Ok(vec![ASTNode::TryCatch {
        try_body: body,
        catch_clauses,
        finally_body,
        span: Span::unknown(),
    }])
}
