use super::super::NyashRunner;
use crate::{
    backend::MirInterpreter,
    box_factory::{BoxFactory, RuntimeError},
    core::model::BoxDeclaration as CoreBoxDecl,
    instance_v2::InstanceBox,
    mir::MirCompiler,
    parser::NyashParser,
};
use std::sync::{Arc, RwLock};
use std::{fs, process};

impl NyashRunner {
    /// Lightweight VM fallback using the in-crate MIR interpreter.
    /// - Respects using preprocessing done earlier in the pipeline
    /// - Relies on global plugin host initialized by runner
    pub(crate) fn execute_vm_fallback_interpreter(&self, filename: &str) {
        // Read source
        let code = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        // Using preprocessing with AST-prelude merge (when NYASH_USING_AST=1)
        let mut code2 = code;
        let use_ast_prelude =
            crate::config::env::enable_using() && crate::config::env::using_ast_enabled();
        let mut prelude_asts: Vec<nyash_rust::ast::ASTNode> = Vec::new();
        let mut alias_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(
                self, &code2, filename,
            ) {
                Ok((clean, paths, alias_pairs)) => {
                    crate::runner::modes::common_util::resolve::register_aliases_in_modules_registry(&alias_pairs);
                    code2 = clean;
                    for (alias, _canon) in alias_pairs.iter() { alias_names.insert(alias.clone()); }
                    if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") {
                        if !alias_names.is_empty() {
                            eprintln!("[vm-fallback/alias] collected aliases: {:?}", alias_names.iter().cloned().collect::<Vec<_>>());
                        }
                    }
                    if !paths.is_empty() && !use_ast_prelude {
                        eprintln!("❌ Pipeline error: `using` resolution error: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                        process::exit(1);
                    }
                    if use_ast_prelude && !paths.is_empty() {
                        match crate::runner::modes::common_util::resolve::parse_preludes_to_asts(self, &paths) {
                            Ok(v) => {
                                // Apply alias rename to prelude top symbols when applicable (collision-guarded)
                                use std::collections::HashMap;
                                let mut alias_map: HashMap<String,String> = HashMap::new();
                                for (a,p) in alias_pairs { alias_map.insert(p.clone(), a.clone()); }
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
                Err(e) => {
                    eprintln!("❌ {}", e);
                    process::exit(1);
                }
            }
        }
        // Dev sugar pre-expand: @name = expr → local name = expr
        code2 = crate::runner::modes::common_util::resolve::preexpand_at_local(&code2);

        // Parse main code
        let main_ast = match NyashParser::parse_from_string(&code2) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // When using AST prelude mode, combine prelude ASTs + main AST into one Program before macro expansion
        let ast_combined = if use_ast_prelude && !prelude_asts.is_empty() {
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(prelude_asts, &main_ast)
        } else { main_ast };
        // Apply alias desugar on combined AST so `Alias.X` becomes `Alias_X` (matching prelude renames)
        let ast_combined = crate::runner::modes::common_util::resolve::alias_tools::desugar_alias_field_access(&ast_combined, &alias_names, true);
        if std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1") && !alias_names.is_empty() {
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
            if contains_alias_var(&ast_combined, &alias_names) {
                eprintln!("[vm-fallback/alias] post-desugar alias var still present");
            }
        }
        // Optional: dump AST statement kinds for quick diagnostics
        if std::env::var("NYASH_AST_DUMP").ok().as_deref() == Some("1") {
            use nyash_rust::ast::ASTNode;
            eprintln!("[ast] dump start (vm-fallback)");
            if let ASTNode::Program { statements, .. } = &ast_combined {
                for (i, st) in statements.iter().enumerate().take(50) {
                    let kind = match st {
                        ASTNode::BoxDeclaration {
                            is_static, name, ..
                        } => {
                            if *is_static {
                                format!("StaticBox({})", name)
                            } else {
                                format!("Box({})", name)
                            }
                        }
                        ASTNode::FunctionDeclaration { name, .. } => format!("FuncDecl({})", name),
                        ASTNode::FunctionCall { name, .. } => format!("FuncCall({})", name),
                        ASTNode::MethodCall { method, .. } => format!("MethodCall({})", method),
                        ASTNode::ScopeBox { .. } => "ScopeBox".to_string(),
                        ASTNode::ImportStatement { path, .. } => format!("Import({})", path),
                        ASTNode::UsingStatement { namespace_name, .. } => {
                            format!("Using({})", namespace_name)
                        }
                        _ => format!("{:?}", st),
                    };
                    eprintln!("[ast] {}: {}", i, kind);
                }
            }
            eprintln!("[ast] dump end");
        }
        let ast = crate::r#macro::maybe_expand_and_dump(&ast_combined, false);

        // Minimal user-defined Box support (Option A):
        // Collect BoxDeclaration entries from AST and register a lightweight
        // factory into the unified registry so `new UserBox()` works on the
        // VM fallback path as well.
        {
            use nyash_rust::ast::ASTNode;

            // Collect user-defined (non-static) box declarations at program level.
            // Additionally, record static box names so we can alias
            // `StaticBoxName` -> `StaticBoxNameInstance` when such a
            // concrete instance box exists (common pattern in libs).
            let mut nonstatic_decls: std::collections::HashMap<String, CoreBoxDecl> =
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
                            continue; // modules/static boxes are not user-instantiable directly
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
                        nonstatic_decls.insert(name.clone(), decl);
                    }
                }
            }
            // Build final map with optional aliases for StaticName -> StaticNameInstance
            let mut decls = nonstatic_decls.clone();
            for s in static_names.into_iter() {
                let inst = format!("{}Instance", s);
                if let Some(d) = nonstatic_decls.get(&inst) {
                    decls.insert(s, d.clone());
                }
            }

            if !decls.is_empty() {
                // Inline factory: minimal User factory backed by collected declarations
                struct InlineUserBoxFactory {
                    decls: Arc<RwLock<std::collections::HashMap<String, CoreBoxDecl>>>,
                }
                impl BoxFactory for InlineUserBoxFactory {
                    fn create_box(
                        &self,
                        name: &str,
                        args: &[Box<dyn crate::box_trait::NyashBox>],
                    ) -> Result<Box<dyn crate::box_trait::NyashBox>, RuntimeError>
                    {
                        let opt = { self.decls.read().unwrap().get(name).cloned() };
                        let decl = match opt {
                            Some(d) => d,
                            None => {
                                return Err(RuntimeError::InvalidOperation {
                                    message: format!("Unknown Box type: {}", name),
                                })
                            }
                        };
                        let mut inst = InstanceBox::from_declaration(
                            decl.name.clone(),
                            decl.fields.clone(),
                            decl.methods.clone(),
                        );
                        let _ = inst.init(args);
                        Ok(Box::new(inst))
                    }

                    fn box_types(&self) -> Vec<&str> {
                        vec![]
                    }

                    fn is_available(&self) -> bool {
                        true
                    }

                    fn factory_type(&self) -> crate::box_factory::FactoryType {
                        crate::box_factory::FactoryType::User
                    }
                }
                let factory = InlineUserBoxFactory {
                    decls: Arc::new(RwLock::new(decls)),
                };
                crate::runtime::unified_registry::register_user_defined_factory(Arc::new(factory));
            }
        }
        let mut compiler = MirCompiler::with_options(!self.config.no_optimize);
        let compile = match compiler.compile(ast) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        // Optional barrier-elision for parity with VM path
        let mut module_vm = compile.module.clone();
        if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
            let removed = crate::mir::passes::escape::escape_elide_barriers_vm(&mut module_vm);
            if removed > 0 {
                crate::cli_v!(
                    "[VM-fallback] escape_elide_barriers: removed {} barriers",
                    removed
                );
            }
        }

        // Optional: dump MIR for diagnostics (parity with vm path)
        if std::env::var("NYASH_VM_DUMP_MIR").ok().as_deref() == Some("1") {
            let p = crate::mir::MirPrinter::new();
            eprintln!("{}", p.print_module(&module_vm));
        }

        // Execute via MIR interpreter
        let mut vm = MirInterpreter::new();
        // Optional: verify MIR before execution (dev-only)
        if std::env::var("NYASH_VM_VERIFY_MIR").ok().as_deref() == Some("1") {
            let mut verifier = crate::mir::verification::MirVerifier::new();
            for (name, func) in module_vm.functions.iter() {
                if let Err(errors) = verifier.verify_function(func) {
                    if !errors.is_empty() {
                        eprintln!("[vm-verify] function: {}", name);
                        for er in errors { eprintln!("  • {}", er); }
                    }
                }
            }
        }
        if std::env::var("NYASH_DUMP_FUNCS").ok().as_deref() == Some("1") {
            eprintln!("[vm] functions available:");
            for k in module_vm.functions.keys() {
                eprintln!("  - {}", k);
            }
        }
        match vm.execute_module(&module_vm) {
            Ok(_ret) => { /* interpreter already prints via println/console in program */ }
            Err(e) => {
                eprintln!("❌ VM fallback error: {}", e);
                process::exit(1);
            }
        }
    }
}
