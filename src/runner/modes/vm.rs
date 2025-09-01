use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::MirCompiler, backend::VM, runtime::{NyashRuntime, NyashRuntimeBuilder}, ast::ASTNode, core::model::BoxDeclaration as CoreBoxDecl, interpreter::SharedState, box_factory::user_defined::UserDefinedBoxFactory};
use std::{fs, process};
use std::sync::Arc;

impl NyashRunner {
    /// Execute VM mode (split)
    pub(crate) fn execute_vm_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        // Prepare runtime and collect Box declarations for VM user-defined types
        let runtime = {
            let mut builder = NyashRuntimeBuilder::new();
            if std::env::var("NYASH_GC_COUNTING").ok().as_deref() == Some("1") {
                builder = builder.with_counting_gc();
            }
            let rt = builder.build();
            self.collect_box_declarations(&ast, &rt);
            // Register UserDefinedBoxFactory backed by the same declarations
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() { reg.register(udf); }
            rt
        };

        // Compile to MIR (opt passes configurable)
        let mut mir_compiler = MirCompiler::with_options(!self.config.no_optimize);
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };

        // Optional: demo scheduling hook
        if std::env::var("NYASH_SCHED_DEMO").ok().as_deref() == Some("1") {
            if let Some(s) = &runtime.scheduler {
                // Immediate task
                s.spawn("demo-immediate", Box::new(|| {
                    println!("[SCHED] immediate task ran at safepoint");
                }));
                // Delayed task
                s.spawn_after(0, "demo-delayed", Box::new(|| {
                    println!("[SCHED] delayed task ran at safepoint");
                }));
            }
        }

        // Optional: VM-only escape analysis to elide barriers before execution
        let mut module_vm = compile_result.module.clone();
        if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
            let removed = nyash_rust::mir::passes::escape::escape_elide_barriers_vm(&mut module_vm);
            if removed > 0 && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[VM] escape_elide_barriers: removed {} barriers", removed);
            }
        }

        // Expose GC/scheduler hooks globally for JIT externs (checkpoint/await, etc.)
        nyash_rust::runtime::global_hooks::set_from_runtime(&runtime);

        // Execute with VM using prepared runtime
        let mut vm = VM::with_runtime(runtime);
        match vm.execute_module(&module_vm) {
            Ok(result) => {
                println!("✅ VM execution completed successfully!");
                // Pretty-print using MIR return type when available to avoid Void-looking floats/bools
                if let Some(func) = compile_result.module.functions.get("main") {
                    use nyash_rust::mir::MirType;
                    use nyash_rust::box_trait::{NyashBox, IntegerBox, BoolBox, StringBox};
                    use nyash_rust::boxes::FloatBox;
                    let (ety, sval) = match &func.signature.return_type {
                        MirType::Float => {
                            if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                                ("Float", format!("{}", fb.value))
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Float", format!("{}", ib.value as f64))
                            } else { ("Float", result.to_string_box().value) }
                        }
                        MirType::Integer => {
                            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Integer", ib.value.to_string())
                            } else { ("Integer", result.to_string_box().value) }
                        }
                        MirType::Bool => {
                            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                                ("Bool", bb.value.to_string())
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Bool", (ib.value != 0).to_string())
                            } else { ("Bool", result.to_string_box().value) }
                        }
                        MirType::String => {
                            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                                ("String", sb.value.clone())
                            } else { ("String", result.to_string_box().value) }
                        }
                        _ => { (result.type_name(), result.to_string_box().value) }
                    };
                    println!("ResultType(MIR): {}", ety);
                    println!("Result: {}", sval);
                } else {
                    println!("Result: {:?}", result);
                }
            },
            Err(e) => { eprintln!("❌ VM execution error: {}", e); process::exit(1); }
        }
    }

    /// Collect Box declarations from AST and register into runtime
    pub(crate) fn collect_box_declarations(&self, ast: &ASTNode, runtime: &NyashRuntime) {
        fn resolve_include_path(filename: &str) -> String {
            if filename.starts_with("./") || filename.starts_with("../") { return filename.to_string(); }
            let parts: Vec<&str> = filename.splitn(2, '/').collect();
            if parts.len() == 2 {
                let root = parts[0]; let rest = parts[1];
                let cfg_path = "nyash.toml";
                if let Ok(toml_str) = std::fs::read_to_string(cfg_path) {
                    if let Ok(toml_val) = toml::from_str::<toml::Value>(&toml_str) {
                        if let Some(include) = toml_val.get("include") {
                            if let Some(roots) = include.get("roots").and_then(|v| v.as_table()) {
                                if let Some(base) = roots.get(root).and_then(|v| v.as_str()) {
                                    let mut b = base.to_string(); if !b.ends_with('/') && !b.ends_with('\\') { b.push('/'); }
                                    return format!("{}{}", b, rest);
                                }
                            }
                        }
                    }
                }
            }
            format!("./{}", filename)
        }

        use std::collections::{HashSet, VecDeque};

        fn walk_with_state(node: &ASTNode, runtime: &NyashRuntime, stack: &mut Vec<String>, visited: &mut HashSet<String>) {
            match node {
                ASTNode::Program { statements, .. } => { for st in statements { walk_with_state(st, runtime, stack, visited); } }
                ASTNode::FunctionDeclaration { body, .. } => { for st in body { walk_with_state(st, runtime, stack, visited); } }
                ASTNode::Include { filename, .. } => {
                    let mut path = resolve_include_path(filename);
                    if std::path::Path::new(&path).is_dir() {
                        path = format!("{}/index.nyash", path.trim_end_matches('/'));
                    } else if std::path::Path::new(&path).extension().is_none() {
                        path.push_str(".nyash");
                    }
                    // Cycle detection using stack
                    if let Some(pos) = stack.iter().position(|p| p == &path) {
                        let mut chain = stack[pos..].to_vec();
                        chain.push(path.clone());
                        eprintln!("include cycle detected (collector): {}", chain.join(" -> "));
                        return; // Skip to avoid infinite recursion
                    }
                    if visited.contains(&path) {
                        return; // Already processed
                    }
                    stack.push(path.clone());
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(inc_ast) = NyashParser::parse_from_string(&content) {
                            walk_with_state(&inc_ast, runtime, stack, visited);
                            visited.insert(path);
                        }
                    }
                    stack.pop();
                }
                ASTNode::Assignment { target, value, .. } => {
                    walk_with_state(target, runtime, stack, visited); walk_with_state(value, runtime, stack, visited);
                }
                ASTNode::Return { value, .. } => { if let Some(v) = value { walk_with_state(v, runtime, stack, visited); } }
                ASTNode::Print { expression, .. } => { walk_with_state(expression, runtime, stack, visited); }
                ASTNode::If { condition, then_body, else_body, .. } => {
                    walk_with_state(condition, runtime, stack, visited);
                    for st in then_body { walk_with_state(st, runtime, stack, visited); }
                    if let Some(eb) = else_body { for st in eb { walk_with_state(st, runtime, stack, visited); } }
                }
                ASTNode::Loop { condition, body, .. } => {
                    walk_with_state(condition, runtime, stack, visited); for st in body { walk_with_state(st, runtime, stack, visited); }
                }
                ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                    for st in try_body { walk_with_state(st, runtime, stack, visited); }
                    for cc in catch_clauses { for st in &cc.body { walk_with_state(st, runtime, stack, visited); } }
                    if let Some(fb) = finally_body { for st in fb { walk_with_state(st, runtime, stack, visited); } }
                }
                ASTNode::Throw { expression, .. } => { walk_with_state(expression, runtime, stack, visited); }
                ASTNode::Local { initial_values, .. } => {
                    for iv in initial_values { if let Some(v) = iv { walk_with_state(v, runtime, stack, visited); } }
                }
                ASTNode::Outbox { initial_values, .. } => {
                    for iv in initial_values { if let Some(v) = iv { walk_with_state(v, runtime, stack, visited); } }
                }
                ASTNode::FunctionCall { arguments, .. } => { for a in arguments { walk_with_state(a, runtime, stack, visited); } }
                ASTNode::MethodCall { object, arguments, .. } => { walk_with_state(object, runtime, stack, visited); for a in arguments { walk_with_state(a, runtime, stack, visited); } }
                ASTNode::FieldAccess { object, .. } => { walk_with_state(object, runtime, stack, visited); }
                ASTNode::New { arguments, .. } => { for a in arguments { walk_with_state(a, runtime, stack, visited); } }
                ASTNode::BinaryOp { left, right, .. } => { walk_with_state(left, runtime, stack, visited); walk_with_state(right, runtime, stack, visited); }
                ASTNode::UnaryOp { operand, .. } => { walk_with_state(operand, runtime, stack, visited); }
                ASTNode::AwaitExpression { expression, .. } => { walk_with_state(expression, runtime, stack, visited); }
                ASTNode::Arrow { sender, receiver, .. } => { walk_with_state(sender, runtime, stack, visited); walk_with_state(receiver, runtime, stack, visited); }
                ASTNode::Nowait { expression, .. } => { walk_with_state(expression, runtime, stack, visited); }
                ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    for (_mname, mnode) in methods { walk_with_state(mnode, runtime, stack, visited); }
                    for (_ckey, cnode) in constructors { walk_with_state(cnode, runtime, stack, visited); }
                    let decl = CoreBoxDecl {
                        name: name.clone(),
                        fields: fields.clone(),
                        public_fields: public_fields.clone(),
                        private_fields: private_fields.clone(),
                        methods: methods.clone(),
                        constructors: constructors.clone(),
                        init_fields: init_fields.clone(),
                        weak_fields: weak_fields.clone(),
                        is_interface: *is_interface,
                        extends: extends.clone(),
                        implements: implements.clone(),
                        type_parameters: type_parameters.clone(),
                    };
                    if let Ok(mut map) = runtime.box_declarations.write() { map.insert(name.clone(), decl); }
                }
                _ => {}
            }
        }
        let mut stack: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        walk_with_state(ast, runtime, &mut stack, &mut visited);
    }
}
