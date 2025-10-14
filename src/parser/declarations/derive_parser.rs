/*!
 * @derive macro (parser-level syntactic sugar)
 *
 * Syntax:
 *   @derive('Equals','ToString', ...)
 *   box Name { ... }
 *
 * Behavior (MVP):
 * - Injects missing methods onto the following BoxDeclaration only.
 * - Equals: equals(other) comparing public fields with '==' (structural equality for primitives).
 * - ToString: toString() returning "Name(f1, f2, ...)" based on public field order.
 * - Existing methods are never overwritten.
 * - Gated by NYASH_MACRO_ENABLE=1 (enforced by caller).
 */

use crate::ast::{ASTNode, LiteralValue, Span};
use crate::ast::BinaryOperator;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::HashMap;

impl NyashParser {
    pub(crate) fn parse_derive_then_box(&mut self) -> Result<ASTNode, ParseError> {
        // Expect '(' derives ')' then a box declaration
        // Current token is IDENT("derive") when called
        self.advance(); // consume 'derive'
        self.consume(TokenType::LPAREN)?;
        let mut derives: Vec<String> = Vec::new();
        while !self.match_token(&TokenType::RPAREN) {
            match &self.current_token().token_type {
                TokenType::STRING(s) => {
                    derives.push(s.clone());
                    self.advance();
                }
                other => {
                    return Err(ParseError::UnexpectedToken { found: other.clone(), expected: "string literal".to_string(), line: self.current_token().line });
                }
            }
            if self.match_token(&TokenType::COMMA) { self.advance(); }
        }
        self.consume(TokenType::RPAREN)?;

        // Next must be a box declaration
        if !self.match_token(&TokenType::BOX) {
            return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "'box' declaration after @derive".to_string(), line: self.current_token().line });
        }
        let mut node = self.parse_box_declaration()?;
        // Inject derives
        if let ASTNode::BoxDeclaration { name, fields: _, public_fields, private_fields: _, methods, .. } = &mut node {
            let want_eq = derives.iter().any(|d| d.eq_ignore_ascii_case("Equals"));
            let want_ts = derives.iter().any(|d| d.eq_ignore_ascii_case("ToString") || d.eq_ignore_ascii_case("Debug"));
            if want_eq && !methods.contains_key("equals") {
                let m = build_equals_method(public_fields);
                methods.insert("equals".to_string(), m);
            }
            if want_ts && !methods.contains_key("toString") {
                let m = build_tostring_method(name, public_fields);
                methods.insert("toString".to_string(), m);
            }
            Ok(node)
        } else {
            Err(ParseError::InvalidStatement { line: self.current_token().line })
        }
    }
}

fn me_field(name: &str) -> ASTNode {
    ASTNode::FieldAccess {
        object: Box::new(ASTNode::Me { span: Span::unknown() }),
        field: name.to_string(),
        span: Span::unknown(),
    }
}

fn var_field(var: &str, field: &str) -> ASTNode {
    ASTNode::FieldAccess {
        object: Box::new(ASTNode::Variable { name: var.to_string(), span: Span::unknown() }),
        field: field.to_string(),
        span: Span::unknown(),
    }
}

fn bin_and(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::And, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

fn bin_eq(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
    ASTNode::BinaryOp { operator: BinaryOperator::Equal, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
}

fn lit_str(s: &str) -> ASTNode { ASTNode::Literal { value: LiteralValue::String(s.to_string()), span: Span::unknown() } }

fn lit_bool(b: bool) -> ASTNode { ASTNode::Literal { value: LiteralValue::Bool(b), span: Span::unknown() } }

fn build_equals_method(public_fields: &Vec<String>) -> ASTNode {
    // equals(__ny_other) { return me.f1 == __ny_other.f1 && ...; }
    let cond = if public_fields.is_empty() { lit_bool(false) } else {
        let mut it = public_fields.iter();
        let first = it.next().unwrap();
        let mut expr = bin_eq(me_field(first), var_field("__ny_other", first));
        for f in it { expr = bin_and(expr, bin_eq(me_field(f), var_field("__ny_other", f))); }
        expr
    };
    ASTNode::FunctionDeclaration {
        name: "equals".to_string(),
        params: vec!["__ny_other".to_string()],
        body: vec![ASTNode::Return { value: Some(Box::new(cond)), span: Span::unknown() }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}

fn build_tostring_method(box_name: &str, public_fields: &Vec<String>) -> ASTNode {
    fn bin_add(lhs: ASTNode, rhs: ASTNode) -> ASTNode {
        ASTNode::BinaryOp { operator: BinaryOperator::Add, left: Box::new(lhs), right: Box::new(rhs), span: Span::unknown() }
    }
    let mut expr = lit_str(&format!("{}(", box_name));
    let mut first = true;
    for f in public_fields {
        if !first { expr = bin_add(expr, lit_str(",")); }
        first = false;
        expr = bin_add(expr, me_field(f));
    }
    expr = bin_add(expr, lit_str(")"));
    ASTNode::FunctionDeclaration {
        name: "toString".to_string(),
        params: vec![],
        body: vec![ASTNode::Return { value: Some(Box::new(expr)), span: Span::unknown() }],
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    }
}
