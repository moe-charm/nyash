/*!
 * Flow Declaration Parser (staged)
 *
 * Syntax: flow Name { methods... }
 * Constraints:
 *  - No fields
 *  - No birth/fini
 *  - No `me` inside methods
 * Lowering:
 *  - Methods are treated as static and lowered to global functions: `Name.method/arity`
 */

use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::HashMap;

impl NyashParser {
    /// Parse `flow Name { ... }` declaration when NYASH_ENABLE_FLOW=1
    pub fn parse_flow_declaration(&mut self) -> Result<ASTNode, ParseError> {
        // current token is IDENTIFIER("flow")
        // Consume 'flow'
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(s) if s == "flow" => self.advance(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "'flow'".to_string(),
                    line: self.current_token().line,
                })
            }
        }

        // Next: Name
        let name = if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
            let nm = n.clone();
            self.advance();
            nm
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "flow name (identifier)".to_string(),
                line: self.current_token().line,
            });
        };

        // '{'
        self.consume(TokenType::LBRACE)?;

        let mut methods: HashMap<String, ASTNode> = HashMap::new();
        let mut fields: Vec<String> = Vec::new();
        let mut last_method_name: Option<String> = None;

        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            // Allow blank lines
            while self.match_token(&TokenType::NEWLINE) { self.advance(); }

            // Allow method postfix (catch/cleanup) to attach to previous method
            if crate::parser::declarations::box_def::members::postfix::try_parse_method_postfix_after_last_method(
                self, &mut methods, &last_method_name,
            )? { continue; }

            if self.match_token(&TokenType::RBRACE) { break; }

            // Expect identifier for method or (forbidden) field
            if let TokenType::IDENTIFIER(name_or_field) = &self.current_token().token_type {
                let ident = name_or_field.clone();
                self.advance();
                let fields_len_before = fields.len();
                crate::parser::declarations::static_def::members::try_parse_method_or_field(
                    self,
                    ident.clone(),
                    &mut methods,
                    &mut fields,
                    &mut last_method_name,
                )?;
                // Field detection
                if fields.len() > fields_len_before {
                    return Err(ParseError::UnexpectedToken {
                        found: TokenType::IDENTIFIER(ident),
                        expected: "flow cannot declare fields".to_string(),
                        line: self.current_token().line,
                    });
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "method declaration in flow".to_string(),
                    line: self.current_token().line,
                });
            }
        }

        // Consume '}'
        self.consume(TokenType::RBRACE)?;

        // Validate forbidden method names and `me` usage inside methods
        for (mn, node) in &methods {
            let mname = mn.as_str();
            if mname == "birth" || mname == "fini" {
                return Err(ParseError::UnexpectedToken {
                    found: TokenType::IDENTIFIER(mn.clone()),
                    expected: "flow forbids birth/fini".to_string(),
                    line: self.current_token().line,
                });
            }
            if let ASTNode::FunctionDeclaration { body, .. } = node {
                if contains_me(body) {
                    return Err(ParseError::UnexpectedToken {
                        found: TokenType::ME,
                        expected: "flow methods have no receiver ('me' not allowed)".to_string(),
                        line: self.current_token().line,
                    });
                }
            }
        }

        // Build BoxDeclaration with static=true (flow lowering piggybacks on static lowering)
        Ok(ASTNode::BoxDeclaration {
            name,
            fields: Vec::new(),
            public_fields: Vec::new(),
            private_fields: Vec::new(),
            methods,
            constructors: HashMap::new(),
            init_fields: Vec::new(),
            weak_fields: Vec::new(),
            is_interface: false,
            extends: Vec::new(),
            implements: Vec::new(),
            type_parameters: Vec::new(),
            is_static: true,
            static_init: None,
            span: Span::unknown(),
        })
    }
}

fn contains_me(nodes: &[ASTNode]) -> bool {
    for n in nodes.iter() {
        if ast_has_me(n) { return true; }
    }
    false
}

fn ast_has_me(node: &ASTNode) -> bool {
    use ASTNode::*;
    match node {
        Me { .. } => true,
        Program { statements, .. } => contains_me(statements),
        Local { initial_values, .. } => initial_values
            .iter()
            .flatten()
            .any(|e| ast_has_me(e)),
        Assignment { target, value, .. } => ast_has_me(target) || ast_has_me(value),
        BinaryOp { left, right, .. } => ast_has_me(left) || ast_has_me(right),
        UnaryOp { operand, .. } => ast_has_me(operand),
        MethodCall { object, arguments, .. } => ast_has_me(object) || arguments.iter().any(ast_has_me),
        FieldAccess { object, .. } => ast_has_me(object),
        FunctionCall { arguments, .. } => arguments.iter().any(ast_has_me),
        FromCall { arguments, .. } => arguments.iter().any(ast_has_me),
        ArrayLiteral { elements, .. } => elements.iter().any(ast_has_me),
        MapLiteral { entries, .. } => entries.iter().any(|(_k, v)| ast_has_me(v)),
        If { condition, then_body, else_body, .. } => {
            ast_has_me(condition) || contains_me(then_body) || else_body.as_ref().map(|b| contains_me(b)).unwrap_or(false)
        }
        Loop { condition, body, .. } => ast_has_me(condition) || contains_me(body),
        TryCatch { try_body, catch_clauses, finally_body, .. } => {
            if contains_me(try_body) { return true; }
            for c in catch_clauses.iter() {
                if contains_me(&c.body) { return true; }
            }
            if let Some(fin) = finally_body { if contains_me(fin) { return true; } }
            false
        }
        Return { value, .. } => value.as_ref().map(|v| ast_has_me(v)).unwrap_or(false),
        _ => false,
    }
}
