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

        // Verify MIR if requested
        if self.config.verify_mir {
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
        if self.config.dump_mir {
            let mut printer = if self.config.mir_verbose {
                MirPrinter::verbose()
            } else {
                MirPrinter::new()
            };
            if self.config.mir_verbose_effects {
                printer.set_show_effects_inline(true);
            }
            println!("🚀 MIR Output for {}:", filename);
            println!("{}", printer.print_module(&compile_result.module));
        }

        // Emit MIR JSON if requested and exit
        if let Some(path) = self.config.emit_mir_json.as_ref() {
            let p = std::path::Path::new(path);
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness(&compile_result.module, p) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            println!("MIR JSON written: {}", p.display());
            std::process::exit(0);
        }

        // Emit native executable via ny-llvmc (crate) and exit
        if let Some(exe_out) = self.config.emit_exe.as_ref() {
            let tmp_dir = std::path::Path::new("tmp");
            let _ = std::fs::create_dir_all(tmp_dir);
            let json_path = tmp_dir.join("nyash_cli_emit.json");
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness(&compile_result.module, &json_path) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            let ny_llvmc = std::env::var("NYASH_NY_LLVM_COMPILER")
                .ok()
                .and_then(|s| if !s.is_empty() { Some(std::path::PathBuf::from(s)) } else { None })
                .or_else(|| which::which("ny-llvmc").ok())
                .unwrap_or_else(|| std::path::PathBuf::from("target/release/ny-llvmc"));
            let mut cmd = std::process::Command::new(ny_llvmc);
            cmd.arg("--in").arg(&json_path)
                .arg("--emit").arg("exe")
                .arg("--out").arg(exe_out);
            if let Some(dir) = self.config.emit_exe_nyrt.as_ref() {
                cmd.arg("--nyrt").arg(dir);
            } else {
                cmd.arg("--nyrt").arg("target/release");
            }
            if let Some(flags) = self.config.emit_exe_libs.as_ref() {
                if !flags.trim().is_empty() {
                    cmd.arg("--libs").arg(flags);
                }
            }
            let status = cmd.status().unwrap_or_else(|e| {
                eprintln!("❌ failed to spawn ny-llvmc: {}", e);
                std::process::exit(1);
            });
            if !status.success() {
                eprintln!("❌ ny-llvmc failed with status: {:?}", status.code());
                std::process::exit(1);
            }
            println!("EXE written: {}", exe_out);
            std::process::exit(0);
        }
    }
}
