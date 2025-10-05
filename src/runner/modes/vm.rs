use super::super::NyashRunner;
use nyash_rust::{
    ast::ASTNode,
    backend::VM,
    core::model::BoxDeclaration as CoreBoxDecl,
    mir::MirCompiler,
    parser::NyashParser,
    runtime::{NyashRuntime, NyashRuntimeBuilder},
};
use std::io::Write;
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
                // If explicit plugin config or direct lib is provided, defer to runner_plugin_init only
                let has_override = std::env::var("NYASH_PLUGIN_CONFIG").ok().map(|v| !v.trim().is_empty()).unwrap_or(false)
                    || std::env::var("NYASH_PLUGIN_DIRECT_LIB").is_ok();
                if !has_override {
                    let _ = nyash_rust::runtime::init_global_plugin_host("nyash.toml");
                }
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

        // Using handling: collect and optionally merge AST preludes (dev profiles default ON)
        let mut code_ref: String = code.clone();
        let mut prelude_asts: Vec<nyash_rust::ast::ASTNode> = Vec::new();
        // Alias symbols collected from `using ... as Alias` (always collected regardless of AST merge)
        let mut alias_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let use_ast = crate::config::env::using_ast_enabled();
        if crate::config::env::enable_using() && !quiet_pipe {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code, filename) {
                Ok((clean, paths, alias_pairs)) => {
                    code_ref = clean;
                    // Always record alias names for later desugaring of `Alias.X` in main AST.
                    for (alias, _canon) in alias_pairs.iter() { alias_names.insert(alias.clone()); }
                    crate::runner::modes::common_util::resolve::register_aliases_in_modules_registry(&alias_pairs);
                    if crate::config::env::resolve_trace() {
                        if !alias_pairs.is_empty() {
                            eprintln!("[using/alias] collected: {:?}", alias_pairs.iter().map(|(a, _)| a.clone()).collect::<Vec<_>>());
                        }
                    }
                    if !paths.is_empty() && !use_ast {
                        // In quiet JSON-only child pipelines, allow skipping AST prelude merge without error.
                        if !quiet_pipe {
                            eprintln!("❌ Pipeline error: `using` resolution error: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                            process::exit(1);
                        }
                        // quiet_pipe: proceed without AST merge (modules/aliases still recorded above)
                    }
                    if use_ast && !paths.is_empty() {
                        match crate::runner::modes::common_util::resolve::parse_preludes_to_asts(self, &paths) {
                            Ok(v) => {
                                // Build alias map: canon_path -> alias
                                use std::collections::HashMap;
                                let mut alias_map: HashMap<String,String> = HashMap::new();
                                for (a,p) in alias_pairs { alias_map.insert(p.clone(), a.clone()); }
                                // Rename top symbols for preludes that were imported with alias (collision-guarded)
                                let mut used_prefixed: std::collections::HashSet<String> = std::collections::HashSet::new();
                                for (path, ast) in v.into_iter() {
                                    let canon = std::fs::canonicalize(&path).ok().map(|pb| pb.to_string_lossy().to_string()).unwrap_or(path.clone());
                                    if let Some(alias) = alias_map.get(&canon) {
                                        match crate::runner::modes::common_util::resolve::alias_tools::rename_with_collision_guard(&ast, alias, &mut used_prefixed, &canon) {
                                            Ok(renamed) => prelude_asts.push(renamed),
                                            Err(e) => { eprintln!("❌ using: {}", e); process::exit(1); }
                                        }
                                    } else {
                                        prelude_asts.push(ast);
                                    }
                                }
                            }
                            Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
                        }
                    }
                }
                Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
            }
        }

        // Pre-expand '@name[:T] = expr' sugar at line-head (same as common/llvm/pyvm paths)
        let code = crate::runner::modes::common_util::resolve::preexpand_at_local(&code_ref);
        // Common pre-lexical normalization (raw strings, numeric separators)
        let code = crate::runner::modes::common_util::prelex::prelex_normalize(&code);

        // Parse to AST
        let main_ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // Merge preludes + main when enabled
        let ast = if use_ast && !prelude_asts.is_empty() {
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(prelude_asts, &main_ast)
        } else { main_ast };
        // Optional trace: check presence of raw alias variables before desugar
        if crate::config::env::resolve_trace() && !alias_names.is_empty() {
            fn contains_alias_var(n: &nyash_rust::ast::ASTNode, aliases: &std::collections::HashSet<String>) -> bool {
                match n {
                    nyash_rust::ast::ASTNode::Variable { name, .. } => aliases.contains(name),
                    nyash_rust::ast::ASTNode::FieldAccess { object, .. } => contains_alias_var(object, aliases),
                    nyash_rust::ast::ASTNode::MethodCall { object, arguments, .. } => {
                        contains_alias_var(object, aliases) || arguments.iter().any(|a| contains_alias_var(a, aliases))
                    }
                    nyash_rust::ast::ASTNode::FunctionCall { arguments, .. } => arguments.iter().any(|a| contains_alias_var(a, aliases)),
                    nyash_rust::ast::ASTNode::Program { statements, .. } => statements.iter().any(|s| contains_alias_var(s, aliases)),
                    nyash_rust::ast::ASTNode::Assignment { target, value, .. } => contains_alias_var(target, aliases) || contains_alias_var(value, aliases),
                    _ => false,
                }
            }
            let has_alias_var = contains_alias_var(&ast, &alias_names);
            if has_alias_var { eprintln!("[using/alias] pre-desugar: alias variable present in AST"); }
        }
        // Alias desugar: transform `Alias.X` to `Alias_X` to match renamed preludes
        let ast = {
            crate::runner::modes::common_util::resolve::alias_tools::desugar_alias_field_access(&ast, &alias_names, true)
        };
        if crate::config::env::resolve_trace() && !alias_names.is_empty() {
            fn contains_alias_var(n: &nyash_rust::ast::ASTNode, aliases: &std::collections::HashSet<String>) -> bool {
                match n {
                    nyash_rust::ast::ASTNode::Variable { name, .. } => aliases.contains(name),
                    nyash_rust::ast::ASTNode::FieldAccess { object, .. } => contains_alias_var(object, aliases),
                    nyash_rust::ast::ASTNode::MethodCall { object, arguments, .. } => {
                        contains_alias_var(object, aliases) || arguments.iter().any(|a| contains_alias_var(a, aliases))
                    }
                    nyash_rust::ast::ASTNode::FunctionCall { arguments, .. } => arguments.iter().any(|a| contains_alias_var(a, aliases)),
                    nyash_rust::ast::ASTNode::Program { statements, .. } => statements.iter().any(|s| contains_alias_var(s, aliases)),
                    nyash_rust::ast::ASTNode::Assignment { target, value, .. } => contains_alias_var(target, aliases) || contains_alias_var(value, aliases),
                    _ => false,
                }
            }
            let has_alias_var = contains_alias_var(&ast, &alias_names);
            if has_alias_var { eprintln!("[using/alias] post-desugar: alias variable still present in AST"); }
        }
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

        // Prepare runtime and collect Box declarations for VM user-defined types
        let runtime = {
            let mut builder = NyashRuntimeBuilder::new();
            if std::env::var("NYASH_GC_COUNTING").ok().as_deref() == Some("1") {
                builder = builder.with_counting_gc();
            }
            let rt = builder.build();
            self.collect_box_declarations(&ast, &rt);
            // Register UserDefinedBoxFactory backed by the same declarations (when available)
            #[cfg(feature = "interpreter-legacy")]
            {
                use nyash_rust::box_factory::SharedState;
                use nyash_rust::box_factory::user_defined::UserDefinedBoxFactory;
                let mut shared = SharedState::new();
                shared.box_declarations = rt.box_declarations.clone();
                let udf = Arc::new(UserDefinedBoxFactory::new(shared));
                if let Ok(mut reg) = rt.box_registry.lock() {
                    reg.register(udf);
                }
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
            #[cfg(feature = "pyvm-bridge")]
            {
                match super::common_util::pyvm::run_pyvm_harness_lib(&module_vm, "vm") {
                    Ok(code) => { process::exit(code); }
                    Err(e) => {
                        if std::env::var("NYASH_VM_REQUIRE_PY").ok().as_deref() == Some("1") {
                            eprintln!("❌ PyVM error: {}", e);
                            process::exit(1);
                        } else {
                            eprintln!("[vm] PyVM unavailable ({}). Falling back to Rust VM…", e);
                        }
                    }
                }
            }
            #[cfg(not(feature = "pyvm-bridge"))]
            {
                if std::env::var("NYASH_VM_REQUIRE_PY").ok().as_deref() == Some("1") {
                    eprintln!("❌ PyVM bridge disabled at build (feature pyvm-bridge is off)");
                    process::exit(1);
                } else {
                    eprintln!("[vm] PyVM bridge disabled (feature off). Using Rust VM.");
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
                    // Flush stdout before exit to ensure output is visible
                    let _ = std::io::stdout().flush();
                }
                // Unify exit behavior across backends: map return value to process exit code.
                // - Integer/Bool → exit code (masked to 0..255)
                // - Others → 0
                let code_i64 = nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref()).unwrap_or(0);
                let code = (code_i64 as i32) & 0xFF;
                process::exit(code);
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
