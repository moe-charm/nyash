use nyash_rust::ast::ASTNode;
use nyash_rust::ast::Span;

use std::collections::HashSet;

/// Desugar Alias access: Alias.X(.Y ... ) => X(.Y ...), or to `Alias_X` when `prefix` is provided.
/// - If `prefix` is None: drop the alias for simple flatten-merge use cases.
/// - If `prefix` is Some(alias): transform `Alias.X` into `Alias_X` to avoid collisions.
pub fn desugar_alias_field_access(ast: &ASTNode, aliases: &HashSet<String>, to_prefixed: bool) -> ASTNode {
    fn rewrite(node: &ASTNode, aliases: &HashSet<String>, to_prefixed: bool) -> ASTNode {
        match node {
            ASTNode::FieldAccess { object, field, .. } => {
                let o2 = rewrite(object, aliases, to_prefixed);
                if let ASTNode::Variable { name, .. } = &o2 {
                    if aliases.contains(name) {
                        if to_prefixed {
                            return ASTNode::Variable { name: format!("{}_{}", name, field), span: Span::unknown() };
                        } else {
                            return ASTNode::Variable { name: field.clone(), span: Span::unknown() };
                        }
                    }
                }
                ASTNode::FieldAccess { object: Box::new(o2), field: field.clone(), span: Span::unknown() }
            }
            ASTNode::MethodCall { object, method, arguments, .. } => {
                let o2 = rewrite(object, aliases, to_prefixed);
                let args2 = arguments.iter().map(|a| rewrite(a, aliases, to_prefixed)).collect();
                ASTNode::MethodCall { object: Box::new(o2), method: method.clone(), arguments: args2, span: Span::unknown() }
            }
            ASTNode::FunctionCall { name, arguments, .. } => {
                let args2 = arguments.iter().map(|a| rewrite(a, aliases, to_prefixed)).collect();
                ASTNode::FunctionCall { name: name.clone(), arguments: args2, span: Span::unknown() }
            }
            ASTNode::ArrayLiteral { elements, .. } => {
                ASTNode::ArrayLiteral { elements: elements.iter().map(|e| rewrite(e, aliases, to_prefixed)).collect(), span: Span::unknown() }
            }
            ASTNode::MapLiteral { entries, .. } => {
                ASTNode::MapLiteral { entries: entries.iter().map(|(k, v)| (k.clone(), rewrite(v, aliases, to_prefixed))).collect(), span: Span::unknown() }
            }
            ASTNode::Assignment { target, value, .. } => {
                ASTNode::Assignment { target: Box::new(rewrite(target, aliases, to_prefixed)), value: Box::new(rewrite(value, aliases, to_prefixed)), span: Span::unknown() }
            }
            ASTNode::Return { value, .. } => {
                ASTNode::Return { value: value.as_ref().map(|v| Box::new(rewrite(v, aliases, to_prefixed))), span: Span::unknown() }
            }
            ASTNode::If { condition, then_body, else_body, .. } => {
                let cond2 = Box::new(rewrite(condition, aliases, to_prefixed));
                let then2 = then_body.iter().map(|n| rewrite(n, aliases, to_prefixed)).collect();
                let else2 = else_body.as_ref().map(|b| b.iter().map(|n| rewrite(n, aliases, to_prefixed)).collect());
                ASTNode::If { condition: cond2, then_body: then2, else_body: else2, span: Span::unknown() }
            }
            ASTNode::Loop { condition, body, .. } => {
                let c2 = Box::new(rewrite(condition, aliases, to_prefixed));
                let b2 = body.iter().map(|n| rewrite(n, aliases, to_prefixed)).collect();
                ASTNode::Loop { condition: c2, body: b2, span: Span::unknown() }
            }
            ASTNode::Program { statements, .. } => {
                ASTNode::Program { statements: statements.iter().map(|n| rewrite(n, aliases, to_prefixed)).collect(), span: Span::unknown() }
            }
            // Default: clone for other nodes
            x => x.clone(),
        }
    }
    rewrite(ast, aliases, to_prefixed)
}

/// Rename top-level symbols for a prelude AST by alias (`Alias_Foo`).
/// Only renames BoxDeclaration (static) and FunctionDeclaration to avoid conflicts with the main file.
pub fn rename_prelude_top_symbols(ast: &ASTNode, alias: &str) -> ASTNode {
    match ast {
        ASTNode::Program { statements, .. } => {
            let mut out: Vec<ASTNode> = Vec::with_capacity(statements.len());
            for st in statements.iter() {
                let n = match st {
                    ASTNode::BoxDeclaration { name, is_static, .. } if *is_static => {
                        let mut c = st.clone();
                        if let ASTNode::BoxDeclaration { name: ref mut nm, .. } = &mut c {
                            *nm = format!("{}_{}", alias, name);
                        }
                        c
                    }
                    ASTNode::FunctionDeclaration { name, .. } => {
                        let mut c = st.clone();
                        if let ASTNode::FunctionDeclaration { name: ref mut nm, .. } = &mut c {
                            *nm = format!("{}_{}", alias, name);
                        }
                        c
                    }
                    _ => st.clone(),
                };
                out.push(n);
            }
            ASTNode::Program { statements: out, span: Span::unknown() }
        }
        _ => ast.clone(),
    }
}

