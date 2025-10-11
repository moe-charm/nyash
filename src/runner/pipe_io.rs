/*!
 * Runner Pipe I/O helpers — JSON v0 handling
 *
 * Extracted from runner/mod.rs to keep the main runner slimmer.
 * Handles:
 *  - Reading JSON v0 from stdin or file
 *  - Optional MIR dump
 *  - Optional PyVM delegation via tools/pyvm_runner.py
 *  - Fallback to MIR interpreter execution
 */

use super::*;

impl NyashRunner {
    /// Try to handle `--ny-parser-pipe` / `--json-file` flow.
    /// Returns true if the request was handled (program should return early).
    pub(super) fn try_run_json_v0_pipe(&self) -> bool {
        let groups = self.config.as_groups();
        if !(groups.parser.ny_parser_pipe || groups.parser.json_file.is_some()) {
            return false;
        }
        let json = if let Some(path) = &groups.parser.json_file {
            match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ json-file read error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            use std::io::Read;
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("❌ stdin read error: {}", e);
                std::process::exit(1);
            }
            buf
        };
        // Thin detection: if input looks like MIR(JSON v0) ({"version":0,"kind":"MIR"}),
        // we currently do not support direct MIR JSON execution in this path.
        // Suggest using Ny-side MirVmMin or the selfhost pipeline to run it.
        if json.contains(""kind":"MIR"") || (json.trim_start().starts_with('{') && json.contains(""functions"")) {
            eprintln!("❌ JSON v0 bridge error: input appears to be MIR(JSON v0).
   Hint: Use a Ny driver with `using selfhost.vm.mir_min as MirVmMin; MirVmMin.run(json)` to execute,
   or convert to AST(JSON v0) and pass via --json-file.");
            std::process::exit(1);
        }
        match super::json_v0_bridge::parse_json_v0_to_module(&json) {
            Ok(module) => {
                // Optional dump via env verbose
                super::json_v0_bridge::maybe_dump_mir(&module);
                // Optional: delegate to PyVM when NYASH_PIPE_USE_PYVM=1
                if crate::config::env::pipe_use_pyvm() {
                    #[cfg(feature = "pyvm-bridge")]
                    {
                        let py = which::which("python3").ok();
                        if let Some(py3) = py {
                            let runner = std::path::Path::new("tools/pyvm_runner.py");
                            if runner.exists() {
                            // Emit MIR(JSON) for PyVM
                            let tmp_dir = std::path::Path::new("tmp");
                            let _ = std::fs::create_dir_all(tmp_dir);
                            let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
                            if let Err(e) = super::mir_json_emit::emit_mir_json_for_harness_bin(
                                &module,
                                &mir_json_path,
                            ) {
                                eprintln!("❌ PyVM MIR JSON emit error: {}", e);
                                std::process::exit(1);
                            }
                            crate::cli_v!("[Bridge] using PyVM (pipe) → {}", mir_json_path.display());
                            // Determine entry function (prefer Main.main; otherwise unique <Box>.main; then top-level main when allowed)
                            let allow_top = crate::config::env::entry_allow_toplevel_main();
                            let prefer_static = crate::config::env::entry_prefer_static_main();
                            let entry = if module.functions.contains_key("Main.main") {
                                "Main.main"
                            } else if prefer_static {
                                let mut cands: Vec<&str> = Vec::new();
                                for k in module.functions.keys() {
                                    if k.ends_with(".main") || k.ends_with(".main/0") {
                                        cands.push(k.as_str());
                                    }
                                }
                                if cands.len() == 1 { cands[0] }
                                else if allow_top && module.functions.contains_key("main") { "main" }
                                else if module.functions.contains_key("main") { eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence."); "main" }
                                else { "Main.main" }
                            } else if allow_top && module.functions.contains_key("main") {
                                "main"
                            } else if module.functions.contains_key("main") {
                                eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence.");
                                "main"
                            } else {
                                "Main.main"
                            };
                            let status = std::process::Command::new(py3)
                                .args([
                                    runner.to_string_lossy().as_ref(),
                                    "--in",
                                    &mir_json_path.display().to_string(),
                                    "--entry",
                                    entry,
                                ])
                                .status()
                                .map_err(|e| format!("spawn pyvm: {}", e))
                                .unwrap();
                            let code = status.code().unwrap_or(1);
                            if !status.success() { crate::cli_v!("❌ PyVM (pipe) failed (status={})", code); }
                            std::process::exit(code);
                            } else {
                                eprintln!("❌ PyVM runner not found: {}", runner.display());
                                std::process::exit(1);
                            }
                        } else {
                            eprintln!("❌ python3 not found in PATH. Install Python 3 to use PyVM with --ny-parser-pipe.");
                            std::process::exit(1);
                        }
                    }
                    #[cfg(not(feature = "pyvm-bridge"))]
                    {
                        eprintln!("[pipe] PyVM bridge disabled (feature off); using MIR interpreter path.");
                    }
                }
                // Default: Execute via MIR interpreter
                self.execute_mir_module(&module);
                true
            }
            Err(e) => {
                eprintln!("❌ JSON v0 bridge error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
