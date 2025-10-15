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
                                // Alias.method(...) → Alias_Alias.method/arity （static box同名規約）
                                let arity = args2.len();
                                let new_name = format!("{}_{}.{}{}", a, a, rest, format!("/{}", arity));
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

/// Rewrite internal references inside a prelude after top-level rename.
/// Maps any reference to original top symbols to their alias-prefixed names.
///
/// Rules (MVP):
/// - Variable: `Top`                → `Alias_Top`
/// - FieldAccess: `Top.x`           → `Alias_Top.x`（object 部分が Variable の場合）
/// - FunctionCall: `Top.fn`         → `Alias_Top.fn`
///                 `Top`            → `Alias_Top`
///                 `Alias_Top.fn/N` → そのまま
/// - MethodCall は object 変換（Top→Alias_Top）のみに留める（関数化は既存経路に委ねる）。
fn rewrite_internal_refs_after_top_rename(
    ast: &ASTNode,
    alias: &str,
    original_tops: &[String],
) -> ASTNode {
    use nyash_rust::ast::ASTNode as N;
    use std::collections::HashMap;

    let mut map: HashMap<String, String> = HashMap::new();
    for t in original_tops.iter() {
        map.insert(t.clone(), format!("{}_{}", alias, t));
    }

    fn rewrite_name(name: &str, map: &HashMap<String, String>) -> String {
        // Exact match
        if let Some(n) = map.get(name) {
            return n.clone();
        }
        // Qualified form: Top.suffix → Alias_Top.suffix
        if let Some(pos) = name.find('.') {
            let head = &name[..pos];
            if let Some(pref) = map.get(head) {
                return format!("{}.{}", pref, &name[pos + 1..]);
            }
        }
        name.to_string()
    }

    fn visit(n: &N, map: &HashMap<String, String>) -> N {
        match n {
            N::Variable { name, .. } => N::Variable { name: rewrite_name(name, map), span: Span::unknown() },
            N::FieldAccess { object, field, .. } => {
                let o2 = visit(object, map);
                N::FieldAccess { object: Box::new(o2), field: field.clone(), span: Span::unknown() }
            }
            N::MethodCall { object, method, arguments, .. } => {
                let o2 = visit(object, map);
                let a2: Vec<N> = arguments.iter().map(|a| visit(a, map)).collect();
                N::MethodCall { object: Box::new(o2), method: method.clone(), arguments: a2, span: Span::unknown() }
            }
            N::FunctionCall { name, arguments, .. } => {
                let new_name = rewrite_name(name, map);
                let a2: Vec<N> = arguments.iter().map(|a| visit(a, map)).collect();
                N::FunctionCall { name: new_name, arguments: a2, span: Span::unknown() }
            }
            N::Assignment { target, value, .. } => {
                let t2 = visit(target, map);
                let v2 = visit(value, map);
                N::Assignment { target: Box::new(t2), value: Box::new(v2), span: Span::unknown() }
            }
            N::ArrayLiteral { elements, .. } => {
                N::ArrayLiteral { elements: elements.iter().map(|e| visit(e, map)).collect(), span: Span::unknown() }
            }
            N::MapLiteral { entries, .. } => {
                // Keys are Strings; rewrite only values
                N::MapLiteral { entries: entries.iter().map(|(k, v)| (k.clone(), visit(v, map))).collect(), span: Span::unknown() }
            }
            N::Program { statements, .. } => {
                N::Program { statements: statements.iter().map(|s| visit(s, map)).collect(), span: Span::unknown() }
            }
            _ => n.clone(),
        }
    }

    visit(ast, &map)
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
    let renamed = rename_prelude_top_symbols(ast, alias);
    // Optional gate: allow disabling internal reference rewrite via env (default ON)
    let do_internal = std::env::var("NYASH_ALIAS_INTERNAL_REWRITE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    if do_internal {
        let rewritten = rewrite_internal_refs_after_top_rename(&renamed, alias, &tops);
        Ok(rewritten)
    } else {
        Ok(renamed)
    }
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

    #[test]
    #[ignore = "AST fixture still uses legacy BoxDeclaration { body }; restore after Phase-31 doc update"]
    fn internal_ref_variable_is_rewritten() {
        // Program: static box X {}; local y = X;  → after alias A: y = A_X
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::BoxDeclaration { name: "X".into(), is_static: true, body: vec![], span: Span::unknown() },
                ASTNode::Assignment {
                    target: Box::new(ASTNode::Variable { name: "y".into(), span: Span::unknown() }),
                    value: Box::new(ASTNode::Variable { name: "X".into(), span: Span::unknown() }),
                    span: Span::unknown(),
                },
            ],
            span: Span::unknown(),
        };
        let mut used = std::collections::HashSet::new();
        let out = rename_with_collision_guard(&ast, "A", &mut used, "<test>").unwrap();
        let s = format!("{:?}", out);
        assert!(s.contains("A_X"));
    }

    #[test]
    #[ignore = "AST fixture still uses legacy BoxDeclaration { body }; restore after Phase-31 doc update"]
    fn internal_ref_function_qualified_is_rewritten() {
        // Program: fn call: Helper.run() → after alias P: P_Helper.run()
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::FunctionDeclaration { name: "Helper".into(), params: vec![], body: vec![], is_static: false, is_override: false, span: Span::unknown() },
                ASTNode::FunctionCall { name: "Helper.run".into(), arguments: vec![], span: Span::unknown() },
            ],
            span: Span::unknown(),
        };
        let mut used = std::collections::HashSet::new();
        let out = rename_with_collision_guard(&ast, "P", &mut used, "<test>").unwrap();
        let s = format!("{:?}", out);
        assert!(s.contains("P_Helper.run"));
    }

    #[test]
    fn alias_collision_is_guarded() {
        // Pre-existing prefixed name collides with upcoming alias rename
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::BoxDeclaration { name: "X".into(), is_static: true, body: vec![], span: Span::unknown() },
            ],
            span: Span::unknown(),
        };
        let mut used = std::collections::HashSet::new();
        // Simulate that A_X is already taken
        used.insert("A_X".to_string());
        let out = rename_with_collision_guard(&ast, "A", &mut used, "<test>");
        assert!(out.is_err());
    }

    #[test]
    fn internal_nested_qualified_head_is_rewritten() {
        // Qualified with two dots: Top.Sub.fn() → Alias_Top.Sub.fn()
        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::FunctionDeclaration { name: "Top".into(), params: vec![], body: vec![], is_static: false, is_override: false, span: Span::unknown() },
                ASTNode::FunctionCall { name: "Top.Sub.run".into(), arguments: vec![], span: Span::unknown() },
            ],
            span: Span::unknown(),
        };
        let mut used = std::collections::HashSet::new();
        let out = rename_with_collision_guard(&ast, "Alias", &mut used, "<test>").unwrap();
        let s = format!("{:?}", out);
        assert!(s.contains("Alias_Top.Sub.run"));
    }

    #[test]
    fn method_call_on_prefixed_static_box_is_functioncall() {
        // P_My.greet() → FunctionCall "P_My.greet/0"
        let aliases = hs(&["P"]);
        let ast = ASTNode::MethodCall {
            object: Box::new(ASTNode::Variable { name: "P_My".into(), span: Span::unknown() }),
            method: "greet".into(),
            arguments: vec![],
            span: Span::unknown(),
        };
        let out = desugar_alias_field_access(&ast, &aliases, true);
        match out {
            ASTNode::FunctionCall { name, arguments, .. } => {
                assert_eq!(name, "P_My.greet/0");
                assert!(arguments.is_empty());
            }
            other => panic!("unexpected AST: {:?}", other),
        }
    }
}
