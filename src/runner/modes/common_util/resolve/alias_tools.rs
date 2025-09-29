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
                let args2: Vec<ASTNode> = arguments.iter().map(|a| rewrite(a, aliases, to_prefixed)).collect();
                if to_prefixed {
                    if let ASTNode::Variable { name, .. } = &o2 {
                        // Case A: Alias.method(args) → Alias_Alias.method/arity(args)
                        if aliases.contains(name) {
                            // Alias.method(args) → Alias_Alias.method/arity(args)
                            // Works for typical pattern where prelude defines `static box Alias { ... }`.
                            let arity = args2.len();
                            let fname = format!("{}_{}.{}{}", name, name, method, format!("/{}", arity));
                            return ASTNode::FunctionCall { name: fname, arguments: args2, span: Span::unknown() };
                        }
                        // Case B: Alias-prefixed static box: P_My.greet() → FunctionCall "P_My.greet/0"
                        // Detect by checking if the variable starts with any alias + '_'
                        for a in aliases {
                            let pref = format!("{}_", a);
                            if name.starts_with(&pref) {
                                let arity = args2.len();
                                let fname = format!("{}.{}{}", name, method, format!("/{}", arity));
                                return ASTNode::FunctionCall { name: fname, arguments: args2, span: Span::unknown() };
                            }
                        }
                    }
                }
                ASTNode::MethodCall { object: Box::new(o2), method: method.clone(), arguments: args2, span: Span::unknown() }
            }
            ASTNode::FunctionCall { name, arguments, .. } => {
                let args2: Vec<ASTNode> = arguments.iter().map(|a| rewrite(a, aliases, to_prefixed)).collect();
                if to_prefixed {
                    // Rewrite qualified calls like Alias.Box.method(...) to CompilerMod_Box.method/arity
                    // to match lowered static method function names.
                    // Only transform when the call name starts with a known alias + '.'
                    for a in aliases {
                        let prefix = format!("{}.", a);
                        if let Some(rest) = name.strip_prefix(&prefix) {
                            if let Some(dot_pos) = rest.find('.') {
                                let head = &rest[..dot_pos]; // Box or top symbol
                                let tail = &rest[dot_pos + 1..]; // method or deeper
                                let arity = args2.len();
                                let new_name = format!("{}_{}.{}{}", a, head, tail, format!("/{}", arity));
                                return ASTNode::FunctionCall { name: new_name, arguments: args2, span: Span::unknown() };
                            } else {
                                // Alias.TopSymbol(...) → Alias_TopSymbol/arity
                                let arity = args2.len();
                                let new_name = format!("{}_{}{}", a, rest, format!("/{}", arity));
                                return ASTNode::FunctionCall { name: new_name, arguments: args2, span: Span::unknown() };
                            }
                        }
                    }
                }
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

/// Collect top-level symbol names from a prelude AST (static boxes and functions only).
pub fn collect_prelude_top_names(ast: &ASTNode) -> Vec<String> {
    let mut out = Vec::new();
    if let ASTNode::Program { statements, .. } = ast {
        for st in statements.iter() {
            match st {
                ASTNode::BoxDeclaration { name, is_static, .. } if *is_static => out.push(name.clone()),
                ASTNode::FunctionDeclaration { name, .. } => out.push(name.clone()),
                _ => {}
            }
        }
    }
    out
}

/// Rename with collision guard: fail if any `Alias_<Top>` collides with previously used names.
pub fn rename_with_collision_guard(
    ast: &ASTNode,
    alias: &str,
    used_prefixed: &mut std::collections::HashSet<String>,
    src_hint: &str,
) -> Result<ASTNode, String> {
    let tops = collect_prelude_top_names(ast);
    for t in tops.iter() {
        let pfx = format!("{}_{}", alias, t);
        if !used_prefixed.insert(pfx.clone()) {
            return Err(format!(
                "alias collision: '{}' already defined (source: {})",
                pfx, src_hint
            ));
        }
    }
    Ok(rename_prelude_top_symbols(ast, alias))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn hs(list: &[&str]) -> HashSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn field_access_alias_prefixed() {
        // Alias.Name -> Alias_Name (to_prefixed=true)
        let ast = ASTNode::FieldAccess {
            object: Box::new(ASTNode::Variable { name: "CompilerMod".into(), span: Span::unknown() }),
            field: "Main".into(),
            span: Span::unknown(),
        };
        let out = desugar_alias_field_access(&ast, &hs(&["CompilerMod"]), true);
        match out {
            ASTNode::Variable { name, .. } => assert_eq!(name, "CompilerMod_Main"),
            _ => panic!("unexpected AST after desugar"),
        }
    }

    #[test]
    fn function_call_alias_static_box_method() {
        // Alias.Box.method(a,b) -> Alias_Box.method/2(a,b)
        let ast = ASTNode::FunctionCall {
            name: "CompilerMod.MirEmitterBox.emit".into(),
            arguments: vec![
                ASTNode::Variable { name: "x".into(), span: Span::unknown() },
                ASTNode::Variable { name: "y".into(), span: Span::unknown() },
            ],
            span: Span::unknown(),
        };
        let out = desugar_alias_field_access(&ast, &hs(&["CompilerMod"]), true);
        match out {
            ASTNode::FunctionCall { name, arguments, .. } => {
                assert_eq!(name, "CompilerMod_MirEmitterBox.emit/2");
                assert_eq!(arguments.len(), 2);
            }
            _ => panic!("unexpected AST after desugar"),
        }
    }

    #[test]
    fn method_call_alias_receiver() {
        // Alias.method() -> Alias_Alias.method/0()
        let ast = ASTNode::MethodCall {
            object: Box::new(ASTNode::Variable { name: "CompilerMod".into(), span: Span::unknown() }),
            method: "main".into(),
            arguments: vec![],
            span: Span::unknown(),
        };
        let out = desugar_alias_field_access(&ast, &hs(&["CompilerMod"]), true);
        match out {
            ASTNode::FunctionCall { name, arguments, .. } => {
                assert_eq!(name, "CompilerMod_CompilerMod.main/0");
                assert!(arguments.is_empty());
            }
            _ => panic!("unexpected AST after desugar"),
        }
    }
}
