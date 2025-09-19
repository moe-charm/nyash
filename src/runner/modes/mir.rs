use super::super::NyashRunner;
use nyash_rust::{
    mir::{MirCompiler, MirPrinter},
    parser::NyashParser,
};
use std::{fs, process};

impl NyashRunner {
    /// Execute MIR compilation and processing mode (split)
    pub(crate) fn execute_mir_mode(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
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

        let groups = self.config.as_groups();
        // Verify MIR if requested
        if groups.debug.verify_mir {
            println!("🔍 Verifying MIR...");
            match &compile_result.verification_result {
                Ok(()) => println!("✅ MIR verification passed!"),
                Err(errors) => {
                    eprintln!("❌ MIR verification failed:");
                    for error in errors {
                        eprintln!("  • {}", error);
                    }
                    process::exit(1);
                }
            }
        }

        // Dump MIR if requested
        if groups.debug.dump_mir {
            let mut printer = if groups.debug.mir_verbose {
                MirPrinter::verbose()
            } else {
                MirPrinter::new()
            };
            if groups.debug.mir_verbose_effects {
                printer.set_show_effects_inline(true);
            }
            println!("🚀 MIR Output for {}:", filename);
            println!("{}", printer.print_module(&compile_result.module));
        }

        // Emit MIR JSON if requested and exit
        if let Some(path) = groups.emit.emit_mir_json.as_ref() {
            let p = std::path::Path::new(path);
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness(&compile_result.module, p) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            println!("MIR JSON written: {}", p.display());
            std::process::exit(0);
        }

        // Emit native executable via ny-llvmc (crate) and exit
        if let Some(exe_out) = groups.emit.emit_exe.as_ref() {
            if let Err(e) = crate::runner::modes::common_util::exec::ny_llvmc_emit_exe_lib(
                &compile_result.module,
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
    }
}
