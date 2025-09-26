//! Validators and light analysis for box members
use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::{HashMap, HashSet};

/// Forbid user-defined methods named exactly as the box (constructor-like names).
/// Nyash constructors are explicit: init/pack/birth. A method with the same name
/// as the box is likely a mistaken constructor attempt; reject for clarity.
pub(crate) fn validate_no_ctor_like_name(
    p: &mut NyashParser,
    box_name: &str,
    methods: &HashMap<String, ASTNode>,
) -> Result<(), ParseError> {
    if methods.contains_key(box_name) {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: format!(
                "method name must not match box name '{}'; use init/pack/birth for constructors",
                box_name
            ),
            line,
        });
    }
    Ok(())
}

/// Validate that birth_once properties do not have cyclic dependencies via me.<prop> references
pub(crate) fn validate_birth_once_cycles(
    p: &mut NyashParser,
    methods: &HashMap<String, ASTNode>,
) -> Result<(), ParseError> {
    if !crate::config::env::unified_members() {
        return Ok(());
    }
    // Collect birth_once compute bodies
    let mut birth_bodies: HashMap<String, Vec<ASTNode>> = HashMap::new();
    for (mname, mast) in methods {
        if let Some(prop) = mname.strip_prefix("__compute_birth_") {
            if let ASTNode::FunctionDeclaration { body, .. } = mast {
                birth_bodies.insert(prop.to_string(), body.clone());
            }
        }
    }
    if birth_bodies.is_empty() {
        return Ok(());
    }
    // Build dependency graph: A -> {B | me.B used inside A}
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let props: HashSet<String> = birth_bodies.keys().cloned().collect();
    for (pname, body) in &birth_bodies {
        let used = ast_collect_me_fields(body);
        let mut set = HashSet::new();
        for u in used {
            if props.contains(&u) && u != *pname {
                set.insert(u);
            }
        }
        deps.insert(pname.clone(), set);
    }
    // Detect cycle via DFS
    fn has_cycle(
        node: &str,
        deps: &HashMap<String, HashSet<String>>,
        temp: &mut HashSet<String>,
        perm: &mut HashSet<String>,
    ) -> bool {
        if perm.contains(node) { return false; }
        if !temp.insert(node.to_string()) { return true; } // back-edge
        if let Some(ns) = deps.get(node) {
            for n in ns {
                if has_cycle(n, deps, temp, perm) { return true; }
            }
        }
        temp.remove(node);
        perm.insert(node.to_string());
        false
    }
    let mut perm = HashSet::new();
    let mut temp = HashSet::new();
    for pname in deps.keys() {
        if has_cycle(pname, &deps, &mut temp, &mut perm) {
            let line = p.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "birth_once declarations must not have cyclic dependencies".to_string(),
                line,
            });
        }
    }
    Ok(())
}

/// Forbid constructor call with the same name as the box; enforce `birth()` usage.
pub(crate) fn forbid_box_named_constructor(p: &mut NyashParser, box_name: &str) -> Result<(), ParseError> {
    if let TokenType::IDENTIFIER(id) = &p.current_token().token_type {
        if id == box_name && p.peek_token() == &TokenType::LPAREN {
            return Err(ParseError::UnexpectedToken {
                expected: format!(
                    "birth() constructor instead of {}(). Nyash uses birth() for unified constructor syntax.",
                    box_name
                ),
                found: TokenType::IDENTIFIER(box_name.to_string()),
                line: p.current_token().line,
            });
        }
    }
    Ok(())
}

/// Collect all `me.<field>` accessed in nodes (flat set)
fn ast_collect_me_fields(nodes: &Vec<ASTNode>) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    fn scan(nodes: &Vec<ASTNode>, out: &mut HashSet<String>) {
        for n in nodes {
            match n {
                ASTNode::FieldAccess { object, field, .. } => {
                    if let ASTNode::Me { .. } = object.as_ref() {
                        out.insert(field.clone());
                    }
                }
                ASTNode::Program { statements, .. } => scan(statements, out),
                ASTNode::If { then_body, else_body, .. } => {
                    scan(then_body, out);
                    if let Some(eb) = else_body { scan(eb, out); }
                }
                ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                    scan(try_body, out);
                    for c in catch_clauses { scan(&c.body, out); }
                    if let Some(fb) = finally_body { scan(fb, out); }
                }
                ASTNode::FunctionDeclaration { body, .. } => scan(body, out),
                _ => {}
            }
        }
    }
    let mut hs = HashSet::new();
    scan(nodes, &mut hs);
    hs
}
