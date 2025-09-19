use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::MirCompiler, backend::MirInterpreter, runtime::{NyashRuntime, NyashRuntimeBuilder}, interpreter::SharedState, box_factory::user_defined::UserDefinedBoxFactory};
use std::{fs, process};
use std::sync::Arc;

impl NyashRunner {
    /// Execute MIR via lightweight interpreter backend
    pub(crate) fn execute_mir_interpreter_mode(&self, filename: &str) {
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
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);

        // Prepare runtime and collect Box declarations for user-defined types
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

        // Optional: VM-only escape analysis elides barriers; safe for interpreter too
        let mut module_interp = compile_result.module.clone();
        if std::env::var("NYASH_VM_ESCAPE_ANALYSIS").ok().as_deref() == Some("1") {
            let removed = nyash_rust::mir::passes::escape::escape_elide_barriers_vm(&mut module_interp);
            if removed > 0 { crate::cli_v!("[MIR-Interp] escape_elide_barriers: removed {} barriers", removed); }
        }

        // Execute with MIR interpreter
        let mut interp = MirInterpreter::new();
        match interp.execute_module(&module_interp) {
            Ok(result) => {
                println!("✅ MIR interpreter execution completed!");
                // Pretty-print using MIR return type when available
                if let Some(func) = module_interp.functions.get("main") {
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
            }
            Err(e) => {
                eprintln!("❌ MIR interpreter error: {}", e);
                process::exit(1);
            }
        }
        let _ = runtime; // reserved for future GC/safepoint integration
    }
}
#![cfg(feature = "interpreter-legacy")]
