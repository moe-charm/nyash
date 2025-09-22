//! Header parsing: `Name<T...> from Parent1, Parent2`
//! Assumes the caller already consumed the leading `box` token.
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

/// Parse the leading header of a box declaration and return
/// (name, type_params, extends, implements). Does not consume the opening '{'.
pub(crate) fn parse_header(
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
        while !p.match_token(&TokenType::GREATER) && !p.is_at_end() {
            crate::must_advance!(p, _unused, "generic type parameter parsing");
            if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
                params.push(param.clone());
                p.advance();
                if p.match_token(&TokenType::COMMA) {
                    p.advance();
                    p.skip_newlines();
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "type parameter name".to_string(),
                    line: p.current_token().line,
                });
            }
        }
        p.consume(TokenType::GREATER)?; // consume '>'
        params
    } else {
        Vec::new()
    };

    // extends: from Parent1, Parent2
    let extends = if p.match_token(&TokenType::FROM) {
        p.advance(); // consume 'from'
        let mut parents = Vec::new();
        if let TokenType::IDENTIFIER(parent) = &p.current_token().token_type {
            parents.push(parent.clone());
            p.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "parent box name after 'from'".to_string(),
                line: p.current_token().line,
            });
        }
        while p.match_token(&TokenType::COMMA) {
            p.advance(); // consume ','
            p.skip_newlines();
            if let TokenType::IDENTIFIER(parent) = &p.current_token().token_type {
                parents.push(parent.clone());
                p.advance();
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "parent box name after comma".to_string(),
                    line: p.current_token().line,
                });
            }
        }
        parents
    } else {
        Vec::new()
    };

    // implements: not supported (TokenType::IMPLEMENTS not defined yet)
    let implements: Vec<String> = Vec::new();

    Ok((name, type_parameters, extends, implements))
}
