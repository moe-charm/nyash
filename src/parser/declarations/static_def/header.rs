//! Header parsing for `static box Name<T...> from Parent1, ... [interface ...]` (staged)
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

/// Parse the leading header of a static box and return
/// (name, type_params, extends, implements). Does not consume the opening '{'.
pub(crate) fn parse_static_header(
    p: &mut NyashParser,
) -> Result<(String, Vec<String>, Vec<String>, Vec<String>), ParseError> {
    // Name
    let name = if let TokenType::IDENTIFIER(name) = &p.current_token().token_type {
        let name = name.clone();
        p.advance();
        name
    } else {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: "identifier".to_string(),
            line,
        });
    };

    // Generic type parameters: <T, U>
    let type_parameters = if p.match_token(&TokenType::LESS) {
        p.advance(); // consume '<'
        let mut params = Vec::new();
        loop {
            if let TokenType::IDENTIFIER(param_name) = &p.current_token().token_type {
                params.push(param_name.clone());
                p.advance();
                if p.match_token(&TokenType::COMMA) { p.advance(); } else { break; }
            } else {
                let line = p.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "type parameter name".to_string(),
                    line,
                });
            }
        }
        p.consume(TokenType::GREATER)?;
        params
    } else { Vec::new() };

    // extends: from Parent1, Parent2
    let extends = if p.match_token(&TokenType::FROM) {
        p.advance(); // consume 'from'
        let mut parents = Vec::new();
        loop {
            if let TokenType::IDENTIFIER(parent_name) = &p.current_token().token_type {
                parents.push(parent_name.clone());
                p.advance();
                if p.match_token(&TokenType::COMMA) { p.advance(); } else { break; }
            } else {
                let line = p.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "parent class name".to_string(),
                    line,
                });
            }
        }
        parents
    } else { Vec::new() };

    // implements: `interface A, B` (optional)
    let implements = if p.match_token(&TokenType::INTERFACE) {
        p.advance(); // consume 'interface'
        let mut list = Vec::new();
        loop {
            if let TokenType::IDENTIFIER(interface_name) = &p.current_token().token_type {
                list.push(interface_name.clone());
                p.advance();
                if p.match_token(&TokenType::COMMA) { p.advance(); } else { break; }
            } else {
                let line = p.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "interface name".to_string(),
                    line,
                });
            }
        }
        list
    } else { Vec::new() };

    Ok((name, type_parameters, extends, implements))
}

