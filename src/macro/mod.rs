//! Macro System scaffolding (Phase 16 – MVP)
//!
//! Goal: Provide minimal, typed interfaces for AST pattern matching and
//! HIR patch based expansion. Backends (MIR/JIT/LLVM) remain unchanged.

pub mod pattern;
pub mod engine;
pub mod macro_box;
pub mod macro_box_ny;
pub mod ast_json;
pub mod ctx;

use nyash_rust::ASTNode;

/// Enable/disable macro system via env gate.
pub fn enabled() -> bool {
    // Default ON. Disable with NYASH_MACRO_DISABLE=1 or NYASH_MACRO_ENABLE=0/false/off.
    if let Ok(v) = std::env::var("NYASH_MACRO_DISABLE") { if v == "1" { return false; } }
    if let Ok(v) = std::env::var("NYASH_MACRO_ENABLE") {
        let v = v.to_ascii_lowercase();
        if v == "0" || v == "false" || v == "off" { return false; }
        return true;
    }
    true
}

/// A hook to dump AST for `--expand` (pre/post). Expansion is no-op for now.
pub fn maybe_expand_and_dump(ast: &ASTNode, _dump_only: bool) -> ASTNode {
    if !enabled() { return ast.clone(); }
    // Initialize user macro boxes (if any, behind env gates)
    self::macro_box::init_builtin();
    self::macro_box_ny::init_from_env();
    if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") { eprintln!("[macro] input AST: {:?}", ast); }
    let mut eng = self::engine::MacroEngine::new();
    let (out, _patches) = eng.expand(ast);
    let out2 = maybe_inject_test_harness(&out);
    if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
        fn count_calls(n: &nyash_rust::ASTNode, acc: &mut std::collections::HashMap<String, usize>) {
            use nyash_rust::ast::ASTNode as A;
            match n.clone() {
                A::Program { statements, .. } => { for s in statements { count_calls(&s, acc); } }
                A::FunctionCall { name, arguments, .. } => { *acc.entry(name).or_insert(0) += 1; for a in arguments { count_calls(&a, acc); } }
                A::MethodCall { object, arguments, .. } => { count_calls(&object, acc); for a in arguments { count_calls(&a, acc); } }
                A::ArrayLiteral { elements, .. } => { for e in elements { count_calls(&e, acc); } }
                A::MapLiteral { entries, .. } => { for (_k, v) in entries { count_calls(&v, acc); } }
                A::BinaryOp { left, right, .. } => { count_calls(&left, acc); count_calls(&right, acc); }
                A::UnaryOp { operand, .. } => { count_calls(&operand, acc); }
                A::Assignment { target, value, .. } => { count_calls(&target, acc); count_calls(&value, acc); }
                A::If { condition, then_body, else_body, .. } => { count_calls(&condition, acc); for s in then_body { count_calls(&s, acc); } if let Some(b) = else_body { for s in b { count_calls(&s, acc); } } }
                _ => {}
            }
        }
        let mut acc = std::collections::HashMap::new();
        count_calls(&out2, &mut acc);
        if !acc.is_empty() { eprintln!("[macro] call census: {:?}", acc); }
        eprintln!("[macro] output AST: {:?}", out2);
    }
    out2
}

fn maybe_inject_test_harness(ast: &ASTNode) -> ASTNode {
    let run_tests = std::env::var("NYASH_TEST_RUN").ok().as_deref() == Some("1")
        || std::env::var("NYASH_RUN_TESTS").ok().as_deref() == Some("1");
    if !run_tests {
        return ast.clone();
    }
    // Test call plan
    #[derive(Clone)]
    struct TestPlan { label: String, setup: Option<nyash_rust::ASTNode>, call: nyash_rust::ASTNode }

    // Collect tests (top-level and Box)
    let mut tests: Vec<TestPlan> = Vec::new();
    let mut has_main_fn = false;
    let mut _main_params_len: usize = 0;

    // Optional JSON args:
    // - Simple: { "test_name": [1, "s", true], "Box.method": [ ... ] }
    // - Detailed per-test: { "Box.method": { "args": [...], "instance": {"ctor":"new|birth","args":[...] } } }
    // - Typed values inside args supported via objects (see json_to_ast below)
    #[derive(Clone, Default)]
    struct InstanceSpec { ctor: String, args: Vec<nyash_rust::ASTNode>, type_args: Vec<String> }
    #[derive(Clone, Default)]
    struct TestArgSpec { args: Vec<nyash_rust::ASTNode>, instance: Option<InstanceSpec> }

    fn json_err(msg: &str) {
        eprintln!("[macro][test][args] {}", msg);
    }

    fn json_to_ast(v: &serde_json::Value) -> Result<nyash_rust::ASTNode, String> {
        use nyash_rust::ast::{ASTNode as A, LiteralValue, Span};
        match v {
            serde_json::Value::String(st) => Ok(A::Literal { value: LiteralValue::String(st.clone()), span: Span::unknown() }),
            serde_json::Value::Bool(b) => Ok(A::Literal { value: LiteralValue::Bool(*b), span: Span::unknown() }),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(A::Literal { value: LiteralValue::Integer(i), span: Span::unknown() })
                } else if let Some(f) = n.as_f64() {
                    Ok(A::Literal { value: LiteralValue::Float(f), span: Span::unknown() })
                } else {
                    Err("unsupported number literal".into())
                }
            }
            serde_json::Value::Null => Ok(A::Literal { value: LiteralValue::Null, span: Span::unknown() }),
            serde_json::Value::Array(elems) => {
                // Treat nested arrays as ArrayLiteral by default
                let mut out = Vec::with_capacity(elems.len());
                for x in elems { out.push(json_to_ast(x)?); }
                Ok(A::ArrayLiteral { elements: out, span: Span::unknown() })
            }
            serde_json::Value::Object(obj) => {
                // Typed shorthands accepted: {i:1}|{int:1}, {f:1.2}|{float:1.2}, {s:"x"}|{string:"x"}, {b:true}|{bool:true}
                if let Some(v) = obj.get("i").or_else(|| obj.get("int")) { return json_to_ast(v); }
                if let Some(v) = obj.get("f").or_else(|| obj.get("float")) { return json_to_ast(v); }
                if let Some(v) = obj.get("s").or_else(|| obj.get("string")) { return json_to_ast(v); }
                if let Some(v) = obj.get("b").or_else(|| obj.get("bool")) { return json_to_ast(v); }
                if let Some(map) = obj.get("map") {
                    if let Some(mo) = map.as_object() {
                        let mut ents: Vec<(String, nyash_rust::ASTNode)> = Vec::with_capacity(mo.len());
                        for (k, vv) in mo { ents.push((k.clone(), json_to_ast(vv)?)); }
                        return Ok(A::MapLiteral { entries: ents, span: Span::unknown() });
                    } else { return Err("map must be an object".into()); }
                }
                if let Some(arr) = obj.get("array") {
                    if let Some(va) = arr.as_array() {
                        let mut out = Vec::with_capacity(va.len());
                        for x in va { out.push(json_to_ast(x)?); }
                        return Ok(A::ArrayLiteral { elements: out, span: Span::unknown() });
                    } else { return Err("array must be an array".into()); }
                }
                if let Some(name) = obj.get("var").and_then(|v| v.as_str()) {
                    return Ok(A::Variable { name: name.to_string(), span: Span::unknown() });
                }
                if let Some(name) = obj.get("call").and_then(|v| v.as_str()) {
                    let mut args: Vec<A> = Vec::new();
                    if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                        for x in va { args.push(json_to_ast(x)?); }
                    }
                    return Ok(A::FunctionCall { name: name.to_string(), arguments: args, span: Span::unknown() });
                }
                if let Some(method) = obj.get("method").and_then(|v| v.as_str()) {
                    let objv = obj.get("object").ok_or_else(|| "method requires 'object'".to_string())?;
                    let object = json_to_ast(objv)?;
                    let mut args: Vec<A> = Vec::new();
                    if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                        for x in va { args.push(json_to_ast(x)?); }
                    }
                    return Ok(A::MethodCall { object: Box::new(object), method: method.to_string(), arguments: args, span: Span::unknown() });
                }
                if let Some(bx) = obj.get("box").and_then(|v| v.as_str()) {
                    let mut args: Vec<A> = Vec::new();
                    if let Some(va) = obj.get("args").and_then(|v| v.as_array()) {
                        for x in va { args.push(json_to_ast(x)?); }
                    }
                    let type_args: Vec<String> = obj.get("type_args").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
                    let ctor = obj.get("ctor").and_then(|v| v.as_str()).unwrap_or("new");
                    if ctor == "new" {
                        return Ok(A::New { class: bx.to_string(), arguments: args, type_arguments: type_args, span: Span::unknown() });
                    } else if ctor == "birth" {
                        return Ok(A::MethodCall { object: Box::new(A::Variable { name: bx.to_string(), span: Span::unknown() }), method: "birth".into(), arguments: args, span: Span::unknown() });
                    } else {
                        return Err(format!("unknown ctor '{}', expected 'new' or 'birth'", ctor));
                    }
                }
                Err("unknown object mapping for AST".into())
            }
        }
    }

    fn parse_test_arg_spec(v: &serde_json::Value) -> Option<TestArgSpec> {
        match v {
            serde_json::Value::Array(arr) => {
                let mut out: Vec<nyash_rust::ASTNode> = Vec::new();
                for a in arr { match json_to_ast(a) { Ok(n) => out.push(n), Err(e) => { json_err(&format!("args element error: {}", e)); return None; } } }
                Some(TestArgSpec { args: out, instance: None })
            }
            serde_json::Value::Object(obj) => {
                let mut spec = TestArgSpec::default();
                if let Some(a) = obj.get("args").and_then(|v| v.as_array()) {
                    let mut out: Vec<nyash_rust::ASTNode> = Vec::new();
                    for x in a { match json_to_ast(x) { Ok(n) => out.push(n), Err(e) => { json_err(&format!("args element error: {}", e)); return None; } } }
                    spec.args = out;
                }
                if let Some(inst) = obj.get("instance").and_then(|v| v.as_object()) {
                    let ctor = inst.get("ctor").and_then(|v| v.as_str()).unwrap_or("new").to_string();
                    let type_args: Vec<String> = inst.get("type_args").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
                    let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                    if let Some(va) = inst.get("args").and_then(|v| v.as_array()) {
                        for x in va { match json_to_ast(x) { Ok(n) => args.push(n), Err(e) => { json_err(&format!("instance.args element error: {}", e)); return None; } } }
                    }
                    spec.instance = Some(InstanceSpec { ctor, args, type_args });
                }
                Some(spec)
            }
            _ => { json_err("test value must be array or object"); None }
        }
    }

    let args_map: Option<std::collections::HashMap<String, TestArgSpec>> = (|| {
        if let Ok(s) = std::env::var("NYASH_TEST_ARGS_JSON") {
            if s.trim().is_empty() { return None; }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                let mut map = std::collections::HashMap::new();
                if let Some(obj) = v.as_object() {
                    for (k, vv) in obj { if let Some(spec) = parse_test_arg_spec(vv) { map.insert(k.clone(), spec); } }
                    return Some(map);
                }
            }
        }
        None
    })();
    if let nyash_rust::ASTNode::Program { statements, .. } = ast {
        for st in statements {
            match st {
                nyash_rust::ASTNode::FunctionDeclaration { name, params, .. } => {
                    if name == "main" { has_main_fn = true; _main_params_len = params.len(); }
                    if name.starts_with("test_") {
                        let label = name.clone();
                        // select args: JSON map > defaults > skip
                        let mut maybe_args: Option<Vec<nyash_rust::ASTNode>> = None;
                        if let Some(m) = &args_map { if let Some(v) = m.get(&label) { maybe_args = Some(v.args.clone()); } }
                        let args = if let Some(a) = maybe_args { a }
                                  else if !params.is_empty() && std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref() == Some("1") {
                                      let mut a: Vec<nyash_rust::ASTNode> = Vec::new(); for _ in params { a.push(nyash_rust::ASTNode::Literal{ value: nyash_rust::ast::LiteralValue::Integer(0), span: nyash_rust::ast::Span::unknown() }); } a
                                  } else if params.is_empty() { Vec::new() }
                                  else {
                                      eprintln!("[macro][test][args] missing args for {} (need {}), skipping (set NYASH_TEST_ARGS_DEFAULTS=1 for zero defaults)", label, params.len());
                                      continue
                                  };
                        tests.push(TestPlan { label, setup: None, call: nyash_rust::ASTNode::FunctionCall { name: name.clone(), arguments: args, span: nyash_rust::ast::Span::unknown() } });
                    }
                }
                _ => {}
            }
        }
    }
    // Collect Box tests: static and instance (no-arg only for instance)
    if let nyash_rust::ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let nyash_rust::ASTNode::BoxDeclaration { name: box_name, methods, .. } = st {
                for (mname, mnode) in methods {
                    if !mname.starts_with("test_") { continue; }
                    if let nyash_rust::ASTNode::FunctionDeclaration { is_static, params, .. } = mnode {
                        if *is_static {
                            // Static: BoxName.test_*()
                            let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                            if let Some(m) = &args_map { if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) { args = v.args.clone(); } }
                            if args.is_empty() && !params.is_empty() {
                                if std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref() == Some("1") { for _ in params { args.push(nyash_rust::ASTNode::Literal { value: nyash_rust::ast::LiteralValue::Integer(0), span: nyash_rust::ast::Span::unknown() }); } }
                                else {
                                    eprintln!("[macro][test][args] missing args for {}.{} (need {}), skipping", box_name, mname, params.len());
                                    continue;
                                }
                            }
                            let call = nyash_rust::ASTNode::MethodCall {
                                object: Box::new(nyash_rust::ASTNode::Variable { name: box_name.clone(), span: nyash_rust::ast::Span::unknown() }),
                                method: mname.clone(),
                                arguments: args,
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            tests.push(TestPlan { label: format!("{}.{}", box_name, mname), setup: None, call });
                        } else {
                            // Instance: try new BoxName() then .test_*()
                            let inst_var = format!("__t_{}", box_name.to_lowercase());
                            // Instance override via JSON
                            let mut inst_ctor: Option<InstanceSpec> = None;
                            if let Some(m) = &args_map { if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) { inst_ctor = v.instance.clone(); } }
                            let inst_init: nyash_rust::ASTNode = if let Some(spec) = inst_ctor {
                                match spec.ctor.as_str() {
                                    "new" => nyash_rust::ASTNode::New { class: box_name.clone(), arguments: spec.args, type_arguments: spec.type_args, span: nyash_rust::ast::Span::unknown() },
                                    "birth" => nyash_rust::ASTNode::MethodCall { object: Box::new(nyash_rust::ASTNode::Variable { name: box_name.clone(), span: nyash_rust::ast::Span::unknown() }), method: "birth".into(), arguments: spec.args, span: nyash_rust::ast::Span::unknown() },
                                    other => {
                                        eprintln!("[macro][test][args] unknown ctor '{}' for {}.{}, using new()", other, box_name, mname);
                                        nyash_rust::ASTNode::New { class: box_name.clone(), arguments: vec![], type_arguments: vec![], span: nyash_rust::ast::Span::unknown() }
                                    }
                                }
                            } else {
                                nyash_rust::ASTNode::New { class: box_name.clone(), arguments: vec![], type_arguments: vec![], span: nyash_rust::ast::Span::unknown() }
                            };
                            let setup = nyash_rust::ASTNode::Local {
                                variables: vec![inst_var.clone()],
                                initial_values: vec![Some(Box::new(inst_init))],
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                            if let Some(m) = &args_map { if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) { args = v.args.clone(); } }
                            if args.is_empty() && !params.is_empty() {
                                if std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref() == Some("1") { for _ in params { args.push(nyash_rust::ASTNode::Literal { value: nyash_rust::ast::LiteralValue::Integer(0), span: nyash_rust::ast::Span::unknown() }); } }
                                else {
                                    eprintln!("[macro][test][args] missing args for {}.{} (need {}), skipping", box_name, mname, params.len());
                                    continue;
                                }
                            }
                            let call = nyash_rust::ASTNode::MethodCall {
                                object: Box::new(nyash_rust::ASTNode::Variable { name: inst_var.clone(), span: nyash_rust::ast::Span::unknown() }),
                                method: mname.clone(),
                                arguments: args,
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            tests.push(TestPlan { label: format!("{}.{}", box_name, mname), setup: Some(setup), call });
                        }
                    }
                }
            }
        }
    }
    // Filter
    if let Ok(substr) = std::env::var("NYASH_TEST_FILTER") {
        if !substr.is_empty() {
            tests.retain(|tp| tp.label.contains(&substr));
        }
    }
    if tests.is_empty() {
        if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") { eprintln!("[macro][test] no tests found (functions starting with 'test_')"); }
        return ast.clone();
    }
    // Decide entry policy when main exists
    let force = std::env::var("NYASH_TEST_FORCE").ok().as_deref() == Some("1");
    let entry_mode = std::env::var("NYASH_TEST_ENTRY").ok(); // Some("wrap"|"override")
    let ret_policy = std::env::var("NYASH_TEST_RETURN").ok(); // Some("tests"|"original") default tests

    // Build harness: top-level function main(args) { ... }
    use nyash_rust::ast::{ASTNode as A, Span, LiteralValue, BinaryOperator};
    let mut body: Vec<A> = Vec::new();
    // locals: pass=0, fail=0
    body.push(A::Local { variables: vec!["pass".into(), "fail".into()], initial_values: vec![Some(Box::new(A::Literal{ value: LiteralValue::Integer(0), span: Span::unknown()})), Some(Box::new(A::Literal{ value: LiteralValue::Integer(0), span: Span::unknown()}))], span: Span::unknown() });
    for tp in &tests {
        // optional setup
        if let Some(set) = tp.setup.clone() { body.push(set); }
        // local r = CALL
        body.push(A::Local { variables: vec!["r".into()], initial_values: vec![Some(Box::new(tp.call.clone()))], span: Span::unknown() });
        // if r { print("PASS t"); pass = pass + 1 } else { print("FAIL t"); fail = fail + 1 }
        let pass_msg = A::Literal { value: LiteralValue::String(format!("PASS {}", tp.label)), span: Span::unknown() };
        let fail_msg = A::Literal { value: LiteralValue::String(format!("FAIL {}", tp.label)), span: Span::unknown() };
        let then_body = vec![
            A::Print { expression: Box::new(pass_msg), span: Span::unknown() },
            A::Assignment { target: Box::new(A::Variable{ name: "pass".into(), span: Span::unknown() }), value: Box::new(A::BinaryOp{ operator: BinaryOperator::Add, left: Box::new(A::Variable{ name: "pass".into(), span: Span::unknown() }), right: Box::new(A::Literal{ value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() },
        ];
        let else_body = vec![
            A::Print { expression: Box::new(fail_msg), span: Span::unknown() },
            A::Assignment { target: Box::new(A::Variable{ name: "fail".into(), span: Span::unknown() }), value: Box::new(A::BinaryOp{ operator: BinaryOperator::Add, left: Box::new(A::Variable{ name: "fail".into(), span: Span::unknown() }), right: Box::new(A::Literal{ value: LiteralValue::Integer(1), span: Span::unknown() }), span: Span::unknown() }), span: Span::unknown() },
        ];
        body.push(A::If { condition: Box::new(A::Variable { name: "r".into(), span: Span::unknown() }), then_body, else_body: Some(else_body), span: Span::unknown() });
    }
    // print summary and return fail
    body.push(A::Print { expression: Box::new(A::Literal{ value: LiteralValue::String(format!("Summary: {} tests", tests.len())), span: Span::unknown() }), span: Span::unknown() });
    body.push(A::Return { value: Some(Box::new(A::Variable{ name: "fail".into(), span: Span::unknown() })), span: Span::unknown() });
    // Build harness main body as above
    let make_harness_main = |body: Vec<A>| -> A {
        A::FunctionDeclaration { name: "main".into(), params: vec!["args".into()], body, is_static: false, is_override: false, span: Span::unknown() }
    };

    // Transform AST according to policy
    if let nyash_rust::ASTNode::Program { statements, span } = ast.clone() {
        let mut out_stmts: Vec<A> = Vec::with_capacity(statements.len() + 1);
        let mut _renamed_main = false;
        let mut orig_call_fn: Option<A> = None;
        for st in statements {
            match st {
                A::FunctionDeclaration { name, params, body: orig_body, is_static, is_override, span: fspan } if name == "main" => {
                    if has_main_fn && (force || entry_mode.is_some()) {
                        // rename original main
                        let new_name = "__ny_orig_main".to_string();
                        out_stmts.push(A::FunctionDeclaration { name: new_name.clone(), params: params.clone(), body: orig_body.clone(), is_static, is_override, span: fspan });
                        _renamed_main = true;
                        if entry_mode.as_deref() == Some("wrap") {
                            let args_exprs = if params.len() >= 1 { vec![A::Variable { name: "args".into(), span: nyash_rust::ast::Span::unknown() }] } else { vec![] };
                            orig_call_fn = Some(A::FunctionCall { name: new_name, arguments: args_exprs, span: nyash_rust::ast::Span::unknown() });
                        }
                    } else {
                        // keep as-is (no injection)
                        out_stmts.push(A::FunctionDeclaration { name, params, body: orig_body, is_static, is_override, span: fspan });
                    }
                }
                other => out_stmts.push(other),
            }
        }
        if has_main_fn && !(force || entry_mode.is_some()) {
            if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") { eprintln!("[macro][test] existing main detected; skip harness (set --test-entry or NYASH_TEST_FORCE=1)"); }
            return nyash_rust::ASTNode::Program { statements: out_stmts, span };
        }
        // Compose harness main now
        let mut body2 = body;
        // Summary is already included in body. Append call/return per policy.
        if let Some(call) = orig_call_fn.take() {
            if ret_policy.as_deref() == Some("original") {
                // local __ny_orig_ret = __ny_orig_main(args)
                body2.push(A::Local { variables: vec!["__ny_orig_ret".into()], initial_values: vec![Some(Box::new(call))], span: nyash_rust::ast::Span::unknown() });
                // return __ny_orig_ret
                body2.push(A::Return { value: Some(Box::new(A::Variable { name: "__ny_orig_ret".into(), span: nyash_rust::ast::Span::unknown() })), span: nyash_rust::ast::Span::unknown() });
            } else {
                // default: tests policy; still call original but ignore result
                body2.push(call);
                // return fail already appended earlier
            }
        }
        let harness_fn = make_harness_main(body2);
        out_stmts.push(harness_fn);
        return nyash_rust::ASTNode::Program { statements: out_stmts, span };
    }
    ast.clone()
}
