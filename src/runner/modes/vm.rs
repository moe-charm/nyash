use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::MirCompiler, backend::VM, runtime::{NyashRuntime, NyashRuntimeBuilder}, ast::ASTNode, core::model::BoxDeclaration as CoreBoxDecl, interpreter::SharedState, box_factory::{builtin::BuiltinGroups, user_defined::UserDefinedBoxFactory}};
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
            let mut builder = NyashRuntimeBuilder::new()
                .with_builtin_groups(BuiltinGroups::native_full());
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

        // Execute with VM using prepared runtime
        let mut vm = VM::with_runtime(runtime);
        match vm.execute_module(&compile_result.module) {
            Ok(result) => {
                println!("✅ VM execution completed successfully!");
                println!("Result: {:?}", result);
            },
            Err(e) => { eprintln!("❌ VM execution error: {}", e); process::exit(1); }
        }
    }

    /// Collect Box declarations from AST and register into runtime
    pub(crate) fn collect_box_declarations(&self, ast: &ASTNode, runtime: &NyashRuntime) {
        fn walk(node: &ASTNode, runtime: &NyashRuntime) {
            match node {
                ASTNode::Program { statements, .. } => { for st in statements { walk(st, runtime); } }
                ASTNode::FunctionDeclaration { body, .. } => { for st in body { walk(st, runtime); } }
                ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    for (_mname, mnode) in methods { walk(mnode, runtime); }
                    for (_ckey, cnode) in constructors { walk(cnode, runtime); }
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
        walk(ast, runtime);
    }
}
