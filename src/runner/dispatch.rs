/*!
 * Runner dispatch helpers — execute MIR module and report result
 */

use super::*;
use crate::runner::json_v0_bridge;
use nyash_rust::parser::NyashParser;
use std::{fs, process};

/// Thin file dispatcher: select backend and delegate to mode executors
pub(crate) fn execute_file_with_backend(runner: &NyashRunner, filename: &str) {
    // Selfhost pipeline (Ny -> JSON v0) behind env gate
    if std::env::var("NYASH_USE_NY_COMPILER").ok().as_deref() == Some("1") {
        if runner.try_run_selfhost_pipeline(filename) {
            return;
        } else {
            crate::cli_v!("[ny-compiler] fallback to default path (MVP unavailable for this input)");
        }
    }

    // Direct v0 bridge when requested via CLI/env
    let groups = runner.config.as_groups();
    let use_ny_parser = groups.parser.parser_ny
        || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
    if use_ny_parser {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        match json_v0_bridge::parse_source_v0_to_module(&code) {
            Ok(module) => {
                crate::cli_v!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                runner.execute_mir_module(&module);
                return;
            }
            Err(e) => {
                eprintln!("❌ Direct bridge parse error: {}", e);
                process::exit(1);
            }
        }
    }

    // AST dump mode
    if groups.debug.dump_ast {
        println!("🧠 Nyash AST Dump - Processing file: {}", filename);
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // Optional macro expansion dump (no-op expansion for now)
        let ast2 = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, true);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else {
            ast.clone()
        };
        println!("{:#?}", ast2);
        return;
    }

    // Dump expanded AST as JSON v0 and exit
    if runner.config.dump_expanded_ast_json {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        let expanded = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, false);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else { ast };
        let j = crate::r#macro::ast_json::ast_to_json(&expanded);
        println!("{}", j.to_string());
        return;
    }

    // MIR dump/verify
    if groups.debug.dump_mir || groups.debug.verify_mir {
        crate::cli_v!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
        runner.execute_mir_mode(filename);
        return;
    }

    // WASM / AOT (feature-gated)
    if groups.compile_wasm {
        #[cfg(feature = "wasm-backend")]
        {
            super::modes::wasm::execute_wasm_mode(runner, filename);
            return;
        }
        #[cfg(not(feature = "wasm-backend"))]
        {
            eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend");
            process::exit(1);
        }
    }
    if groups.compile_native {
        #[cfg(feature = "cranelift-jit")]
        {
            runner.execute_aot_mode(filename);
            return;
        }
        #[cfg(not(feature = "cranelift-jit"))]
        {
            eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit");
            process::exit(1);
        }
    }

    // Backend selection
    match groups.backend.backend.as_str() {
        "mir" => {
            crate::cli_v!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
            runner.execute_mir_mode(filename);
        }
        "vm" => {
            crate::cli_v!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            // Unified VM engine (Phase‑A): default=fallback
            runner.execute_vm_engine(filename);
        }
        #[cfg(feature = "cranelift-jit")]
        "jit-direct" => {
            crate::cli_v!("⚡ Nyash JIT-Direct Backend - Executing file: {} ⚡", filename);
            #[cfg(feature = "cranelift-jit")]
            {
                // Use independent JIT-direct runner method (no VM execute loop)
                runner.run_file_jit_direct(filename);
            }
            #[cfg(not(feature = "cranelift-jit"))]
            {
                eprintln!("❌ Cranelift backend not available. Please rebuild with: cargo build --features cranelift-jit");
                process::exit(1);
            }
        }
        "llvm" => {
            crate::cli_v!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
            runner.execute_llvm_mode(filename);
        }
        other => {
            eprintln!("❌ Unknown backend: {}. Use 'vm' or 'llvm'.", other);
            std::process::exit(2);
        }
    }
}

impl NyashRunner {
    /// Compile Nyash file to MIR and execute via selected VM engine (unified entry)
    pub(crate) fn execute_vm_engine(&self, filename: &str) {
        use nyash_rust::parser::NyashParser;
        use std::{fs, process};

        let code = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        // Using preprocessing with AST-prelude merge when enabled
        let use_ast = crate::config::env::using_ast_enabled();
        let mut code2 = code;
        let mut prelude_asts: Vec<nyash_rust::ast::ASTNode> = Vec::new();
        // Using + Alias (MVP): collect alias pairs, pre-parse preludes and rename their top symbols,
        // then desugar alias field/call access on the combined AST.
        let mut alias_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut alias_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code2, filename) {
                Ok((clean, paths, alias_pairs)) => {
                    code2 = clean;
                    for (alias, canon) in alias_pairs.iter() {
                        alias_names.insert(alias.clone());
                        alias_map.insert(canon.clone(), alias.clone());
                    }
                    if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                        if !alias_pairs.is_empty() {
                            eprintln!("[using/alias] collected: {:?}", alias_pairs);
                        }
                    }
                    if !paths.is_empty() && !use_ast {
                        eprintln!("❌ using: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                        process::exit(1);
                    }
                    if use_ast && !paths.is_empty() {
                        match crate::runner::modes::common_util::resolve::parse_preludes_to_asts(self, &paths) {
                            Ok(v) => {
                                // Apply alias rename to prelude top symbols when present (collision-guarded)
                                let mut used_prefixed: std::collections::HashSet<String> = std::collections::HashSet::new();
                                for (path, ast) in v.into_iter() {
                                    let canon = std::fs::canonicalize(&path)
                                        .ok()
                                        .map(|pb| pb.to_string_lossy().to_string())
                                        .unwrap_or(path.clone());
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
        // Dev sugar and pre-lex normalization
        code2 = crate::runner::modes::common_util::resolve::preexpand_at_local(&code2);
        code2 = crate::runner::modes::common_util::prelex::prelex_normalize(&code2);
        // Parse
        let main_ast = match NyashParser::parse_from_string(&code2) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        // Merge preludes
        let ast = if use_ast && !prelude_asts.is_empty() {
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(prelude_asts, &main_ast)
        } else { main_ast };
        // Alias desugar: transform `Alias.X` to `Alias_X` (and call forms)
        let ast = crate::runner::modes::common_util::resolve::alias_tools::desugar_alias_field_access(&ast, &alias_names, true);
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") && !alias_names.is_empty() {
            // Dev trace: check if any alias variable remains in AST after desugar
            fn contains_alias_var(n: &nyash_rust::ast::ASTNode, aliases: &std::collections::HashSet<String>) -> bool {
                use nyash_rust::ast::ASTNode as N;
                match n {
                    N::Variable { name, .. } => aliases.contains(name),
                    N::FieldAccess { object, .. } => contains_alias_var(object, aliases),
                    N::MethodCall { object, arguments, .. } => contains_alias_var(object, aliases) || arguments.iter().any(|a| contains_alias_var(a, aliases)),
                    N::FunctionCall { name, arguments, .. } => {
                        // If name still starts with any alias prefix, report
                        if aliases.iter().any(|a| name.starts_with(&format!("{}.", a))) { return true; }
                        arguments.iter().any(|a| contains_alias_var(a, aliases))
                    }
                    N::Program { statements, .. } => statements.iter().any(|s| contains_alias_var(s, aliases)),
                    N::Assignment { target, value, .. } => contains_alias_var(target, aliases) || contains_alias_var(value, aliases),
                    _ => false,
                }
            }
            if contains_alias_var(&ast, &alias_names) {
                eprintln!("[using/alias] post-desugar: alias variable still present in AST");
            } else {
                eprintln!("[using/alias] post-desugar: no alias variable remains");
            }
        }
        // Macro normalization (child-safe)
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
        // Register user-defined Boxes (inline factory) so NewBox for dev boxes works on fallback VM path
        {
            use nyash_rust::ast::ASTNode;
            use std::sync::{Arc, RwLock};
            // Collect non-static BoxDeclaration entries from AST (top-level only)
            let mut nonstatic_decls: std::collections::HashMap<String, nyash_rust::core::model::BoxDeclaration> =
                std::collections::HashMap::new();
            let mut static_names: Vec<String> = Vec::new();
            if let ASTNode::Program { statements, .. } = &ast {
                for st in statements {
                    if let ASTNode::BoxDeclaration {
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
                        is_static,
                        ..
                    } = st {
                        if *is_static {
                            static_names.push(name.clone());
                            continue;
                        }
                        let decl = nyash_rust::core::model::BoxDeclaration {
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
                        nonstatic_decls.insert(name.clone(), decl);
                    }
                }
            }
            // Alias: map StaticName -> StaticNameInstance when both exist
            let mut decls = nonstatic_decls.clone();
            for s in static_names.into_iter() {
                let inst = format!("{}Instance", s);
                if let Some(d) = nonstatic_decls.get(&inst) {
                    decls.insert(s, d.clone());
                }
            }
            if !decls.is_empty() {
                struct InlineUserBoxFactory {
                    decls: Arc<RwLock<std::collections::HashMap<String, nyash_rust::core::model::BoxDeclaration>>>,
                }
                impl nyash_rust::box_factory::BoxFactory for InlineUserBoxFactory {
                    fn create_box(
                        &self,
                        name: &str,
                        args: &[Box<dyn nyash_rust::box_trait::NyashBox>],
                    ) -> Result<Box<dyn nyash_rust::box_trait::NyashBox>, nyash_rust::box_factory::RuntimeError> {
                        let opt = { self.decls.read().unwrap().get(name).cloned() };
                        let decl = match opt {
                            Some(d) => d,
                            None => {
                                return Err(nyash_rust::box_factory::RuntimeError::InvalidOperation {
                                    message: format!("Unknown Box type: {}", name),
                                })
                            }
                        };
                        let mut inst = nyash_rust::instance_v2::InstanceBox::from_declaration(
                            decl.name.clone(),
                            decl.fields.clone(),
                            decl.methods.clone(),
                        );
                        let _ = inst.init(args);
                        Ok(Box::new(inst))
                    }
                    fn box_types(&self) -> Vec<&str> { vec![] }
                    fn is_available(&self) -> bool { true }
                    fn factory_type(&self) -> nyash_rust::box_factory::FactoryType { nyash_rust::box_factory::FactoryType::User }
                }
                let factory = InlineUserBoxFactory { decls: Arc::new(RwLock::new(decls)) };
                nyash_rust::runtime::unified_registry::register_user_defined_factory(Arc::new(factory));
            }
        }
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("2") {
            eprintln!("[ast-dump] {:?}", ast);
        }
        // Compile MIR
        let mut mir_compiler = nyash_rust::mir::MirCompiler::with_options(!self.config.no_optimize);
        let compile = match mir_compiler.compile(ast) {
            Ok(c) => c,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };
        // Execute via selected engine
        let mut engine = crate::runner::modes::super_iface::vm_engine_from_env();
        match engine.execute(&compile.module) {
            Ok(_code) => { /* program output handled inside VM; keep quiet */ }
            Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
        }
    }
    pub(crate) fn execute_mir_module(&self, module: &crate::mir::MirModule) {
        // If CLI requested MIR JSON emit, write to file and exit immediately.
        let groups = self.config.as_groups();
        if let Some(path) = groups.emit.emit_mir_json.as_ref() {
            let p = std::path::Path::new(path);
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, p) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            println!("MIR JSON written: {}", p.display());
            std::process::exit(0);
        }
        // If CLI requested EXE emit, generate JSON then invoke ny-llvmc to link NyRT and exit.
        if let Some(exe_out) = groups.emit.emit_exe.as_ref() {
            if let Err(e) = crate::runner::modes::common_util::exec::ny_llvmc_emit_exe_bin(
                module,
                exe_out,
                groups.emit.emit_exe_nyrt.as_deref(),
                groups.emit.emit_exe_libs.as_deref(),
            ) {
                eprintln!("❌ {}", e);
                std::process::exit(1);
            }
            println!("EXE written: {}", exe_out);
            std::process::exit(0);
        }
        use crate::backend::MirInterpreter;
        use crate::box_trait::{BoolBox, IntegerBox, StringBox};
        use crate::boxes::FloatBox;
        use crate::mir::MirType;

        let mut interp = MirInterpreter::new();
        match interp.execute_module(module) {
            Ok(result) => {
                println!("✅ MIR interpreter execution completed!");
                if let Some(func) = module.functions.get("main") {
                    let (ety, sval) = match &func.signature.return_type {
                        MirType::Float => {
                            if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                                ("Float", format!("{}", fb.value))
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Float", format!("{}", ib.value as f64))
                            } else {
                                ("Float", result.to_string_box().value)
                            }
                        }
                        MirType::Integer => {
                            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Integer", ib.value.to_string())
                            } else {
                                ("Integer", result.to_string_box().value)
                            }
                        }
                        MirType::Bool => {
                            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                                ("Bool", bb.value.to_string())
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Bool", (ib.value != 0).to_string())
                            } else {
                                ("Bool", result.to_string_box().value)
                            }
                        }
                        MirType::String => {
                            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                                ("String", sb.value.clone())
                            } else {
                                ("String", result.to_string_box().value)
                            }
                        }
                        _ => (result.type_name(), result.to_string_box().value),
                    };
                    println!("ResultType(MIR): {}", ety);
                    println!("Result: {}", sval);
                } else {
                    println!("Result: {:?}", result);
                }
            }
            Err(e) => {
                eprintln!("❌ MIR interpreter error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
