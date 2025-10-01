/*!
 * Runner selfhost helpers — Ny compiler pipeline (Ny -> JSON v0)
 *
 * Transitional shim: provides a stable entrypoint from callers, while the
 * heavy implementation currently lives in modes/common.rs. Next step will
 * migrate the full implementation here.
 */

use super::*;
use nyash_rust::{mir::MirCompiler, parser::NyashParser};
use std::{fs, process};

impl NyashRunner {
    /// Selfhost (Ny -> JSON v0) pipeline: EXE/VM/Python フォールバック含む
    pub(crate) fn try_run_selfhost_pipeline(&self, filename: &str) -> bool {
        use std::io::Write;
        // Read input source
        let code = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[ny-compiler] read error: {}", e);
                return false;
            }
        };
        // Optional Phase-15: strip `using` lines and register modules (same policy as execute_nyash_file)
        let mut code_ref: std::borrow::Cow<'_, str> = std::borrow::Cow::Borrowed(&code);
        if crate::config::env::enable_using() {
            match crate::runner::modes::common_util::resolve::resolve_prelude_paths_profiled(self, &code, filename) {
                Ok((clean, paths, _aliases)) => {
                    if !paths.is_empty() && !crate::config::env::using_ast_enabled() {
                        eprintln!("[ny-compiler] using: AST prelude merge is disabled in this profile. Enable NYASH_USING_AST=1 or remove 'using' lines.");
                        return false;
                    }
                    code_ref = std::borrow::Cow::Owned(clean);
                    // Selfhost compile path does not need to parse prelude ASTs here.
                }
                Err(e) => { eprintln!("[ny-compiler] {}", e); return false; }
            }
        }

        // Promote dev sugar to standard: pre-expand line-head '@name[:T] = expr' to 'local name[:T] = expr'
        {
            let expanded = crate::runner::modes::common_util::resolve::preexpand_at_local(code_ref.as_ref());
            code_ref = std::borrow::Cow::Owned(expanded);
        }

        // Write to tmp/ny_parser_input.ny (as expected by Ny parser v0), unless forced to reuse existing tmp
        let use_tmp_only = crate::config::env::ny_compiler_use_tmp_only();
        let tmp_dir = std::path::Path::new("tmp");
        if let Err(e) = std::fs::create_dir_all(tmp_dir) {
            eprintln!("[ny-compiler] mkdir tmp failed: {}", e);
            return false;
        }

        // Optional macro pre‑expand path for selfhost
        // Default: auto when macro engine is enabled (safe: PyVM only)
        // Gate: NYASH_MACRO_SELFHOST_PRE_EXPAND={1|auto|0}
        {
            let preenv = std::env::var("NYASH_MACRO_SELFHOST_PRE_EXPAND")
                .ok()
                .or_else(|| if crate::r#macro::enabled() { Some("auto".to_string()) } else { None });
            let do_pre = match preenv.as_deref() {
                Some("1") => true,
                Some("auto") => crate::r#macro::enabled() && crate::config::env::vm_use_py(),
                _ => false,
            };
            if do_pre && crate::r#macro::enabled() {
            crate::cli_v!("[ny-compiler] selfhost macro pre-expand: engaging (mode={:?})", preenv);
            match NyashParser::parse_from_string(code_ref.as_ref()) {
                Ok(ast0) => {
                    let ast = crate::r#macro::maybe_expand_and_dump(&ast0, false);
                    // Compile to MIR and execute (respect VM/PyVM policy similar to vm mode)
                    let mut mir_compiler = MirCompiler::with_options(true);
                    match mir_compiler.compile(ast) {
                        Ok(result) => {
                            let prefer_pyvm = crate::config::env::vm_use_py();
                            if prefer_pyvm {
                                if let Ok(code) = crate::runner::modes::common_util::pyvm::run_pyvm_harness_lib(&result.module, "selfhost-preexpand") {
                                    println!("Result: {}", code);
                                    std::process::exit(code);
                                } else {
                                    eprintln!("❌ PyVM error (selfhost-preexpand)");
                                    std::process::exit(1);
                                }
                            } else {
                                // For now, only PyVM path is supported in pre-expand mode; fall back otherwise.
                                crate::cli_v!("[ny-compiler] pre-expand path requires NYASH_VM_USE_PY=1; falling back to default selfhost");
                                return false;
                            }
                        }
                        Err(e) => {
                            eprintln!("[ny-compiler] pre-expand compile error: {}", e);
                            return false;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[ny-compiler] pre-expand parse error: {}", e);
                    return false;
                }
            }
            }
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
                Err(e) => {
                    eprintln!("[ny-compiler] open tmp failed: {}", e);
                    return false;
                }
            }
        }
        // Preferred: run Ny selfhost compiler program (apps/selfhost-compiler or apps/selfhost)
        // This avoids inline embedding pitfalls and supports Stage-3 gating via args.
        {
            use crate::runner::modes::common_util::selfhost::{child, json};
            let exe = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("target/release/nyash"));
            let parser_prog_hako = std::path::Path::new("apps/selfhost-compiler/compiler.hako");
            let parser_prog_nyash = std::path::Path::new("apps/selfhost-compiler/compiler.nyash");
            let parser_prog_legacy = std::path::Path::new("apps/selfhost/compiler/compiler.nyash");
            let parser_prog = if parser_prog_hako.exists() { parser_prog_hako } else if parser_prog_nyash.exists() { parser_prog_nyash } else { parser_prog_legacy };
            if parser_prog.exists() {
                // Build extra args forwarded to child program
                let mut extra_owned: Vec<String> = Vec::new();
                // Always start with delimiter for child args
                extra_owned.push("--".to_string());
                if crate::config::env::selfhost_read_tmp() { extra_owned.push("--read-tmp".to_string()); }
                if crate::config::env::ny_compiler_min_json() { extra_owned.push("--min-json".to_string()); }
                if crate::config::env::ny_compiler_stage3() { extra_owned.push("--stage3".to_string()); }
                // Dev trace: map NYASH_EMIT_TRACE=1 -> --emit-trace (emit-only, safe; default OFF)
                if std::env::var("NYASH_EMIT_TRACE").ok().as_deref() == Some("1") {
                    extra_owned.push("--emit-trace".to_string());
                }
                // Optional lowering preference (CFG/materialize); default OFF
                if std::env::var("NYASH_PREFER_CFG2").ok().as_deref() == Some("1") {
                    extra_owned.push("--prefer-cfg2".to_string());
                } else if std::env::var("NYASH_PREFER_CFG").ok().as_deref() == Some("1") {
                    extra_owned.push("--prefer-cfg".to_string());
                }
                // Optional: map env toggles to child args (prepasses)
                if std::env::var("NYASH_SCOPEBOX_ENABLE").ok().as_deref() == Some("1") {
                    extra_owned.push("--scopebox".to_string());
                }
                if std::env::var("NYASH_LOOPFORM_NORMALIZE").ok().as_deref() == Some("1") {
                    extra_owned.push("--loopform".to_string());
                }
                // Optional: developer-provided child args passthrough（新: NYASH_NY_COMPILER_CHILD_ARGS, 旧: NYASH_SELFHOST_CHILD_ARGS）
                if let Some(raw) = crate::config::env::ny_compiler_child_args() {
                    for tok in raw.split_whitespace() { if !tok.is_empty() { extra_owned.push(tok.to_string()); } }
                } else if let Ok(raw) = std::env::var("NYASH_SELFHOST_CHILD_ARGS") {
                    for tok in raw.split_whitespace() { if !tok.trim().is_empty() { extra_owned.push(tok.to_string()); } }
                }
                let extra: Vec<&str> = extra_owned.iter().map(|s| s.as_str()).collect();
                let timeout_ms: u64 = crate::config::env::ny_compiler_timeout_ms();
                if let Some(line) = child::run_ny_program_capture_json(
                    &exe,
                    parser_prog,
                    timeout_ms,
                    &extra,
                    &["NYASH_USE_NY_COMPILER", "NYASH_CLI_VERBOSE"],
                    &[
                        ("NYASH_JSON_ONLY", "1"),
                        ("NYASH_ENABLE_USING", "1"),
                        ("NYASH_ALLOW_USING_FILE", "1"),
                        ("NYASH_USING_AST", "1"),
                    ],
                ) {
                    // Emit-only: print raw JSON (even if not JSON v0)
                    if crate::config::env::ny_compiler_emit_only() {
                        println!("{}", line);
                        return true;
                    }
                    match json::parse_json_v0_line(&line) {
                        Ok(module) => {
                            super::json_v0_bridge::maybe_dump_mir(&module);
                            // Regular execution path
                            // Prefer PyVM path when requested
                            if crate::config::env::vm_use_py() {
                                    if let Some(code) = crate::runner::modes::common_util::selfhost::json::run_pyvm_module(&module, "selfhost") {
                                        println!("Result: {}", code);
                                        std::process::exit(code);
                                    }
                                }
                                self.execute_mir_module(&module);
                                return true;
                            }
                        Err(e) => {
                            eprintln!("[ny-compiler] json parse error (child): {}", e);
                        }
                    }
                }
            }
        }

        // Python MVP-first: prefer the lightweight harness to produce JSON v0 (unless skipped)
        if std::env::var("NYASH_NY_COMPILER_SKIP_PY").ok().as_deref() != Some("1") {
            if let Ok(py3) = which::which("python3") {
                let py = std::path::Path::new("tools/ny_parser_mvp.py");
                if py.exists() {
                    let mut cmd = std::process::Command::new(&py3);
                    cmd.arg(py).arg(&tmp_path);
                    let timeout_ms: u64 = crate::config::env::ny_compiler_timeout_ms();
                    let out = match super::modes::common_util::io::spawn_with_timeout(cmd, timeout_ms) {
                        Ok(o) => o,
                        Err(e) => { eprintln!("[ny-compiler] python harness failed: {}", e); return false; }
                    };
                    if !out.timed_out {
                        if let Ok(s) = String::from_utf8(out.stdout) {
                            if let Some(line) = crate::runner::modes::common_util::selfhost::json::first_json_v0_line(&s) {
                                if std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1" {
                                    println!("{}", line);
                                    return true;
                                }
                                match super::json_v0_bridge::parse_json_v0_to_module(&line) {
                                    Ok(module) => {
                                        super::json_v0_bridge::maybe_dump_mir(&module);
                                        // Regular execution path
                                        // Prefer PyVM for selfhost pipeline (parity reference)
                                        if std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
                                            let code = match crate::runner::modes::common_util::pyvm::run_pyvm_harness(&module, "selfhost-py") {
                                                Ok(c) => c,
                                                Err(e) => { eprintln!("❌ PyVM error: {}", e); 1 }
                                            };
                                            println!("Result: {}", code);
                                            std::process::exit(code);
                                        }
                                        self.execute_mir_module(&module);
                                        return true;
                                    }
                                    Err(e) => {
                                        eprintln!("[ny-compiler] json parse error: {}", e);
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
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
                {
                    p.push("nyash_compiler.exe");
                }
                #[cfg(not(windows))]
                {
                    p.push("nyash_compiler");
                }
                if !p.exists() {
                    // Try PATH
                    if let Ok(w) = which::which("nyash_compiler") {
                        w
                    } else {
                        p
                    }
                } else {
                    p
                }
            };
            if exe_path.exists() {
                    let timeout_ms: u64 = crate::config::env::ny_compiler_timeout_ms();
                if let Some(module) = super::modes::common_util::selfhost_exe::exe_try_parse_json_v0(filename, timeout_ms) {
                    super::json_v0_bridge::maybe_dump_mir(&module);
                    let emit_only = std::env::var("NYASH_NY_COMPILER_EMIT_ONLY")
                        .unwrap_or_else(|_| "1".to_string())
                        == "1";
                    if emit_only { return false; }
                    // Prefer PyVM when requested (reference semantics)
                    if std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
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
                                crate::cli_v!("[Bridge] using PyVM (selfhost) → {}", mir_json_path.display());
                                let allow_top = crate::config::env::entry_allow_toplevel_main();
                                let entry = if module.functions.contains_key("Main.main") { "Main.main" }
                                            else if allow_top && module.functions.contains_key("main") { "main" }
                                            else if module.functions.contains_key("main") { eprintln!("[entry] Warning: using top-level 'main' without explicit allow; set NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 to silence."); "main" }
                                            else { "Main.main" };
                                let status = std::process::Command::new(py3)
                                    .args(["tools/pyvm_runner.py", "--in", &mir_json_path.display().to_string(), "--entry", entry])
                                    .status()
                                    .map_err(|e| format!("spawn pyvm: {}", e))
                                    .unwrap();
                                let code = status.code().unwrap_or(1);
                                println!("Result: {}", code);
                                std::process::exit(code);
                            }
                        }
                    }
                    self.execute_mir_module(&module);
                    return true;
                } else {
                    return false;
                }
            }
        }

        // Ny child compiler path via ENV 透過（昇格）: 親→子に --min-json/--stage3/child-args を渡して JSON を受け取る
        // 既定はOFF。ENV が立っている場合のみ有効化し、失敗時は従来の inline フォールバックに戻る。
        if std::env::var("NYASH_USE_NY_COMPILER").ok().as_deref() == Some("1") {
            let want_min_json = crate::config::env::ny_compiler_min_json();
            let want_stage3 = crate::config::env::ny_compiler_stage3();
            let want_read_tmp = crate::config::env::selfhost_read_tmp();
            let child_args_env = crate::config::env::ny_compiler_child_args();
            if want_min_json || want_stage3 || want_read_tmp || child_args_env.is_some() {
                use crate::runner::modes::common_util::selfhost::child::run_ny_program_capture_json;
                let exe = std::env::current_exe()
                    .unwrap_or_else(|_| std::path::PathBuf::from("target/release/nyash"));
                let program = std::path::Path::new("apps/selfhost-compiler/compiler.nyash");
                let mut extra: Vec<String> = Vec::new();
                // Pass delimiter then args to child compiler
                extra.push("--".to_string());
                if want_read_tmp { extra.push("--read-tmp".to_string()); }
                if want_min_json { extra.push("--min-json".to_string()); }
                if want_stage3 { extra.push("--stage3".to_string()); }
                // Dev trace: map NYASH_EMIT_TRACE=1 -> --emit-trace
                if std::env::var("NYASH_EMIT_TRACE").ok().as_deref() == Some("1") {
                    extra.push("--emit-trace".to_string());
                }
                // Optional lowering preference flags
                if std::env::var("NYASH_PREFER_CFG2").ok().as_deref() == Some("1") {
                    extra.push("--prefer-cfg2".to_string());
                } else if std::env::var("NYASH_PREFER_CFG").ok().as_deref() == Some("1") {
                    extra.push("--prefer-cfg".to_string());
                }
                if let Some(a) = child_args_env {
                    for tok in a.split_whitespace() { if !tok.is_empty() { extra.push(tok.to_string()); } }
                }
                let extra_refs: Vec<&str> = extra.iter().map(|s| s.as_str()).collect();
                let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS")
                    .ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
                if let Some(line) = run_ny_program_capture_json(
                    &exe,
                    program,
                    timeout_ms,
                    &extra_refs,
                    &["NYASH_USE_NY_COMPILER", "NYASH_CLI_VERBOSE"],
                    &[
                        ("NYASH_JSON_ONLY", "1"),
                        ("NYASH_ENABLE_USING", "1"),
                        ("NYASH_ALLOW_USING_FILE", "1"),
                        ("NYASH_USING_AST", "1"),
                    ],
                ) {
                    // Emit-only: print raw JSON and treat as handled
                    if std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1" {
                        println!("{}", line);
                        return true;
                    }
                    match super::json_v0_bridge::parse_json_v0_to_module(&line) {
                        Ok(module) => {
                            super::json_v0_bridge::maybe_dump_mir(&module);
                            // Prefer PyVM when requested
                            if std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
                                if let Some(code) = crate::runner::modes::common_util::selfhost::json::run_pyvm_module(&module, "selfhost-child") {
                                    println!("Result: {}", code);
                                    std::process::exit(code);
                                }
                            }
                            self.execute_mir_module(&module);
                            return true;
                        }
                        Err(e) => {
                            eprintln!("[ny-compiler] child json parse error: {}", e);
                            // fall through to inline fallback
                        }
                    }
                }
            }
        }

        // Fallback: inline VM run (embed source into a tiny wrapper that prints JSON)
        // This avoids CLI arg forwarding complexity and does not require FileBox.
        let mut json_line = String::new();
        {
            // Escape source for embedding as string literal
            let mut esc = String::with_capacity(code_ref.len());
            for ch in code_ref.chars() {
                match ch {
                    '\\' => esc.push_str("\\\\"),
                    '"' => esc.push_str("\\\""),
                    '\n' => esc.push_str("\n"),
                    '\r' => esc.push_str(""),
                    _ => esc.push(ch),
                }
            }
            let inline_path = std::path::Path::new("tmp").join("inline_selfhost_emit.nyash");
            let inline_code = format!(
                "include \"apps/selfhost-compiler/boxes/parser_box.hako\"\ninclude \"apps/selfhost-compiler/boxes/emitter_box.hako\"\nstatic box Main {{\n  main(args) {{\n    local s = \"{}\"\n    local p = new ParserBox()\n    p.stage3_enable(1)\n    local json = p.parse_program2(s)\n    local e = new EmitterBox()\n    json = e.emit_program(json, \"[]\")\n    print(json)\n    return 0\n  }}\n}}\n",
                esc
            );
            if let Err(e) = std::fs::write(&inline_path, inline_code) {
                eprintln!("[ny-compiler] write inline failed: {}", e);
                return false;
            }
            let exe = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("target/release/nyash"));
            let mut cmd = std::process::Command::new(exe);
            cmd.arg("--backend").arg("vm").arg(&inline_path);
            cmd.env_remove("NYASH_USE_NY_COMPILER");
            cmd.env_remove("NYASH_CLI_VERBOSE");
            cmd.env("NYASH_JSON_ONLY", "1");
            // Allow file-based using in the inline child code (it includes ParserBox/EmitterBox)
            cmd.env("NYASH_ENABLE_USING", "1");
            cmd.env("NYASH_ALLOW_USING_FILE", "1");
            cmd.env("NYASH_USING_AST", "1");
            let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2000);
            let out = match super::modes::common_util::io::spawn_with_timeout(cmd, timeout_ms) {
                Ok(o) => o,
                Err(e) => { eprintln!("[ny-compiler] spawn inline vm failed: {}", e); return false; }
            };
            if out.timed_out {
                let head = String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>();
                eprintln!("[ny-compiler] inline timeout after {} ms; stdout(head)='{}'", timeout_ms, head.replace('\n', "\\n"));
            }
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if let Some(line) = crate::runner::modes::common_util::selfhost::json::first_json_v0_line(&stdout) {
                json_line = line;
            }
        }
        if json_line.is_empty() {
            return false;
        }
        // Emit-only mode: print raw JSON line and return
        if std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1" {
            println!("{}", json_line);
            return true;
        }
        match super::json_v0_bridge::parse_json_v0_to_module(&json_line) {
            Ok(module) => {
                super::json_v0_bridge::maybe_dump_mir(&module);
                // Policy update: prefer Rust VM by default. Use PyVM only when explicitly requested.
                if std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1") {
                    if let Some(code) = crate::runner::modes::common_util::selfhost::json::run_pyvm_module(&module, "selfhost") {
                        println!("Result: {}", code);
                        std::process::exit(code);
                    }
                }
                self.execute_mir_module(&module);
                true
            }
            Err(e) => {
                eprintln!("❌ JSON v0 bridge error: {}", e);
                false
            }
        }
    }
}
