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
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        // Using preprocessing with AST-prelude merge (when NYASH_USING_AST=1)
        let mut code2 = code;
        let use_ast_prelude = crate::config::env::enable_using()
            && crate::config::env::using_ast_enabled();
        let mut prelude_asts: Vec<nyash_rust::ast::ASTNode> = Vec::new();
        if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code2, filename) {
                Ok((clean, paths)) => {
                    code2 = clean;
                    if !paths.is_empty() && !use_ast_prelude {
                        eprintln!("❌ using: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                        process::exit(1);
                    }
                    // Normalize initial prelude paths relative to filename or $NYASH_ROOT,
                    // then recursively process prelude files: strip their using-lines and parse cleaned ASTs
                    let mut visited = std::collections::HashSet::<String>::new();
                    let mut stack: Vec<String> = Vec::new();
                    for raw in paths {
                        let mut pb = std::path::PathBuf::from(&raw);
                        if pb.is_relative() {
                            if let Some(dir) = std::path::Path::new(filename).parent() {
                                let cand = dir.join(&pb);
                                if cand.exists() { pb = cand; }
                            }
                            if pb.is_relative() {
                                if let Ok(root) = std::env::var("NYASH_ROOT") {
                                    let cand = std::path::Path::new(&root).join(&pb);
                                    if cand.exists() { pb = cand; }
                                } else {
                                    if let Ok(exe) = std::env::current_exe() {
                                        if let Some(root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
                                            let cand = root.join(&pb);
                                            if cand.exists() { pb = cand; }
                                        }
                                    }
                                }
                            }
                        }
                        stack.push(pb.to_string_lossy().to_string());
                    }
                    while let Some(mut p) = stack.pop() {
                        if std::path::Path::new(&p).is_relative() {
                            if let Ok(root) = std::env::var("NYASH_ROOT") {
                                let cand = std::path::Path::new(&root).join(&p);
                                p = cand.to_string_lossy().to_string();
                            }
                        }
                        if !visited.insert(p.clone()) { continue; }
                        match std::fs::read_to_string(&p) {
                            Ok(src) => match crate::runner::modes::common_util::resolve::collect_using_and_strip(self, &src, &p) {
                                Ok((clean_src, nested)) => {
                                    for np in nested {
                                        let mut npp = std::path::PathBuf::from(&np);
                                        if npp.is_relative() {
                                            if let Some(dir) = std::path::Path::new(&p).parent() {
                                                let cand = dir.join(&npp);
                                                if cand.exists() { npp = cand; }
                                            }
                                            if npp.is_relative() {
                                                if let Ok(root) = std::env::var("NYASH_ROOT") {
                                                    let cand = std::path::Path::new(&root).join(&npp);
                                                    if cand.exists() { npp = cand; }
                                                } else {
                                                    if let Ok(exe) = std::env::current_exe() {
                                                        if let Some(root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
                                                            let cand = root.join(&npp);
                                                            if cand.exists() { npp = cand; }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        let nps = npp.to_string_lossy().to_string();
                                        if !visited.contains(&nps) { stack.push(nps); }
                                    }
                                    match NyashParser::parse_from_string(&clean_src) {
                                        Ok(ast) => prelude_asts.push(ast),
                                        Err(e) => { eprintln!("❌ Parse error in using prelude {}: {}", p, e); process::exit(1); }
                                    }
                                }
                                Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
                            },
                            Err(e) => { eprintln!("❌ Error reading using prelude {}: {}", p, e); process::exit(1); }
                        }
                    }
                }
                Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
            }
        }
        // Dev sugar pre-expand: @name = expr → local name = expr
        code2 = crate::runner::modes::common_util::resolve::preexpand_at_local(&code2);

        // Parse main code
        let main_ast = match NyashParser::parse_from_string(&code2) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        // When using AST prelude mode, combine prelude ASTs + main AST into one Program before macro expansion
        let ast_combined = if use_ast_prelude && !prelude_asts.is_empty() {
            use nyash_rust::ast::ASTNode;
            let mut combined: Vec<ASTNode> = Vec::new();
            for a in prelude_asts {
                if let ASTNode::Program { statements, .. } = a { combined.extend(statements); }
            }
            if let ASTNode::Program { statements, .. } = main_ast.clone() {
                combined.extend(statements);
            }
            ASTNode::Program { statements: combined, span: nyash_rust::ast::Span::unknown() }
        } else { main_ast };
        // Optional: dump AST statement kinds for quick diagnostics
        if std::env::var("NYASH_AST_DUMP").ok().as_deref() == Some("1") {
            use nyash_rust::ast::ASTNode;
            eprintln!("[ast] dump start (vm-fallback)");
            if let ASTNode::Program { statements, .. } = &ast_combined {
                for (i, st) in statements.iter().enumerate().take(50) {
                    let kind = match st {
                        ASTNode::BoxDeclaration { is_static, name, .. } => {
                            if *is_static { format!("StaticBox({})", name) } else { format!("Box({})", name) }
                        }
                        ASTNode::FunctionDeclaration { name, .. } => format!("FuncDecl({})", name),
                        ASTNode::FunctionCall { name, .. } => format!("FuncCall({})", name),
                        ASTNode::MethodCall { method, .. } => format!("MethodCall({})", method),
                        ASTNode::ScopeBox { .. } => "ScopeBox".to_string(),
                        ASTNode::ImportStatement { path, .. } => format!("Import({})", path),
                        ASTNode::UsingStatement { namespace_name, .. } => format!("Using({})", namespace_name),
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
            let mut decls: std::collections::HashMap<String, CoreBoxDecl> =
                std::collections::HashMap::new();
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
                    } = st
                    {
                        if *is_static {
                            continue; // modules/static boxes are not user-instantiable
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
                        decls.insert(name.clone(), decl);
                    }
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
                    ) -> Result<Box<dyn crate::box_trait::NyashBox>, RuntimeError> {
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

                    fn box_types(&self) -> Vec<&str> { vec![] }

                    fn is_available(&self) -> bool { true }

                    fn factory_type(
                        &self,
                    ) -> crate::box_factory::FactoryType {
                        crate::box_factory::FactoryType::User
                    }
                }
                let factory = InlineUserBoxFactory { decls: Arc::new(RwLock::new(decls)) };
                crate::runtime::unified_registry::register_user_defined_factory(Arc::new(factory));
            }
        }
        let mut compiler = MirCompiler::with_options(!self.config.no_optimize);
        let compile = match compiler.compile(ast) {
            Ok(c) => c,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };

        // Optional barrier-elision for parity with VM path
        let mut module_vm = compile.module.clone();
        if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
            let removed = crate::mir::passes::escape::escape_elide_barriers_vm(&mut module_vm);
            if removed > 0 { crate::cli_v!("[VM-fallback] escape_elide_barriers: removed {} barriers", removed); }
        }

        // Execute via MIR interpreter
        let mut vm = MirInterpreter::new();
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
