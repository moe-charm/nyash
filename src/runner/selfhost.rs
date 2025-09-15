/*!
 * Runner selfhost helpers — Ny compiler pipeline (Ny -> JSON v0)
 *
 * Transitional shim: provides a stable entrypoint from callers, while the
 * heavy implementation currently lives in modes/common.rs. Next step will
 * migrate the full implementation here.
 */

use super::*;

use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter};
use std::{fs, process};
use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::thread::sleep;

impl NyashRunner {
    /// Selfhost (Ny -> JSON v0) pipeline: EXE/VM/Python フォールバック含む
    pub(crate) fn try_run_selfhost_pipeline(&self, filename: &str) -> bool {
        use std::io::Write;
        // Read input source
        let code = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => { eprintln!("[ny-compiler] read error: {}", e); return false; }
        };
        // Optional Phase-15: strip `using` lines and register modules (same policy as execute_nyash_file)
        let enable_using = std::env::var("NYASH_ENABLE_USING").ok().as_deref() == Some("1");
        let mut code_ref: std::borrow::Cow<'_, str> = std::borrow::Cow::Borrowed(&code);
        if enable_using {
            let mut out = String::with_capacity(code.len());
            let mut used_names: Vec<(String, Option<String>)> = Vec::new();
            for line in code.lines() {
                let t = line.trim_start();
                if t.starts_with("using ") {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[using] stripped(line→selfhost): {}", line);
                    }
                    let rest0 = t.strip_prefix("using ").unwrap().trim();
                    let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                    let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                        (rest0[..pos].trim().to_string(), Some(rest0[pos+4..].trim().to_string()))
                    } else { (rest0.to_string(), None) };
                    let is_path = target.starts_with('"') || target.starts_with("./") || target.starts_with('/') || target.ends_with(".nyash");
                    if is_path {
                        let path = target.trim_matches('"').to_string();
                        let name = alias.clone().unwrap_or_else(|| {
                            std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string()
                        });
                        used_names.push((name, Some(path)));
                    } else {
                        used_names.push((target, alias));
                    }
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            // Register modules into minimal registry with best-effort path resolution
            for (ns_or_alias, alias_or_path) in used_names {
                if let Some(path) = alias_or_path {
                    let sb = crate::box_trait::StringBox::new(path);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                } else {
                    let rel = format!("apps/{}.nyash", ns_or_alias.replace('.', "/"));
                    let exists = std::path::Path::new(&rel).exists();
                    let path_or_ns = if exists { rel } else { ns_or_alias.clone() };
                    let sb = crate::box_trait::StringBox::new(path_or_ns);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                }
            }
            code_ref = std::borrow::Cow::Owned(out);
        }

        // Write to tmp/ny_parser_input.ny (as expected by Ny parser v0), unless forced to reuse existing tmp
        let use_tmp_only = std::env::var("NYASH_NY_COMPILER_USE_TMP_ONLY").ok().as_deref() == Some("1");
        let tmp_dir = std::path::Path::new("tmp");
        if let Err(e) = std::fs::create_dir_all(tmp_dir) {
            eprintln!("[ny-compiler] mkdir tmp failed: {}", e);
            return false;
        }
        let tmp_path = tmp_dir.join("ny_parser_input.ny");
        if !use_tmp_only {
            match std::fs::File::create(&tmp_path) {
                Ok(mut f) => {
                    if let Err(e) = f.write_all(code_ref.as_bytes()) {
                        eprintln!("[ny-compiler] write tmp failed: {}", e);
                        return false;
                    }
                }
                Err(e) => { eprintln!("[ny-compiler] open tmp failed: {}", e); return false; }
            }
        }
        // EXE-first: if requested, try external parser EXE (nyash_compiler)
        if std::env::var("NYASH_USE_NY_COMPILER_EXE").ok().as_deref() == Some("1") {
            // Resolve parser EXE path
            let exe_path = if let Ok(p) = std::env::var("NYASH_NY_COMPILER_EXE_PATH") {
                std::path::PathBuf::from(p)
            } else {
                let mut p = std::path::PathBuf::from("dist/nyash_compiler");
                #[cfg(windows)]
                { p.push("nyash_compiler.exe"); }
                #[cfg(not(windows))]
                { p.push("nyash_compiler"); }
                if !p.exists() {
                    // Try PATH
                    if let Ok(w) = which::which("nyash_compiler") { w } else { p }
                } else { p }
            };
            if exe_path.exists() {
                let mut cmd = std::process::Command::new(&exe_path);
                // Prefer passing the original filename directly (parser EXE accepts positional path)
                cmd.arg(filename);
                // Gates
                if std::env::var("NYASH_NY_COMPILER_MIN_JSON").ok().as_deref() == Some("1") { cmd.arg("--min-json"); }
                if std::env::var("NYASH_SELFHOST_READ_TMP").ok().as_deref() == Some("1") { cmd.arg("--read-tmp"); }
                if let Ok(raw) = std::env::var("NYASH_NY_COMPILER_CHILD_ARGS") { for tok in raw.split_whitespace() { cmd.arg(tok); } }
                let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
                let mut cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
                let mut child = match cmd.spawn() { Ok(c) => c, Err(e) => { eprintln!("[ny-compiler] exe spawn failed: {}", e); return false; } };
                let mut ch_stdout = child.stdout.take();
                let mut ch_stderr = child.stderr.take();
                let start = Instant::now();
                let mut timed_out = false;
                loop {
                    match child.try_wait() {
                        Ok(Some(_status)) => { break; }
                        Ok(None) => {
                            if start.elapsed() >= Duration::from_millis(timeout_ms) { let _ = child.kill(); let _ = child.wait(); timed_out = true; break; }
                            sleep(Duration::from_millis(10));
                        }
                        Err(e) => { eprintln!("[ny-compiler] exe wait error: {}", e); return false; }
                    }
                }
                let mut out_buf = Vec::new();
                let mut err_buf = Vec::new();
                if let Some(mut s) = ch_stdout { let _ = s.read_to_end(&mut out_buf); }
                if let Some(mut s) = ch_stderr { let _ = s.read_to_end(&mut err_buf); }
                if timed_out {
                    let head = String::from_utf8_lossy(&out_buf).chars().take(200).collect::<String>();
                    eprintln!("[ny-compiler] exe timeout after {} ms; stdout(head)='{}'", timeout_ms, head.replace('\n', "\\n"));
                    return false;
                }
                let stdout = match String::from_utf8(out_buf) { Ok(s) => s, Err(_) => String::new() };
                let mut json_line = String::new();
                for line in stdout.lines() { let t = line.trim(); if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") { json_line = t.to_string(); break; } }
                if json_line.is_empty() {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        let head: String = stdout.chars().take(200).collect();
                        let errh: String = String::from_utf8_lossy(&err_buf).chars().take(200).collect();
                        eprintln!("[ny-compiler] exe produced no JSON; stdout(head)='{}' stderr(head)='{}'", head.replace('\n', "\\n"), errh.replace('\n', "\\n"));
                    }
                    return false;
                }
                // Parse JSON v0 → MIR module
                match super::json_v0_bridge::parse_json_v0_to_module(&json_line) {
                    Ok(module) => {
                        println!("🚀 Ny compiler EXE path (ny→json_v0) ON");
                        super::json_v0_bridge::maybe_dump_mir(&module);
                        let emit_only = std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1";
                        if emit_only {
                            return false;
                        } else {
                            // Prefer PyVM when requested AND the module contains BoxCalls (Stage-2 semantics)
                            let needs_pyvm = module.functions.values().any(|f| {
                                f.blocks.values().any(|bb| bb.instructions.iter().any(|inst| matches!(inst, crate::mir::MirInstruction::BoxCall { .. })))
                            });
                            if needs_pyvm && std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
                                if let Ok(py3) = which::which("python3") {
                                    let runner = std::path::Path::new("tools/pyvm_runner.py");
                                    if runner.exists() {
                                        let tmp_dir = std::path::Path::new("tmp");
                                        let _ = std::fs::create_dir_all(tmp_dir);
                                        let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
                                        if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(&module, &mir_json_path) {
                                            eprintln!("❌ PyVM MIR JSON emit error: {}", e);
                                            process::exit(1);
                                        }
                                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                            eprintln!("[Bridge] using PyVM (selfhost) → {}", mir_json_path.display());
                                        }
                                        let entry = if module.functions.contains_key("Main.main") { "Main.main" } else if module.functions.contains_key("main") { "main" } else { "Main.main" };
                                        let status = std::process::Command::new(py3)
                                            .args(["tools/pyvm_runner.py", "--in", &mir_json_path.display().to_string(), "--entry", entry])
                                            .status().map_err(|e| format!("spawn pyvm: {}", e)).unwrap();
                                        let code = status.code().unwrap_or(1);
                                        if !status.success() {
                                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                                eprintln!("❌ PyVM (selfhost) failed (status={})", code);
                                            }
                                        }
                                        // Harmonize with interpreter path for smokes: print Result then exit code
                                        println!("Result: {}", code);
                                        std::process::exit(code);
                                    }
                                }
                            }
                            self.execute_mir_module(&module);
                            return true;
                        }
                    }
                    Err(e) => { eprintln!("[ny-compiler] json parse error: {}", e); return false; }
                }
            }
        }

        // Fallback: run compiler.nyash via VM(PyVM) and pick the JSON line
        // Guard against recursion: ensure child does NOT enable selfhost pipeline.
        let mut raw = String::new();
        {
            // Locate current nyash executable
            let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("target/release/nyash"));
            let mut cmd = std::process::Command::new(exe);
            cmd.arg("--backend").arg("vm").arg("apps/selfhost-compiler/compiler.nyash");
            // Pass script args to child when gated
            if std::env::var("NYASH_NY_COMPILER_MIN_JSON").ok().as_deref() == Some("1") { cmd.arg("--").arg("--min-json"); }
            if std::env::var("NYASH_SELFHOST_READ_TMP").ok().as_deref() == Some("1") { cmd.arg("--").arg("--read-tmp"); }
            // Recursion guard and minimal, quiet env for child
            cmd.env_remove("NYASH_USE_NY_COMPILER");
            cmd.env_remove("NYASH_CLI_VERBOSE");
            cmd.env("NYASH_JSON_ONLY", "1");
            if let Ok(v) = std::env::var("NYASH_JSON_INCLUDE_USINGS") { cmd.env("NYASH_JSON_INCLUDE_USINGS", v); }
            if let Ok(v) = std::env::var("NYASH_ENABLE_USING") { cmd.env("NYASH_ENABLE_USING", v); }

            // Timeout guard (default 2000ms)
            let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
            let mut cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => { eprintln!("[ny-compiler] spawn nyash vm failed: {}", e); return false; }
            };
            let mut ch_stdout = child.stdout.take();
            let mut ch_stderr = child.stderr.take();
            let start = Instant::now();
            let mut timed_out = false;
            loop {
                match child.try_wait() {
                    Ok(Some(_status)) => { break; }
                    Ok(None) => {
                        if start.elapsed() >= Duration::from_millis(timeout_ms) {
                            let _ = child.kill();
                            let _ = child.wait();
                            timed_out = true;
                            break;
                        }
                        sleep(Duration::from_millis(10));
                    }
                    Err(e) => { eprintln!("[ny-compiler] child wait error: {}", e); break; }
                }
            }
            let mut out_buf = Vec::new();
            let mut err_buf = Vec::new();
            if let Some(mut s) = ch_stdout { let _ = s.read_to_end(&mut out_buf); }
            if let Some(mut s) = ch_stderr { let _ = s.read_to_end(&mut err_buf); }
            if timed_out {
                let head = String::from_utf8_lossy(&out_buf).chars().take(200).collect::<String>();
                eprintln!("[ny-compiler] child timeout after {} ms; stdout(head)='{}'", timeout_ms, head.replace('\n', "\\n"));
            }
            raw = String::from_utf8_lossy(&out_buf).to_string();
        }
        let mut json_line = String::new();
        for line in raw.lines() {
            let t = line.trim();
            if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") { json_line = t.to_string(); break; }
        }
        if json_line.is_empty() { return false; }
        match super::json_v0_bridge::parse_json_v0_to_module(&json_line) {
            Ok(module) => {
                super::json_v0_bridge::maybe_dump_mir(&module);
                let emit_only = std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1";
                if emit_only { return false; }
                // Prefer PyVM when requested AND the module contains BoxCalls (Stage-2 semantics)
                let needs_pyvm = module.functions.values().any(|f| {
                    f.blocks.values().any(|bb| bb.instructions.iter().any(|inst| matches!(inst, crate::mir::MirInstruction::BoxCall { .. })))
                });
                if needs_pyvm && std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
                    if let Ok(py3) = which::which("python3") {
                        let runner = std::path::Path::new("tools/pyvm_runner.py");
                        if runner.exists() {
                            let tmp_dir = std::path::Path::new("tmp");
                            let _ = std::fs::create_dir_all(tmp_dir);
                            let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
                            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(&module, &mir_json_path) {
                                eprintln!("❌ PyVM MIR JSON emit error: {}", e);
                                process::exit(1);
                            }
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!("[Bridge] using PyVM (selfhost-fallback) → {}", mir_json_path.display());
                            }
                            let entry = if module.functions.contains_key("Main.main") { "Main.main" } else if module.functions.contains_key("main") { "main" } else { "Main.main" };
                            let status = std::process::Command::new(py3)
                                .args(["tools/pyvm_runner.py", "--in", &mir_json_path.display().to_string(), "--entry", entry])
                                .status().map_err(|e| format!("spawn pyvm: {}", e)).unwrap();
                            let code = status.code().unwrap_or(1);
                            if !status.success() {
                                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                    eprintln!("❌ PyVM (selfhost-fallback) failed (status={})", code);
                                }
                            }
                            // Harmonize with interpreter path for smokes
                            println!("Result: {}", code);
                            std::process::exit(code);
                        }
                    }
                }
                self.execute_mir_module(&module);
                true
            }
            Err(e) => { eprintln!("❌ JSON v0 bridge error: {}", e); false }
        }
    }
}
