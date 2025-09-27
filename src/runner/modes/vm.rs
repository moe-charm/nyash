use super::super::NyashRunner;
use nyash_rust::{
    ast::ASTNode,
    backend::VM,
    box_factory::user_defined::UserDefinedBoxFactory,
    core::model::BoxDeclaration as CoreBoxDecl,
    box_factory::SharedState,
    mir::MirCompiler,
    parser::NyashParser,
    runtime::{NyashRuntime, NyashRuntimeBuilder},
};
use std::sync::Arc;
use std::{fs, process};

impl NyashRunner {
    /// Execute VM mode (split)
    pub(crate) fn execute_vm_mode(&self, filename: &str) {
        // Quiet mode for child pipelines (e.g., selfhost compiler JSON emit)
        let quiet_pipe = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1");
        // Enforce plugin-first policy for VM on this branch (deterministic):
        // - Initialize plugin host if not yet loaded
        // - Prefer plugin implementations for core boxes
        // - Optionally fail fast when plugins are missing (NYASH_VM_PLUGIN_STRICT=1)
        {
            // Initialize unified registry globals (idempotent)
            nyash_rust::runtime::init_global_unified_registry();
            // Init plugin host from nyash.toml if not yet loaded
            let need_init = {
                let host = nyash_rust::runtime::get_global_plugin_host();
                host.read()
                    .map(|h| h.config_ref().is_none())
                    .unwrap_or(true)
            };
            if need_init {
                let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
                crate::runner_plugin_init::init_bid_plugins();
            }
            // Prefer plugin-builtins for core types unless explicitly disabled
            if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().is_none() {
                std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
            }
            // Build stable override list
            let mut override_types: Vec<String> =
                if let Ok(list) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
                    list.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    vec![]
                };
            for t in [
                "FileBox",
                "TOMLBox", // IO/config
                "ConsoleBox",
                "StringBox",
                "IntegerBox", // core value-ish
                "ArrayBox",
                "MapBox", // collections
                "MathBox",
                "TimeBox", // math/time helpers
            ] {
                if !override_types.iter().any(|x| x == t) {
                    override_types.push(t.to_string());
                }
            }
            std::env::set_var("NYASH_PLUGIN_OVERRIDE_TYPES", override_types.join(","));

            // Strict mode: verify providers exist for override types
            if std::env::var("NYASH_VM_PLUGIN_STRICT").ok().as_deref() == Some("1") {
                let v2 = nyash_rust::runtime::get_global_registry();
                let mut missing: Vec<String> = Vec::new();
                for t in [
                    "FileBox",
                    "ConsoleBox",
                    "ArrayBox",
                    "MapBox",
                    "StringBox",
                    "IntegerBox",
                ] {
                    if v2.get_provider(t).is_none() {
                        missing.push(t.to_string());
                    }
                }
                if !missing.is_empty() {
                    eprintln!(
                        "❌ VM plugin-first strict: missing providers for: {:?}",
                        missing
                    );
                    std::process::exit(1);
                }
            }
        }

        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Using handling: AST-prelude collection (legacy inlining removed)
        let code = if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code, filename) {
                Ok((clean, paths)) => {
                    if !paths.is_empty() && !crate::config::env::using_ast_enabled() {
                        eprintln!("❌ using: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                        process::exit(1);
                    }
                    // VM path currently does not merge prelude ASTs; rely on compile pipeline path for that.
                    clean
                }
                Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
            }
        } else { code };

        // Pre-expand '@name[:T] = expr' sugar at line-head (same as common/llvm/pyvm paths)
        let code = crate::runner::modes::common_util::resolve::preexpand_at_local(&code);

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

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
            if let Ok(mut reg) = rt.box_registry.lock() {
                reg.register(udf);
            }
            rt
        };

        // Compile to MIR (opt passes configurable)
        let mut mir_compiler = MirCompiler::with_options(!self.config.no_optimize);
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Optional: demo scheduling hook
        if std::env::var("NYASH_SCHED_DEMO").ok().as_deref() == Some("1") {
            if let Some(s) = &runtime.scheduler {
                // Immediate task
                s.spawn(
                    "demo-immediate",
                    Box::new(|| {
                        println!("[SCHED] immediate task ran at safepoint");
                    }),
                );
                // Delayed task
                s.spawn_after(
                    0,
                    "demo-delayed",
                    Box::new(|| {
                        println!("[SCHED] delayed task ran at safepoint");
                    }),
                );
            }
        }

        // Optional: dump MIR for diagnostics
        if std::env::var("NYASH_VM_DUMP_MIR").ok().as_deref() == Some("1") {
            let p = nyash_rust::mir::MirPrinter::new();
            eprintln!("{}", p.print_module(&compile_result.module));
        }

        // Optional: VM-only escape analysis to elide barriers before execution
        let mut module_vm = compile_result.module.clone();
        if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
            let removed = nyash_rust::mir::passes::escape::escape_elide_barriers_vm(&mut module_vm);
            if removed > 0 { crate::cli_v!("[VM] escape_elide_barriers: removed {} barriers", removed); }
        }

        // Optional: PyVM path. When NYASH_VM_USE_PY=1, emit MIR(JSON) and delegate execution to tools/pyvm_runner.py
        // Safety valve: if runner is not found or fails to launch, gracefully fall back to Rust VM
        if std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
            match super::common_util::pyvm::run_pyvm_harness_lib(&module_vm, "vm") {
                Ok(code) => { process::exit(code); }
                Err(e) => {
                    // Fallback unless explicitly required
                    if std::env::var("NYASH_VM_REQUIRE_PY").ok().as_deref() == Some("1") {
                        eprintln!("❌ PyVM error: {}", e);
                        process::exit(1);
                    } else {
                        eprintln!("[vm] PyVM unavailable ({}). Falling back to Rust VM…", e);
                    }
                }
            }
        }

        // Expose GC/scheduler hooks globally for JIT externs (checkpoint/await, etc.)
        nyash_rust::runtime::global_hooks::set_from_runtime(&runtime);

        // Execute with VM using prepared runtime
        let mut vm = VM::with_runtime(runtime);
        match vm.execute_module(&module_vm) {
            Ok(result) => {
                if !quiet_pipe {
                    println!("✅ VM execution completed successfully!");
                }
                // Pretty-print with coercions for plugin-backed values
                // Prefer MIR signature when available, but fall back to runtime coercions to keep VM/JIT consistent.
                let (ety, sval) = if let Some(func) = compile_result.module.functions.get("main") {
                    use nyash_rust::box_trait::{BoolBox, IntegerBox, StringBox};
                    use nyash_rust::boxes::FloatBox;
                    use nyash_rust::mir::MirType;
                    match &func.signature.return_type {
                        MirType::Float => {
                            if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                                ("Float", format!("{}", fb.value))
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Float", format!("{}", ib.value as f64))
                            } else if let Some(s) =
                                nyash_rust::runtime::semantics::coerce_to_string(result.as_ref())
                            {
                                ("String", s)
                            } else {
                                (result.type_name(), result.to_string_box().value)
                            }
                        }
                        MirType::Integer => {
                            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Integer", ib.value.to_string())
                            } else if let Some(i) =
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            {
                                ("Integer", i.to_string())
                            } else {
                                (result.type_name(), result.to_string_box().value)
                            }
                        }
                        MirType::Bool => {
                            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                                ("Bool", bb.value.to_string())
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Bool", (ib.value != 0).to_string())
                            } else {
                                (result.type_name(), result.to_string_box().value)
                            }
                        }
                        MirType::String => {
                            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                                ("String", sb.value.clone())
                            } else if let Some(s) =
                                nyash_rust::runtime::semantics::coerce_to_string(result.as_ref())
                            {
                                ("String", s)
                            } else {
                                (result.type_name(), result.to_string_box().value)
                            }
                        }
                        _ => {
                            if let Some(i) =
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            {
                                ("Integer", i.to_string())
                            } else if let Some(s) =
                                nyash_rust::runtime::semantics::coerce_to_string(result.as_ref())
                            {
                                ("String", s)
                            } else {
                                (result.type_name(), result.to_string_box().value)
                            }
                        }
                    }
                } else {
                    if let Some(i) = nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                    {
                        ("Integer", i.to_string())
                    } else if let Some(s) =
                        nyash_rust::runtime::semantics::coerce_to_string(result.as_ref())
                    {
                        ("String", s)
                    } else {
                        (result.type_name(), result.to_string_box().value)
                    }
                };
                if !quiet_pipe {
                    println!("ResultType(MIR): {}", ety);
                    println!("Result: {}", sval);
                }
            }
            Err(e) => {
                eprintln!("❌ VM execution error: {}", e);
                process::exit(1);
            }
        }
    }

    /// Collect Box declarations from AST and register into runtime
    pub(crate) fn collect_box_declarations(&self, ast: &ASTNode, runtime: &NyashRuntime) {
        // include support removed; using is resolved by runner/strip

        use std::collections::HashSet;

        fn walk_with_state(
            node: &ASTNode,
            runtime: &NyashRuntime,
            stack: &mut Vec<String>,
            visited: &mut HashSet<String>,
        ) {
            match node {
                ASTNode::Program { statements, .. } => {
                    for st in statements {
                        walk_with_state(st, runtime, stack, visited);
                    }
                }
                ASTNode::FunctionDeclaration { body, .. } => {
                    for st in body {
                        walk_with_state(st, runtime, stack, visited);
                    }
                }
                
                ASTNode::Assignment { target, value, .. } => {
                    walk_with_state(target, runtime, stack, visited);
                    walk_with_state(value, runtime, stack, visited);
                }
                ASTNode::Return { value, .. } => {
                    if let Some(v) = value {
                        walk_with_state(v, runtime, stack, visited);
                    }
                }
                ASTNode::Print { expression, .. } => {
                    walk_with_state(expression, runtime, stack, visited);
                }
                ASTNode::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    walk_with_state(condition, runtime, stack, visited);
                    for st in then_body {
                        walk_with_state(st, runtime, stack, visited);
                    }
                    if let Some(eb) = else_body {
                        for st in eb {
                            walk_with_state(st, runtime, stack, visited);
                        }
                    }
                }
                ASTNode::Loop {
                    condition, body, ..
                } => {
                    walk_with_state(condition, runtime, stack, visited);
                    for st in body {
                        walk_with_state(st, runtime, stack, visited);
                    }
                }
                ASTNode::TryCatch {
                    try_body,
                    catch_clauses,
                    finally_body,
                    ..
                } => {
                    for st in try_body {
                        walk_with_state(st, runtime, stack, visited);
                    }
                    for cc in catch_clauses {
                        for st in &cc.body {
                            walk_with_state(st, runtime, stack, visited);
                        }
                    }
                    if let Some(fb) = finally_body {
                        for st in fb {
                            walk_with_state(st, runtime, stack, visited);
                        }
                    }
                }
                ASTNode::Throw { expression, .. } => {
                    walk_with_state(expression, runtime, stack, visited);
                }
                ASTNode::Local { initial_values, .. } => {
                    for iv in initial_values {
                        if let Some(v) = iv {
                            walk_with_state(v, runtime, stack, visited);
                        }
                    }
                }
                ASTNode::Outbox { initial_values, .. } => {
                    for iv in initial_values {
                        if let Some(v) = iv {
                            walk_with_state(v, runtime, stack, visited);
                        }
                    }
                }
                ASTNode::FunctionCall { arguments, .. } => {
                    for a in arguments {
                        walk_with_state(a, runtime, stack, visited);
                    }
                }
                ASTNode::MethodCall {
                    object, arguments, ..
                } => {
                    walk_with_state(object, runtime, stack, visited);
                    for a in arguments {
                        walk_with_state(a, runtime, stack, visited);
                    }
                }
                ASTNode::FieldAccess { object, .. } => {
                    walk_with_state(object, runtime, stack, visited);
                }
                ASTNode::New { arguments, .. } => {
                    for a in arguments {
                        walk_with_state(a, runtime, stack, visited);
                    }
                }
                ASTNode::BinaryOp { left, right, .. } => {
                    walk_with_state(left, runtime, stack, visited);
                    walk_with_state(right, runtime, stack, visited);
                }
                ASTNode::UnaryOp { operand, .. } => {
                    walk_with_state(operand, runtime, stack, visited);
                }
                ASTNode::AwaitExpression { expression, .. } => {
                    walk_with_state(expression, runtime, stack, visited);
                }
                ASTNode::Arrow {
                    sender, receiver, ..
                } => {
                    walk_with_state(sender, runtime, stack, visited);
                    walk_with_state(receiver, runtime, stack, visited);
                }
                ASTNode::Nowait { expression, .. } => {
                    walk_with_state(expression, runtime, stack, visited);
                }
                ASTNode::BoxDeclaration {
                    name,
                    fields,
                    public_fields,
                    private_fields,
                    methods,
                    constructors,
                    init_fields,
                    weak_fields,
                    is_interface,
                    extends,
                    implements,
                    type_parameters,
                    ..
                } => {
                    for (_mname, mnode) in methods {
                        walk_with_state(mnode, runtime, stack, visited);
                    }
                    for (_ckey, cnode) in constructors {
                        walk_with_state(cnode, runtime, stack, visited);
                    }
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
                    if let Ok(mut map) = runtime.box_declarations.write() {
                        map.insert(name.clone(), decl);
                    }
                }
                _ => {}
            }
        }
        let mut stack: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        walk_with_state(ast, runtime, &mut stack, &mut visited);
    }
}
