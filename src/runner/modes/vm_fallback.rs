use super::super::NyashRunner;
use crate::{parser::NyashParser, mir::MirCompiler, backend::MirInterpreter};
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
        // Using preprocessing (legacy inline or AST-prelude merge when NYASH_USING_AST=1)
        let mut code2 = code;
        let use_ast_prelude = crate::config::env::enable_using()
            && std::env::var("NYASH_USING_AST").ok().as_deref() == Some("1");
        let mut prelude_asts: Vec<nyash_rust::ast::ASTNode> = Vec::new();
        if crate::config::env::enable_using() {
            if use_ast_prelude {
                match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code2, filename) {
                    Ok((clean, paths)) => {
                        code2 = clean;
                        for p in paths {
                            match std::fs::read_to_string(&p) {
                                Ok(src) => match NyashParser::parse_from_string(&src) {
                                    Ok(ast) => prelude_asts.push(ast),
                                    Err(e) => { eprintln!("❌ Parse error in using prelude {}: {}", p, e); process::exit(1); }
                                },
                                Err(e) => { eprintln!("❌ Error reading using prelude {}: {}", p, e); process::exit(1); }
                            }
                        }
                    }
                    Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
                }
            } else {
                match crate::runner::modes::common_util::resolve::strip_using_and_register(self, &code2, filename) {
                    Ok(s) => { code2 = s; }
                    Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
                }
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
