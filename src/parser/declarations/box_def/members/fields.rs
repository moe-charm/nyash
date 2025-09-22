//! Fields parsing (header-first: `name: Type` + unified members gates)
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::HashMap;

/// Parse a header-first field or property that starts with an already parsed identifier `fname`.
/// Handles:
/// - `name: Type`                      → field
/// - `name: Type = expr`               → field with initializer (initializer is parsed then discarded at P0)
/// - `name: Type => expr`              → computed property (getter function generated)
/// - `name: Type { ... } [catch|cleanup]` → computed property block with optional postfix handlers
/// Returns Ok(true) when this function consumed and handled the construct; Ok(false) if not applicable.
pub(crate) fn try_parse_header_first_field_or_property(
    p: &mut NyashParser,
    fname: String,
    methods: &mut HashMap<String, ASTNode>,
    fields: &mut Vec<String>,
) -> Result<bool, ParseError> {
    // Expect ':' Type after name
    if !p.match_token(&TokenType::COLON) {
        // No type annotation: treat as bare stored field
        fields.push(fname);
        return Ok(true);
    }
    p.advance(); // consume ':'
    // Optional type name (identifier). For now we accept and ignore.
    if let TokenType::IDENTIFIER(_ty) = &p.current_token().token_type {
        p.advance();
    } else {
        // If no type present, still proceed (tolerant parsing), but only when unified_members gate is off
        // Keep behavior aligned with existing parser (it allowed missing type in some branches)
    }

    // Unified members gate behavior
    if crate::config::env::unified_members() {
        // name: Type = expr  → field with initializer (store as field, initializer discarded at P0)
        if p.match_token(&TokenType::ASSIGN) {
            p.advance();
            let _init_expr = p.parse_expression()?; // P0: parse and discard
            fields.push(fname);
            p.skip_newlines();
            return Ok(true);
        }
        // name: Type => expr  → computed property (getter method with return expr)
        if p.match_token(&TokenType::FatArrow) {
            p.advance();
            let expr = p.parse_expression()?;
            let body = vec![ASTNode::Return {
                value: Some(Box::new(expr)),
                span: Span::unknown(),
            }];
            let getter_name = format!("__get_{}", fname);
            let method = ASTNode::FunctionDeclaration {
                name: getter_name.clone(),
                params: vec![],
                body,
                is_static: false,
                is_override: false,
                span: Span::unknown(),
            };
            methods.insert(getter_name, method);
            p.skip_newlines();
            return Ok(true);
        }
        // name: Type { ... } [postfix]
        if p.match_token(&TokenType::LBRACE) {
            let body = p.parse_block_statements()?;
            let body = crate::parser::declarations::box_def::members::postfix::wrap_with_optional_postfix(p, body)?;
            let getter_name = format!("__get_{}", fname);
            let method = ASTNode::FunctionDeclaration {
                name: getter_name.clone(),
                params: vec![],
                body,
                is_static: false,
                is_override: false,
                span: Span::unknown(),
            };
            methods.insert(getter_name, method);
            p.skip_newlines();
            return Ok(true);
        }
    }

    // Default: treat as a plain field when unified-members gate didn't match any special form
    fields.push(fname);
    Ok(true)
}

/// Parse a visibility block or a single header-first property with visibility prefix.
/// Handles:
/// - `public { a, b, c }`  → pushes to fields/public_fields
/// - `private { ... }`      → pushes to fields/private_fields
/// - `public name: Type ...` (delegates to header-first field/property)
/// Returns Ok(true) if consumed, Ok(false) if visibility keyword not matched.
pub(crate) fn try_parse_visibility_block_or_single(
    p: &mut NyashParser,
    visibility: &str,
    methods: &mut HashMap<String, ASTNode>,
    fields: &mut Vec<String>,
    public_fields: &mut Vec<String>,
    private_fields: &mut Vec<String>,
    last_method_name: &mut Option<String>,
) -> Result<bool, ParseError> {
    if visibility != "public" && visibility != "private" {
        return Ok(false);
    }
    if p.match_token(&TokenType::LBRACE) {
        p.advance();
        p.skip_newlines();
        while !p.match_token(&TokenType::RBRACE) && !p.is_at_end() {
            if let TokenType::IDENTIFIER(fname) = &p.current_token().token_type {
                let fname = fname.clone();
                if visibility == "public" { public_fields.push(fname.clone()); } else { private_fields.push(fname.clone()); }
                fields.push(fname);
                p.advance();
                if p.match_token(&TokenType::COMMA) { p.advance(); }
                p.skip_newlines();
                continue;
            }
            return Err(ParseError::UnexpectedToken {
                expected: "identifier in visibility block".to_string(),
                found: p.current_token().token_type.clone(),
                line: p.current_token().line,
            });
        }
        p.consume(TokenType::RBRACE)?;
        p.skip_newlines();
        return Ok(true);
    }
    if let TokenType::IDENTIFIER(n) = &p.current_token().token_type {
        let fname = n.clone();
        p.advance();
        if try_parse_header_first_field_or_property(p, fname.clone(), methods, fields)? {
            if visibility == "public" { public_fields.push(fname.clone()); } else { private_fields.push(fname.clone()); }
            *last_method_name = None;
            p.skip_newlines();
            return Ok(true);
        } else {
            if visibility == "public" { public_fields.push(fname.clone()); } else { private_fields.push(fname.clone()); }
            fields.push(fname);
            p.skip_newlines();
            return Ok(true);
        }
    }
    Ok(false)
}

/// Parse `init { ... }` non-call block to collect initializable fields and weak flags.
/// Returns Ok(true) if consumed; Ok(false) if no `init {` at current position.
pub(crate) fn parse_init_block_if_any(
    p: &mut NyashParser,
    init_fields: &mut Vec<String>,
    weak_fields: &mut Vec<String>,
) -> Result<bool, ParseError> {
    if !(p.match_token(&TokenType::INIT) && p.peek_token() != &TokenType::LPAREN) {
        return Ok(false);
    }
    p.advance(); // consume 'init'
    p.consume(TokenType::LBRACE)?;
    while !p.match_token(&TokenType::RBRACE) && !p.is_at_end() {
        p.skip_newlines();
        if p.match_token(&TokenType::RBRACE) {
            break;
        }
        let is_weak = if p.match_token(&TokenType::WEAK) {
            p.advance();
            true
        } else {
            false
        };
        if let TokenType::IDENTIFIER(field_name) = &p.current_token().token_type {
            init_fields.push(field_name.clone());
            if is_weak {
                weak_fields.push(field_name.clone());
            }
            p.advance();
            if p.match_token(&TokenType::COMMA) { p.advance(); }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: if is_weak { "field name after 'weak'" } else { "field name" }.to_string(),
                found: p.current_token().token_type.clone(),
                line: p.current_token().line,
            });
        }
    }
    p.consume(TokenType::RBRACE)?;
    Ok(true)
}
