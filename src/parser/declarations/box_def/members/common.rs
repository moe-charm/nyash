//! Shared helpers for members parsing (scaffold)
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MemberKind {
    Field,
    Method,
    Constructor,
    PropertyComputed,
    PropertyOnce,
    PropertyBirthOnce,
}

/// Decide member kind via simple lookahead (scaffold placeholder)
pub(crate) fn classify_member(p: &mut NyashParser) -> Result<MemberKind, ParseError> {
    // block-first: { body } as (once|birth_once)? name : Type
    if crate::config::env::unified_members() && p.match_any_token(&[TokenType::LBRACE]) {
        return Ok(MemberKind::PropertyComputed);
    }

    // Constructors by keyword or name
    match &p.current_token().token_type {
        TokenType::PACK | TokenType::BIRTH => {
            if p.peek_token() == &TokenType::LPAREN { return Ok(MemberKind::Constructor); }
        }
        TokenType::IDENTIFIER(name) if (name == "init" || name == "birth" || name == "pack")
            && p.peek_token() == &TokenType::LPAREN => {
            return Ok(MemberKind::Constructor);
        }
        _ => {}
    }

    // Method: ident '(' ...
    if matches!(&p.current_token().token_type, TokenType::IDENTIFIER(_))
        && p.peek_token() == &TokenType::LPAREN
    {
        return Ok(MemberKind::Method);
    }

    // Field: [weak] ident ':' Type
    if p.match_any_token(&[TokenType::WEAK]) {
        // weak IDENT ':'
        // do not consume; use peek via offset: current is WEAK, next should be IDENT, then ':'
        // We only classify; the main parser will handle errors.
        return Ok(MemberKind::Field);
    }
    if matches!(&p.current_token().token_type, TokenType::IDENTIFIER(_))
        && p.peek_token() == &TokenType::COLON
    {
        return Ok(MemberKind::Field);
    }

    // Default: treat as method for graceful recovery
    Ok(MemberKind::Method)
}
