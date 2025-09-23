//! Constructors parsing (init/pack/birth)
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

/// Try to parse a constructor at current position.
/// Supported: `init(...) {}`, `pack(...) {}`, `birth(...) {}`.
/// Returns Ok(Some((key, node))) when a constructor was parsed and consumed.
pub(crate) fn try_parse_constructor(
    p: &mut NyashParser,
    is_override: bool,
) -> Result<Option<(String, ASTNode)>, ParseError> {
    // init(...)
    if p.match_token(&TokenType::INIT) && p.peek_token() == &TokenType::LPAREN {
        if is_override {
            return Err(ParseError::UnexpectedToken {
                expected: "method definition, not constructor after override keyword".to_string(),
                found: TokenType::INIT,
                line: p.current_token().line,
            });
        }
        let name = "init".to_string();
        p.advance(); // consume 'init'
        p.consume(TokenType::LPAREN)?;
        let mut params = Vec::new();
        while !p.match_token(&TokenType::RPAREN) && !p.is_at_end() {
            crate::must_advance!(p, _unused, "constructor parameter parsing");
            if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
                params.push(param.clone());
                p.advance();
            }
            if p.match_token(&TokenType::COMMA) {
                p.advance();
            }
        }
        p.consume(TokenType::RPAREN)?;
        let mut body = p.parse_block_statements()?;
        // Optional postfix catch/cleanup (method-level gate)
        if p.match_token(&TokenType::CATCH) || p.match_token(&TokenType::CLEANUP) {
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
                if p.match_token(&TokenType::CATCH) {
                    let line = p.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: p.current_token().token_type.clone(),
                        expected: "single catch only after method body".to_string(),
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
            body = vec![ASTNode::TryCatch { try_body: body, catch_clauses, finally_body, span: Span::unknown() }];
        }
        let node = ASTNode::FunctionDeclaration {
            name: name.clone(),
            params: params.clone(),
            body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        let key = format!("{}/{}", name, params.len());
        return Ok(Some((key, node)));
    }

    // pack(...)
    if p.match_token(&TokenType::PACK) && p.peek_token() == &TokenType::LPAREN {
        if is_override {
            return Err(ParseError::UnexpectedToken {
                expected: "method definition, not constructor after override keyword".to_string(),
                found: TokenType::PACK,
                line: p.current_token().line,
            });
        }
        let name = "pack".to_string();
        p.advance(); // consume 'pack'
        p.consume(TokenType::LPAREN)?;
        let mut params = Vec::new();
        while !p.match_token(&TokenType::RPAREN) && !p.is_at_end() {
            crate::must_advance!(p, _unused, "pack parameter parsing");
            if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
                params.push(param.clone());
                p.advance();
            }
            if p.match_token(&TokenType::COMMA) {
                p.advance();
            }
        }
        p.consume(TokenType::RPAREN)?;
        let body = p.parse_block_statements()?;
        let node = ASTNode::FunctionDeclaration {
            name: name.clone(),
            params: params.clone(),
            body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        let key = format!("{}/{}", name, params.len());
        return Ok(Some((key, node)));
    }

    // birth(...)
    if p.match_token(&TokenType::BIRTH) && p.peek_token() == &TokenType::LPAREN {
        if is_override {
            return Err(ParseError::UnexpectedToken {
                expected: "method definition, not constructor after override keyword".to_string(),
                found: TokenType::BIRTH,
                line: p.current_token().line,
            });
        }
        let name = "birth".to_string();
        p.advance(); // consume 'birth'
        p.consume(TokenType::LPAREN)?;
        let mut params = Vec::new();
        while !p.match_token(&TokenType::RPAREN) && !p.is_at_end() {
            crate::must_advance!(p, _unused, "birth parameter parsing");
            if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
                params.push(param.clone());
                p.advance();
            }
            if p.match_token(&TokenType::COMMA) {
                p.advance();
            }
        }
        p.consume(TokenType::RPAREN)?;
        let body = p.parse_block_statements()?;
        let node = ASTNode::FunctionDeclaration {
            name: name.clone(),
            params: params.clone(),
            body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        let key = format!("{}/{}", name, params.len());
        return Ok(Some((key, node)));
    }

    Ok(None)
}
