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
        // Using preprocessing (strip + autoload)
        let mut code2 = code;
        if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::strip_using_and_register(self, &code2, filename) {
                Ok(s) => { code2 = s; }
                Err(e) => { eprintln!("❌ {}", e); process::exit(1); }
            }
        }
        // Dev sugar pre-expand: @name = expr → local name = expr
        code2 = crate::runner::modes::common_util::resolve::preexpand_at_local(&code2);

        // Parse -> expand macros -> compile MIR
        let ast = match NyashParser::parse_from_string(&code2) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        let ast = crate::r#macro::maybe_expand_and_dump(&ast, false);
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
        match vm.execute_module(&module_vm) {
            Ok(_ret) => { /* interpreter already prints via println/console in program */ }
            Err(e) => {
                eprintln!("❌ VM fallback error: {}", e);
                process::exit(1);
            }
        }
    }
}
