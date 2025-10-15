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
use serde_json::Value as J;

fn unwrap_value_in_place(x: &mut J) {
    if let Some(v) = x.get("value").and_then(|y| y.as_u64()) {
        *x = J::from(v as u32);
    }
}

fn gate_c_canonicalize_mir_json(v: &mut J) {
    if let Some(funcs) = v.get_mut("functions").and_then(|x| x.as_array_mut()) {
        for f in funcs.iter_mut() {
            if let Some(blocks) = f.get_mut("blocks").and_then(|x| x.as_array_mut()) {
                for b in blocks.iter_mut() {
            if let Some(insts) = b.get_mut("instructions").and_then(|x| x.as_array_mut()) {
                for inst in insts.iter_mut() {
                    // dst: {type:..., value:N} → N
                    if let Some(dst) = inst.get_mut("dst") { unwrap_value_in_place(dst); }
                    // ret.value: {type:..., value:N} → N
                    let op_name = inst.get("op").and_then(|x| x.as_str()).map(|s| s.to_string());
                    if let Some(op) = op_name.as_deref() {
                        match op {
                            "ret" | "return" => {
                                if let Some(vv) = inst.get_mut("value") { unwrap_value_in_place(vv); }
                            }
                            "binop" | "compare" => {
                                if let Some(lhs) = inst.get_mut("lhs") { unwrap_value_in_place(lhs); }
                                if let Some(rhs) = inst.get_mut("rhs") { unwrap_value_in_place(rhs); }
                            }
                            "branch" => {
                                if let Some(cond) = inst.get_mut("cond") { unwrap_value_in_place(cond); }
                                if let Some(th) = inst.get_mut("then") { unwrap_value_in_place(th); }
                                if let Some(el) = inst.get_mut("else") { unwrap_value_in_place(el); }
                            }
                            "jump" => {
                                if let Some(t) = inst.get_mut("target") { unwrap_value_in_place(t); }
                            }
                            _ => {}
                        }
                    }
                }
            }
                }
            }
        }
    }
}

impl NyashRunner {
    /// Gate C: Read MIR(JSON v0) and execute via Hakorune VM Core
    /// --nyvm-json-file <path> or --nyvm-pipe (stdin)
    /// Returns true if handled.
    pub(super) fn try_run_nyvm_mir_pipe(&self) -> bool {
        let groups = self.config.as_groups();
        if !(groups.parser.nyvm_pipe || groups.parser.nyvm_json_file.is_some()) {
            return false;
        }
        // Early direct path for Gate C: parse MIR(JSON) and execute via interpreter (quiet)
        // This avoids going through Ny wrappers and eliminates noisy logs.
        // Now gated by NYASH_GATE_C_DIRECT=1 to keep default behavior stable for smokes.
        if std::env::var("NYASH_GATE_C_DIRECT").ok().as_deref() == Some("1") {
            // Quiet/run-only env for Gate C
            std::env::set_var("NYASH_QUIET", "1");
            std::env::set_var("HAKO_QUIET", "1");
            std::env::set_var("NYASH_CLI_VERBOSE", "0");
            std::env::set_var("NYASH_NYRT_SILENT_RESULT", "1");
            std::env::set_var("NYASH_OPERATOR_BOX_PRELUDE", "0");
            std::env::set_var("NYASH_DISABLE_PLUGINS", "1");
            std::env::set_var("NYASH_SUPPRESS_MODULE_CONFLICT", "1");
            std::env::set_var("NYASH_CHECK_CONTRACTS", "0");

            if let Some(path) = &groups.parser.nyvm_json_file {
                if let Ok(json) = std::fs::read_to_string(path) {
                    let mut v: J = match serde_json::from_str(&json) { Ok(x) => x, Err(_) => return true };
                    gate_c_canonicalize_mir_json(&mut v);
                    let json2 = serde_json::to_string(&v).unwrap_or_default();
                    if let Ok(mut module) = super::mir_json_reader::parse_mir_json_v0_to_module(&json2) {
                        // Gate B safety: run optimizer repairs (materialize receivers)
                        let mut opt = crate::mir::optimizer::MirOptimizer::new();
                        let _ = opt.optimize_module(&mut module);
                        use crate::backend::MirInterpreter;
                        let mut interp = MirInterpreter::new();
                        if let Ok(result) = interp.execute_module(&module) {
                            if let Some(ib) = result.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                                println!("{}", ib.value);
                            } else {
                                let s = result.to_string_box().value;
                                if let Some(num) = s.trim().parse::<i64>().ok() { println!("{}", num); } else { println!("{}", s); }
                            }
                        } else { eprintln!("❌ VM execution error: run failed"); }
                    } else { eprintln!("❌ MIR JSON reader error: parse failed"); }
                } else { eprintln!("❌ json-file read error"); }
                return true;
            }
            if groups.parser.nyvm_pipe {
                use std::io::Read;
                let mut buf = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut buf) { eprintln!("❌ stdin read error: {}", e); return true; }
                let mut v: J = match serde_json::from_str(&buf) { Ok(x) => x, Err(_) => return true };
                gate_c_canonicalize_mir_json(&mut v);
                let json2 = serde_json::to_string(&v).unwrap_or_default();
                if let Ok(mut module) = super::mir_json_reader::parse_mir_json_v0_to_module(&json2) {
                    let mut opt = crate::mir::optimizer::MirOptimizer::new();
                    let _ = opt.optimize_module(&mut module);
                    use crate::backend::MirInterpreter;
                    let mut interp = MirInterpreter::new();
                    if let Ok(result) = interp.execute_module(&module) {
                        if let Some(ib) = result.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                            println!("{}", ib.value);
                        } else {
                            let s = result.to_string_box().value;
                            if let Some(num) = s.trim().parse::<i64>().ok() { println!("{}", num); } else { println!("{}", s); }
                        }
                    } else { eprintln!("❌ VM execution error: run failed"); }
                } else { eprintln!("❌ MIR JSON reader error: parse failed"); }
                return true;
            }
        }
        // Default path: parse MIR(JSON v0) and execute via interpreter, printing a single numeric line.
        // This avoids Ny wrapper VM pipeline instability during Gate C bring-up.
        if let Some(path) = &groups.parser.nyvm_json_file {
            // Quiet + strict env for Gate C
            std::env::set_var("NYASH_USING", "1");
            std::env::set_var("NYASH_USING_AST", "1");
            std::env::set_var("HAKO_ALLOW_USING_FILE", "1");
            std::env::set_var("NYASH_SUPPRESS_MODULE_CONFLICT", "1");
            std::env::set_var("NYASH_NYRT_SILENT_RESULT", "1");
            std::env::set_var("HAKO_QUIET", "1");
            std::env::set_var("NYASH_QUIET", "1");
            std::env::set_var("NYASH_CLI_VERBOSE", "0");
            std::env::set_var("NYASH_OPERATOR_BOX_PRELUDE", "0");
            std::env::set_var("NYASH_DISABLE_PLUGINS", "1");
            std::env::set_var("NYASH_CHECK_CONTRACTS", "0");
            // Quiet/run-only env for Gate C numeric output
            std::env::set_var("NYASH_QUIET", "1");
            std::env::set_var("HAKO_QUIET", "1");
            std::env::set_var("NYASH_CLI_VERBOSE", "0");
            std::env::set_var("NYASH_NYRT_SILENT_RESULT", "1");
            // Read and canonicalize, then execute via interpreter
            match std::fs::read_to_string(path) {
                Ok(json) => {
                    let mut v: J = match serde_json::from_str(&json) { Ok(x) => x, Err(_) => return true };
                    gate_c_canonicalize_mir_json(&mut v);
                    let json2 = serde_json::to_string(&v).unwrap_or_default();
                    if let Ok(module) = super::mir_json_reader::parse_mir_json_v0_to_module(&json2) {
                        use crate::backend::MirInterpreter;
                        let mut interp = MirInterpreter::new();
                        if let Ok(result) = interp.execute_module(&module) {
                            if let Some(ib) = result.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                                println!("{}", ib.value);
                            } else {
                                let s = result.to_string_box().value;
                                if let Some(num) = s.trim().parse::<i64>().ok() { println!("{}", num); } else { println!("{}", s); }
                            }
                        } else { eprintln!("❌ VM execution error: run failed"); }
                    } else { eprintln!("❌ MIR JSON reader error: parse failed"); }
                }
                Err(e) => eprintln!("❌ json-file read error: {}", e),
            }
            return true;
        }
        // stdin mode
        // stdin mode (direct path first)
        if groups.parser.nyvm_pipe {
            // If direct path is explicitly requested, handle here and return
            if std::env::var("NYASH_GATE_C_DIRECT").ok().as_deref() == Some("1") {
                use std::io::Read;
                let mut buf = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                    eprintln!("❌ stdin read error: {}", e);
                    std::process::exit(1);
                }
                std::env::set_var("NYASH_USING", "1");
                std::env::set_var("NYASH_USING_AST", "1");
                std::env::set_var("HAKO_ALLOW_USING_FILE", "1");
                std::env::set_var("NYASH_SUPPRESS_MODULE_CONFLICT", "1");
                std::env::set_var("NYASH_NYRT_SILENT_RESULT", "1");
                std::env::set_var("HAKO_QUIET", "1");
                std::env::set_var("NYASH_QUIET", "1");
                std::env::set_var("NYASH_CLI_VERBOSE", "0");
                std::env::set_var("NYASH_OPERATOR_BOX_PRELUDE", "0");
                std::env::set_var("NYASH_DISABLE_PLUGINS", "1");
                std::env::set_var("NYASH_CHECK_CONTRACTS", "0");
                match super::mir_json_reader::parse_mir_json_v0_to_module(&buf) {
                    Ok(mut module) => {
                        let mut opt = crate::mir::optimizer::MirOptimizer::new();
                        let _ = opt.optimize_module(&mut module);
                        use crate::backend::MirInterpreter;
                        let mut interp = MirInterpreter::new();
                        match interp.execute_module(&module) {
                            Ok(result) => {
                                if let Some(ib) = result.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                                    println!("{}", ib.value);
                                } else {
                                    let s = result.to_string_box().value;
                                    if let Some(num) = s.trim().parse::<i64>().ok() { println!("{}", num); } else { println!("{}", s); }
                                }
                            }
                            Err(e) => { eprintln!("❌ VM execution error: {}", e); }
                        }
                    }
                    Err(e) => eprintln!("❌ MIR JSON reader error: {}", e),
                }
                return true;
            }
            use std::io::Read;
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("❌ stdin read error: {}", e);
                std::process::exit(1);
            }
            // Directly execute the MIR(JSON) via interpreter and print a single numeric line
            let mut v: J = match serde_json::from_str(&buf) { Ok(x) => x, Err(_) => return true };
            gate_c_canonicalize_mir_json(&mut v);
            let json2 = serde_json::to_string(&v).unwrap_or_default();
            if let Ok(mut module) = super::mir_json_reader::parse_mir_json_v0_to_module(&json2) {
                let mut opt = crate::mir::optimizer::MirOptimizer::new();
                let _ = opt.optimize_module(&mut module);
                use crate::backend::MirInterpreter;
                let mut interp = MirInterpreter::new();
                if let Ok(result) = interp.execute_module(&module) {
                    if let Some(ib) = result.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
                        println!("{}", ib.value);
                    } else {
                        let s = result.to_string_box().value;
                        if let Some(num) = s.trim().parse::<i64>().ok() { println!("{}", num); } else { println!("{}", s); }
                    }
                } else { eprintln!("❌ VM execution error: run failed"); }
            } else { eprintln!("❌ MIR JSON reader error: parse failed"); }
            return true;
        }
        // Fallback wrapper path removed (Gate C uses direct interpreter path above)
        true
    }
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
        if json.contains("\"kind\":\"MIR\"") || (json.trim_start().starts_with('{') && json.contains("\"functions\"")) {
            eprintln!("❌ JSON v0 bridge error: input appears to be MIR(JSON v0).
   Hint: Use a Ny driver with 'using selfhost.vm.mir_min as MirVmMin; MirVmMin.run(json)' to execute,
   or convert to AST(JSON v0) and pass via --json-file.");
            std::process::exit(1);
        }
        if json.contains("\"kind\":\"MIR\"") || (json.trim_start().starts_with('{') && json.contains("\"functions\"")) {
            if std::env::var("NYASH_JSON_MIR_READER_DEV").ok().as_deref() == Some("1") {
                match super::mir_json_reader::parse_mir_json_v0_to_module(&json) {
                    Ok(module) => {
                        super::json_v0_bridge::maybe_dump_mir(&module);
                        self.execute_mir_module(&module);
                        return true;
                    }
                    Err(e) => {
                        eprintln!("❌ MIR JSON reader error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("❌ JSON v0 bridge error: input appears to be MIR(JSON v0).\n   Hint: Use a Ny driver with 'using selfhost.vm.mir_min as MirVmMin; MirVmMin.run(json)' to execute,\n   or set NYASH_JSON_MIR_READER_DEV=1 to enable experimental reader.");
                std::process::exit(1);
            }
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
