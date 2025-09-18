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
pub fn try_parse_header_first_field_or_property(
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
