use nyash_rust::ASTNode;

/// Load MacroBoxes written in Nyash.
/// Preferred env: NYASH_MACRO_PATHS=comma,separated,paths
/// Backward compat: NYASH_MACRO_BOX_NY=1 + NYASH_MACRO_BOX_NY_PATHS
pub fn init_from_env() {
    // Preferred: NYASH_MACRO_PATHS
    let paths = match std::env::var("NYASH_MACRO_PATHS") {
        Ok(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    }
    .or_else(|| {
        // Back-compat: NYASH_MACRO_BOX_NY / NYASH_MACRO_BOX_NY_PATHS
        if std::env::var("NYASH_MACRO_BOX_NY").ok().as_deref() == Some("1") {
            if let Ok(s) = std::env::var("NYASH_MACRO_BOX_NY_PATHS") {
                if !s.trim().is_empty() {
                    eprintln!("[macro][compat] NYASH_MACRO_BOX_NY*_ vars are deprecated; use NYASH_MACRO_PATHS");
                    return Some(s);
                }
            }
        }
        None
    });
    // Soft deprecations for legacy envs
    if std::env::var("NYASH_MACRO_TOPLEVEL_ALLOW").ok().is_some() {
        eprintln!("[macro][compat] NYASH_MACRO_TOPLEVEL_ALLOW is deprecated; default is OFF. Prefer CLI --macro-top-level-allow if needed");
    }
    if std::env::var("NYASH_MACRO_BOX_CHILD_RUNNER").ok().is_some() {
        eprintln!("[macro][compat] NYASH_MACRO_BOX_CHILD_RUNNER is deprecated; runner mode is managed automatically");
    }
    let Some(paths) = paths else { return; };
    for p in paths.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if let Err(e) = try_load_one(p) {
            eprintln!("[macro][box_ny] failed to load '{}': {}", p, e);
        }
    }
}

fn try_load_one(path: &str) -> Result<(), String> {
    let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    // Enable minimal sugar for macro files during scanning (array/map literals etc.)
    let prev_sugar = std::env::var("NYASH_SYNTAX_SUGAR_LEVEL").ok();
    std::env::set_var("NYASH_SYNTAX_SUGAR_LEVEL", "basic");
    let ast_res = nyash_rust::parser::NyashParser::parse_from_string(&src);
    if let Some(v) = prev_sugar { std::env::set_var("NYASH_SYNTAX_SUGAR_LEVEL", v); } else { std::env::remove_var("NYASH_SYNTAX_SUGAR_LEVEL"); }
    let ast = ast_res.map_err(|e| format!("parse error: {:?}", e))?;
    // Find a BoxDeclaration with static function expand(...)
    if let ASTNode::Program { statements, .. } = ast {
        // Capabilities: conservative scan before registration
        if let Err(msg) = caps_allow_macro_source(&ASTNode::Program { statements: statements.clone(), span: nyash_rust::ast::Span::unknown() }) {
            eprintln!("[macro][box_ny][caps] {} (in '{}')", msg, path);
            if strict_enabled() { return Err(msg); }
            return Ok(());
        }
        for st in &statements {
            if let ASTNode::BoxDeclaration { name: box_name, methods, .. } = st {
                if let Some(ASTNode::FunctionDeclaration { name: mname, body: exp_body, params, .. }) = methods.get("expand") {
                    if mname == "expand" {
                        let reg_name = derive_box_name(&box_name, methods.get("name"));
                        // Prefer Nyash runner route by default (self-hosting). Child-proxy only when explicitly enabled.
                        let use_child = std::env::var("NYASH_MACRO_BOX_CHILD").ok().map(|v| v != "0" && v != "false" && v != "off").unwrap_or(true);
                        if use_child {
                            let nm = reg_name;
                            let file_static: &'static str = Box::leak(path.to_string().into_boxed_str());
                            crate::r#macro::macro_box::register(Box::leak(Box::new(NyChildMacroBox { nm, file: file_static })));
                            eprintln!("[macro][box_ny] registered child-proxy MacroBox '{}' for {}", nm, path);
                        } else {
                            // Heuristic mapping by name first, otherwise inspect body pattern.
                            let mut mapped = false;
                            match reg_name {
                                "UppercasePrintMacro" => {
                                    crate::r#macro::macro_box::register(&crate::r#macro::macro_box::UppercasePrintMacro);
                                    eprintln!("[macro][box_ny] registered built-in '{}' from {}", reg_name, path);
                                    mapped = true;
                                }
                                _ => {}
                            }
                            if !mapped {
                                if expand_is_identity(exp_body, params) {
                                    let nm = reg_name;
                                    crate::r#macro::macro_box::register(Box::leak(Box::new(NyIdentityMacroBox { nm })));
                                    eprintln!("[macro][box_ny] registered Ny MacroBox '{}' (identity by body) from {}", nm, path);
                                } else if expand_indicates_uppercase(exp_body, params) {
                                    crate::r#macro::macro_box::register(&crate::r#macro::macro_box::UppercasePrintMacro);
                                    eprintln!("[macro][box_ny] registered built-in 'UppercasePrintMacro' by body pattern from {}", path);
                                } else {
                                    let nm = reg_name;
                                    crate::r#macro::macro_box::register(Box::leak(Box::new(NyIdentityMacroBox { nm })));
                                    eprintln!("[macro][box_ny] registered Ny MacroBox '{}' (identity: unknown body) from {}", nm, path);
                                }
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }
        // Fallback: accept top-level `static function MacroBoxSpec.expand(json)` without a BoxDeclaration
        // Default OFF for safety; can be enabled via CLI/env
        let allow_top = std::env::var("NYASH_MACRO_TOPLEVEL_ALLOW").ok().map(|v| v != "0" && v != "false" && v != "off").unwrap_or(false);
        for st in &statements {
            if let ASTNode::FunctionDeclaration { is_static: true, name, .. } = st {
                if let Some((box_name, method)) = name.split_once('.') {
                    if method == "expand" {
                        let nm: &'static str = Box::leak(box_name.to_string().into_boxed_str());
                        let file_static: &'static str = Box::leak(path.to_string().into_boxed_str());
                        let use_child = std::env::var("NYASH_MACRO_BOX_CHILD").ok().map(|v| v != "0" && v != "false" && v != "off").unwrap_or(true);
                        if use_child && allow_top {
                            crate::r#macro::macro_box::register(Box::leak(Box::new(NyChildMacroBox { nm, file: file_static })));
                            eprintln!("[macro][box_ny] registered child-proxy MacroBox '{}' (top-level static) for {}", nm, path);
                        } else {
                            crate::r#macro::macro_box::register(Box::leak(Box::new(NyIdentityMacroBox { nm })));
                            eprintln!("[macro][box_ny] registered identity MacroBox '{}' (top-level static) for {}", nm, path);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
    Err("no Box with static expand(ast) found".into())
}

fn derive_box_name(default: &str, name_fn: Option<&ASTNode>) -> &'static str {
    // If name() { return "X" } pattern is detected, use it; else box name
    if let Some(ASTNode::FunctionDeclaration { body, .. }) = name_fn {
        if body.len() == 1 {
            if let ASTNode::Return { value: Some(v), .. } = &body[0] {
                if let ASTNode::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } = &**v {
                    let owned = s.clone();
                    return Box::leak(owned.into_boxed_str());
                }
            }
        }
    }
    Box::leak(default.to_string().into_boxed_str())
}

pub(crate) struct NyIdentityMacroBox { nm: &'static str }

impl super::macro_box::MacroBox for NyIdentityMacroBox {
    fn name(&self) -> &'static str { self.nm }
    fn expand(&self, ast: &ASTNode) -> ASTNode {
        if std::env::var("NYASH_MACRO_BOX_NY_IDENTITY_ROUNDTRIP").ok().as_deref() == Some("1") {
            let j = crate::r#macro::ast_json::ast_to_json(ast);
            if let Some(a2) = crate::r#macro::ast_json::json_to_ast(&j) { return a2; }
        }
        ast.clone()
    }
}

fn expand_is_identity(body: &Vec<ASTNode>, params: &Vec<String>) -> bool {
    if body.len() != 1 { return false; }
    if let ASTNode::Return { value: Some(v), .. } = &body[0] {
        if let ASTNode::Variable { name, .. } = &**v {
            return params.get(0).map(|p| p == name).unwrap_or(false);
        }
    }
    false
}

fn expand_indicates_uppercase(body: &Vec<ASTNode>, params: &Vec<String>) -> bool {
    if body.len() != 1 { return false; }
    let p0 = params.get(0).cloned().unwrap_or_else(|| "ast".to_string());
    match &body[0] {
        ASTNode::Return { value: Some(v), .. } => match &**v {
            ASTNode::FunctionCall { name, arguments, .. } => {
                if (name == "uppercase_print" || name == "upper_print") && arguments.len() == 1 {
                    if let ASTNode::Variable { name: an, .. } = &arguments[0] {
                        return an == &p0;
                    }
                }
                false
            }
            _ => false,
        },
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroBehavior { Identity, Uppercase, ArrayPrependZero, MapInsertTag, LoopNormalize, IfMatchNormalize }

pub fn analyze_macro_file(path: &str) -> MacroBehavior {
    let src = match std::fs::read_to_string(path) { Ok(s) => s, Err(_) => return MacroBehavior::Identity };
    let ast = match nyash_rust::parser::NyashParser::parse_from_string(&src) { Ok(a) => a, Err(_) => return MacroBehavior::Identity };
    // Quick heuristics based on literals present in file
    fn ast_has_literal_string(a: &ASTNode, needle: &str) -> bool {
        use nyash_rust::ast::ASTNode as A;
        match a {
            A::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } => s.contains(needle),
            A::Program { statements, .. } => statements.iter().any(|n| ast_has_literal_string(n, needle)),
            A::Print { expression, .. } => ast_has_literal_string(expression, needle),
            A::Return { value, .. } => value.as_ref().map(|v| ast_has_literal_string(v, needle)).unwrap_or(false),
            A::Assignment { target, value, .. } => ast_has_literal_string(target, needle) || ast_has_literal_string(value, needle),
            A::If { condition, then_body, else_body, .. } => {
                ast_has_literal_string(condition, needle)
                    || then_body.iter().any(|n| ast_has_literal_string(n, needle))
                    || else_body.as_ref().map(|v| v.iter().any(|n| ast_has_literal_string(n, needle))).unwrap_or(false)
            }
            A::FunctionDeclaration { body, .. } => body.iter().any(|n| ast_has_literal_string(n, needle)),
            A::BinaryOp { left, right, .. } => ast_has_literal_string(left, needle) || ast_has_literal_string(right, needle),
            A::UnaryOp { operand, .. } => ast_has_literal_string(operand, needle),
            A::MethodCall { object, arguments, .. } => ast_has_literal_string(object, needle) || arguments.iter().any(|n| ast_has_literal_string(n, needle)),
            A::FunctionCall { arguments, .. } => arguments.iter().any(|n| ast_has_literal_string(n, needle)),
            A::ArrayLiteral { elements, .. } => elements.iter().any(|n| ast_has_literal_string(n, needle)),
            A::MapLiteral { entries, .. } => entries.iter().any(|(_, v)| ast_has_literal_string(v, needle)),
            _ => false,
        }
    }
    fn ast_has_method(a: &ASTNode, method: &str) -> bool {
        use nyash_rust::ast::ASTNode as A;
        match a {
            A::Program { statements, .. } => statements.iter().any(|n| ast_has_method(n, method)),
            A::Print { expression, .. } => ast_has_method(expression, method),
            A::Return { value, .. } => value.as_ref().map(|v| ast_has_method(v, method)).unwrap_or(false),
            A::Assignment { target, value, .. } => ast_has_method(target, method) || ast_has_method(value, method),
            A::If { condition, then_body, else_body, .. } => ast_has_method(condition, method)
                || then_body.iter().any(|n| ast_has_method(n, method))
                || else_body.as_ref().map(|v| v.iter().any(|n| ast_has_method(n, method))).unwrap_or(false),
            A::FunctionDeclaration { body, .. } => body.iter().any(|n| ast_has_method(n, method)),
            A::BinaryOp { left, right, .. } => ast_has_method(left, method) || ast_has_method(right, method),
            A::UnaryOp { operand, .. } => ast_has_method(operand, method),
            A::MethodCall { object, method: m, arguments, .. } => m == method
                || ast_has_method(object, method)
                || arguments.iter().any(|n| ast_has_method(n, method)),
            A::FunctionCall { arguments, .. } => arguments.iter().any(|n| ast_has_method(n, method)),
            A::ArrayLiteral { elements, .. } => elements.iter().any(|n| ast_has_method(n, method)),
            A::MapLiteral { entries, .. } => entries.iter().any(|(_, v)| ast_has_method(v, method)),
            _ => false,
        }
    }
    // Detect array prepend-zero macro by pattern strings present in macro source
    if ast_has_literal_string(&ast, "\"kind\":\"Array\",\"elements\":[") || ast_has_literal_string(&ast, "\"elements\":[") {
        return MacroBehavior::ArrayPrependZero;
    }
    // Detect map insert-tag macro by pattern strings
    if ast_has_literal_string(&ast, "\"kind\":\"Map\",\"entries\":[") || ast_has_literal_string(&ast, "\"entries\":[") {
        return MacroBehavior::MapInsertTag;
    }
    // Detect upper-string macro by pattern or toUpperCase usage
    if ast_has_literal_string(&ast, "\"value\":\"UPPER:") || ast_has_method(&ast, "toUpperCase") {
        return MacroBehavior::Uppercase;
    }
    if let ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let ASTNode::BoxDeclaration { name: _, methods, .. } = st {
                // Detect LoopNormalize/IfMatchNormalize by name() returning a specific string
                if let Some(ASTNode::FunctionDeclaration { name: mname, body, .. }) = methods.get("name") {
                    if mname == "name" {
                        if body.len() == 1 {
                            if let ASTNode::Return { value: Some(v), .. } = &body[0] {
                                if let ASTNode::Literal { value: nyash_rust::ast::LiteralValue::String(s), .. } = &**v {
                                    if s == "LoopNormalize" { return MacroBehavior::LoopNormalize; }
                                    if s == "IfMatchNormalize" { return MacroBehavior::IfMatchNormalize; }
                                }
                            }
                        }
                    }
                }
                if let Some(ASTNode::FunctionDeclaration { name: mname, body, params, .. }) = methods.get("expand") {
                    if mname == "expand" {
                        if expand_indicates_uppercase(body, params) {
                            return MacroBehavior::Uppercase;
                        }
                    }
                }
            }
        }
    }
    MacroBehavior::Identity
}

struct NyChildMacroBox { nm: &'static str, file: &'static str }

fn cap_enabled(name: &str) -> bool {
    match std::env::var(name).ok() {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        }
        None => false,
    }
}

fn caps_allow_macro_source(ast: &ASTNode) -> Result<(), String> {
    let allow_io = cap_enabled("NYASH_MACRO_CAP_IO");
    let allow_net = cap_enabled("NYASH_MACRO_CAP_NET");
    use nyash_rust::ast::ASTNode as A;
    fn scan(n: &A, seen: &mut Vec<String>) {
        match n {
            A::New { class, .. } => seen.push(class.clone()),
            A::Program { statements, .. } => for s in statements { scan(s, seen); },
            A::FunctionDeclaration { body, .. } => for s in body { scan(s, seen); },
            A::Assignment { target, value, .. } => { scan(target, seen); scan(value, seen); },
            A::Return { value, .. } => if let Some(v) = value { scan(v, seen); },
            A::If { condition, then_body, else_body, .. } => {
                scan(condition, seen);
                for s in then_body { scan(s, seen); }
                if let Some(b) = else_body { for s in b { scan(s, seen); } }
            }
            A::BinaryOp { left, right, .. } => { scan(left, seen); scan(right, seen); }
            A::UnaryOp { operand, .. } => scan(operand, seen),
            A::MethodCall { object, arguments, .. } => { scan(object, seen); for a in arguments { scan(a, seen); } }
            A::FunctionCall { arguments, .. } => for a in arguments { scan(a, seen); },
            A::ArrayLiteral { elements, .. } => for e in elements { scan(e, seen); },
            A::MapLiteral { entries, .. } => for (_, v) in entries { scan(v, seen); },
            _ => {}
        }
    }
    let mut boxes = Vec::new();
    scan(ast, &mut boxes);
    if !allow_io && boxes.iter().any(|c| c == "FileBox" || c == "PathBox" || c == "DirBox") {
        return Err("macro capability violation: IO (File/Path/Dir) denied".into());
    }
    if !allow_net && boxes.iter().any(|c| c.contains("HTTP") || c.contains("Http") || c == "SocketBox") {
        return Err("macro capability violation: NET (HTTP/Socket) denied".into());
    }
    Ok(())
}

impl super::macro_box::MacroBox for NyChildMacroBox {
    fn name(&self) -> &'static str { self.nm }
    fn expand(&self, ast: &ASTNode) -> ASTNode {
        // Parent-side proxy: prefer runner script (PyVM) when enabled; otherwise fallback to internal child mode.
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => { eprintln!("[macro-proxy] current_exe failed: {}", e); return ast.clone(); }
        };
        // Prefer Nyash runner route by default for self-hosting; legacy env can force internal child with 0.
        let use_runner = std::env::var("NYASH_MACRO_BOX_CHILD_RUNNER").ok().map(|v| v != "0" && v != "false" && v != "off").unwrap_or(false);
        if std::env::var("NYASH_MACRO_BOX_CHILD_RUNNER").ok().is_some() {
            eprintln!("[macro][compat] NYASH_MACRO_BOX_CHILD_RUNNER is deprecated; prefer defaults");
        }
        let mut cmd = std::process::Command::new(exe.clone());
        if use_runner {
            // Synthesize a tiny runner that inlines the macro file and calls MacroBoxSpec.expand
            use std::io::Write as _;
            let tmp_dir = std::path::Path::new("tmp");
            let _ = std::fs::create_dir_all(tmp_dir);
            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
            let tmp_path = tmp_dir.join(format!("macro_expand_runner_{}.nyash", ts));
            let mut f = match std::fs::File::create(&tmp_path) { Ok(x) => x, Err(e) => { eprintln!("[macro-proxy] create tmp runner failed: {}", e); return ast.clone(); } };
            let macro_src = std::fs::read_to_string(self.file)
                .unwrap_or_else(|_| String::from("// failed to read macro file\n"));
            let script = format!(
                "{}\n\nfunction main(args) {{\n    if args.length() == 0 {{\n        print(\"{{}}\")\n        return 0\n    }}\n    local j, r, ctx\n    j = args.get(0)\n    if args.length() > 1 {{ ctx = args.get(1) }} else {{ ctx = \"{{}}\" }}\n    r = MacroBoxSpec.expand(j, ctx)\n    print(r)\n    return 0\n}}\n",
                macro_src
            );
            if let Err(e) = f.write_all(script.as_bytes()) { eprintln!("[macro-proxy] write tmp runner failed: {}", e); return ast.clone(); }
            // Run Nyash runner script under PyVM: nyash --backend vm <tmp_runner> -- <json>
            cmd.arg("--backend").arg("vm").arg(tmp_path);
            // Append script args after '--'
            let j = crate::r#macro::ast_json::ast_to_json(ast).to_string();
            cmd.arg("--").arg(j);
            // Provide MacroCtx as JSON (caps only, MVP)
            let mctx = crate::r#macro::ctx::MacroCtx::from_env();
            let ctx_json = format!("{{\"caps\":{{\"io\":{},\"net\":{},\"env\":{}}}}}", mctx.caps.io, mctx.caps.net, mctx.caps.env);
            cmd.arg(ctx_json);
            cmd.stdin(std::process::Stdio::null());
        } else {
            // Internal child mode: --macro-expand-child <macro file> with stdin JSON
            cmd.arg("--macro-expand-child").arg(self.file)
                .stdin(std::process::Stdio::piped());
        }
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        // Sandbox env (PoC): prefer PyVM; disable plugins; enable minimal syntax sugar for macros
        cmd.env("NYASH_VM_USE_PY", "1");
        cmd.env("NYASH_DISABLE_PLUGINS", "1");
        cmd.env("NYASH_SYNTAX_SUGAR_LEVEL", "basic");
        // Disable macro system inside child to avoid recursive registration/expansion
        cmd.env("NYASH_MACRO_ENABLE", "0");
        cmd.env_remove("NYASH_MACRO_PATHS");
        cmd.env_remove("NYASH_MACRO_BOX_NY");
        cmd.env_remove("NYASH_MACRO_BOX_NY_PATHS");
        cmd.env_remove("NYASH_MACRO_BOX_CHILD");
        cmd.env_remove("NYASH_MACRO_BOX_CHILD_RUNNER");
        // Timeout
        let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
        // Spawn
        let mut child = match cmd.spawn() { Ok(c) => c, Err(e) => {
            eprintln!("[macro-proxy] spawn failed: {}", e);
            if strict_enabled() { std::process::exit(2); }
            return ast.clone();
        } };
        // Write stdin only in internal child mode
        if !use_runner {
            if let Some(mut sin) = child.stdin.take() {
                let j = crate::r#macro::ast_json::ast_to_json(ast).to_string();
                use std::io::Write;
                let _ = sin.write_all(j.as_bytes());
            }
        }
        // Wait with timeout
        use std::time::{Duration, Instant};
        let start = Instant::now();
        let mut out = String::new();
        let mut status_opt = None;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    status_opt = Some(status);
                    if let Some(mut so) = child.stdout.take() { use std::io::Read; let _ = so.read_to_string(&mut out); }
                    break;
                }
                Ok(None) => {
                    if start.elapsed() >= Duration::from_millis(timeout_ms) {
                        let _ = child.kill(); let _ = child.wait();
                        eprintln!("[macro-proxy] timeout {} ms", timeout_ms);
                        if strict_enabled() { std::process::exit(124); }
                        return ast.clone();
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => { eprintln!("[macro-proxy] wait error: {}", e); if strict_enabled() { std::process::exit(2); } return ast.clone(); }
            }
        }
        // Capture stderr for diagnostics
        let mut err = String::new();
        if let Some(mut se) = child.stderr.take() { use std::io::Read; let _ = se.read_to_string(&mut err); }
        // Parse output JSON
        match serde_json::from_str::<serde_json::Value>(&out) {
            Ok(v) => match crate::r#macro::ast_json::json_to_ast(&v) {
                Some(a) => a,
                None => { eprintln!("[macro-proxy] child JSON did not map to AST. stderr=\n{}", err); if strict_enabled() { std::process::exit(2); } ast.clone() }
            },
            Err(e) => {
                let code = status_opt.and_then(|s| s.code()).unwrap_or(-1);
                eprintln!("[macro-proxy] invalid JSON from child (code={}): {}\n-- child stderr --\n{}\n-- end stderr --", code, e, err);
                if strict_enabled() { std::process::exit(2); }
                ast.clone()
            }
        }
    }
}

fn strict_enabled() -> bool {
    match std::env::var("NYASH_MACRO_STRICT").ok() {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "off")
        }
        None => true,
    }
}
