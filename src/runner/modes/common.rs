use super::super::NyashRunner;
use crate::runner::json_v0_bridge;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter};
// Use the library crate's plugin init module rather than the bin crate root
use nyash_rust::runner_plugin_init;
use std::{fs, process};
use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::thread::sleep;
use crate::runner::pipeline::{suggest_in_base, resolve_using_target};
use crate::runner::trace::cli_verbose;
use crate::cli_v;

// (moved) suggest_in_base is now in runner/pipeline.rs

impl NyashRunner {
    /// File-mode dispatcher (thin wrapper around backend/mode selection)
    #[allow(dead_code)]
    pub(crate) fn run_file_legacy(&self, filename: &str) {
        // Phase-15.3: Ny compiler MVP (Ny -> JSON v0) behind env gate
        if std::env::var("NYASH_USE_NY_COMPILER").ok().as_deref() == Some("1") {
            if self.try_run_selfhost_pipeline(filename) {
                return;
            } else if crate::config::env::cli_verbose() {
                eprintln!("[ny-compiler] fallback to default path (MVP unavailable for this input)");
            }
        }
        // Direct v0 bridge when requested via CLI/env
        let use_ny_parser = self.config.parser_ny || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
        if use_ny_parser {
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            match json_v0_bridge::parse_source_v0_to_module(&code) {
                Ok(module) => {
                    if crate::config::env::cli_verbose() {
                        println!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                    }
                    self.execute_mir_module(&module);
                    return;
                }
                Err(e) => { eprintln!("❌ Direct bridge parse error: {}", e); process::exit(1); }
            }
        }
        // AST dump mode
        if self.config.dump_ast {
            println!("🧠 Nyash AST Dump - Processing file: {}", filename);
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            let ast = match NyashParser::parse_from_string(&code) {
                Ok(ast) => ast,
                Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
            };
            println!("{:#?}", ast);
            return;
        }

        // MIR dump/verify
        if self.config.dump_mir || self.config.verify_mir {
            if crate::config::env::cli_verbose() {
                println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
            }
            self.execute_mir_mode(filename);
            return;
        }

        // WASM / AOT (feature-gated)
        if self.config.compile_wasm {
            #[cfg(feature = "wasm-backend")]
            { self.execute_wasm_mode(filename); return; }
            #[cfg(not(feature = "wasm-backend"))]
            { eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend"); process::exit(1); }
        }
        if self.config.compile_native {
            #[cfg(feature = "cranelift-jit")]
            { self.execute_aot_mode(filename); return; }
            #[cfg(not(feature = "cranelift-jit"))]
            { eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit"); process::exit(1); }
        }

        // Backend selection
        match self.config.backend.as_str() {
            "mir" => {
                if crate::config::env::cli_verbose() {
                    println!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
                }
                self.execute_mir_interpreter_mode(filename);
            }
            "vm" => {
                if crate::config::env::cli_verbose() {
                    println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
                }
                self.execute_vm_mode(filename);
            }
            "cranelift" => {
                #[cfg(feature = "cranelift-jit")]
                {
                    if cli_verbose() { println!("⚙️  Nyash Cranelift JIT - Executing file: {}", filename); }
                    self.execute_cranelift_mode(filename);
                }
                #[cfg(not(feature = "cranelift-jit"))]
                {
                    eprintln!("❌ Cranelift backend not available. Please rebuild with: cargo build --features cranelift-jit");
                    process::exit(1);
                }
            }
            "llvm" => {
                if cli_verbose() { println!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename); }
                self.execute_llvm_mode(filename);
            }
            _ => {
                if cli_verbose() {
                    println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
                    if let Some(fuel) = self.config.debug_fuel {
                        println!("🔥 Debug fuel limit: {} iterations", fuel);
                    } else {
                        println!("🔥 Debug fuel limit: unlimited");
                    }
                    println!("====================================================");
                }
                self.execute_nyash_file(filename);
            }
        }
    }

    /// Helper: run PyVM harness over a MIR module, returning the exit code
    fn run_pyvm_harness(&self, module: &nyash_rust::mir::MirModule, tag: &str) -> Result<i32, String> {
        super::common_util::pyvm::run_pyvm_harness(module, tag)
    }

    /// Helper: try external selfhost compiler EXE to parse Ny -> JSON v0 and return MIR module
    /// Returns Some(module) on success, None on failure (timeout/invalid output/missing exe)
    fn exe_try_parse_json_v0(&self, filename: &str, timeout_ms: u64) -> Option<nyash_rust::mir::MirModule> {
        super::common_util::selfhost_exe::exe_try_parse_json_v0(filename, timeout_ms)
    }

    /// Phase-15.3: Attempt Ny compiler pipeline (Ny -> JSON v0 via Ny program), then execute MIR
    pub(crate) fn try_run_ny_compiler_pipeline(&self, filename: &str) -> bool {
        // Delegate to centralized selfhost pipeline to avoid drift
        return self.try_run_selfhost_pipeline(filename);
        use std::io::Write;
        // Read input source
        let code = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => { eprintln!("[ny-compiler] read error: {}", e); return false; }
        };
        // Optional Phase-15: strip `using` lines and register modules (same policy as execute_nyash_file)
        let enable_using = crate::config::env::enable_using();
        let mut code_ref: std::borrow::Cow<'_, str> = std::borrow::Cow::Borrowed(&code);
        if enable_using {
            let mut out = String::with_capacity(code.len());
            let mut used_names: Vec<(String, Option<String>)> = Vec::new();
            for line in code.lines() {
                let t = line.trim_start();
                if t.starts_with("using ") {
                    cli_v!("[using] stripped(line→selfhost): {}", line);
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
        let use_tmp_only = crate::config::env::ny_compiler_use_tmp_only();
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
        if crate::config::env::use_ny_compiler_exe() {
            // Resolve parser EXE path
            let exe_path = if let Some(p) = crate::config::env::ny_compiler_exe_path() {
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
                if crate::config::env::ny_compiler_min_json() { cmd.arg("--min-json"); }
                if crate::config::env::selfhost_read_tmp() { cmd.arg("--read-tmp"); }
                if let Some(raw) = crate::config::env::ny_compiler_child_args() { for tok in raw.split_whitespace() { cmd.arg(tok); } }
                let timeout_ms: u64 = crate::config::env::ny_compiler_timeout_ms();
                let out = match super::common_util::io::spawn_with_timeout(cmd, timeout_ms) {
                    Ok(o) => o,
                    Err(e) => { eprintln!("[ny-compiler] exe spawn failed: {}", e); return false; }
                };
                if out.timed_out {
                    let head = String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>();
                    eprintln!("[ny-compiler] exe timeout after {} ms; stdout(head)='{}'", timeout_ms, head.replace('\n', "\\n"));
                    return false;
                }
                let stdout = match String::from_utf8(out.stdout) { Ok(s) => s, Err(_) => String::new() };
                let mut json_line = String::new();
                for line in stdout.lines() { let t = line.trim(); if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") { json_line = t.to_string(); break; } }
                if json_line.is_empty() {
                    if crate::config::env::cli_verbose() {
                        let head: String = stdout.chars().take(200).collect();
                        let errh: String = String::from_utf8_lossy(&out.stderr).chars().take(200).collect();
                        eprintln!("[ny-compiler] exe produced no JSON; stdout(head)='{}' stderr(head)='{}'", head.replace('\n', "\\n"), errh.replace('\n', "\\n"));
                    }
                    return false;
                }
                // Parse JSON v0 → MIR module
                match json_v0_bridge::parse_json_v0_to_module(&json_line) {
                    Ok(module) => {
                        println!("🚀 Ny compiler EXE path (ny→json_v0) ON");
                        json_v0_bridge::maybe_dump_mir(&module);
                        let emit_only = crate::config::env::ny_compiler_emit_only();
                        if emit_only {
                            return false;
                        } else {
                            // Prefer PyVM when requested (reference semantics), regardless of BoxCall presence
                            let prefer_pyvm = crate::config::env::vm_use_py();
                            if prefer_pyvm {
                                if let Ok(py3) = which::which("python3") {
                                    let runner = std::path::Path::new("tools/pyvm_runner.py");
                                    if runner.exists() {
                                        let tmp_dir = std::path::Path::new("tmp");
                                        let _ = std::fs::create_dir_all(tmp_dir);
                                        let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
                                        if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(&module, &mir_json_path) {
                                            eprintln!("❌ PyVM MIR JSON emit error: {}", e);
                                            return true; // prevent double-run fallback
                                        }
                                        let code = self.run_pyvm_harness(&module, "exe").unwrap_or(1);
                                        println!("Result: {}", code);
                                        std::process::exit(code);
                                    } else {
                                        eprintln!("❌ PyVM runner not found: {}", runner.display());
                                        std::process::exit(1);
                                    }
                                } else {
                                    eprintln!("❌ python3 not found in PATH. Install Python 3 to use PyVM.");
                                    std::process::exit(1);
                                }
                            }
                            // Default: execute via built-in MIR interpreter
                            self.execute_mir_module(&module);
                            return true;
                        }
                    }
                    Err(e) => { eprintln!("[ny-compiler] JSON parse failed (exe): {}", e); return false; }
                }
            } else {
                if crate::config::env::cli_verbose() { eprintln!("[ny-compiler] exe not found at {}", exe_path.display()); }
            }
        }

        // Locate current exe to invoke Ny VM for the Ny parser program
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => { eprintln!("[ny-compiler] current_exe failed: {}", e); return false; }
        };
        // Select selfhost compiler entry
        //   NYASH_NY_COMPILER_PREF=legacy|new|auto (default auto: prefer new when exists)
        let cand_new = std::path::Path::new("apps/selfhost-compiler/compiler.nyash");
        let cand_old = std::path::Path::new("apps/selfhost/parser/ny_parser_v0/main.nyash");
        let pref = std::env::var("NYASH_NY_COMPILER_PREF").ok();
        let parser_prog = match pref.as_deref() {
            Some("legacy") => cand_old,
            Some("new") => cand_new,
            _ => if cand_new.exists() { cand_new } else { cand_old },
        };
        if !parser_prog.exists() { eprintln!("[ny-compiler] compiler program not found: {}", parser_prog.display()); return false; }
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("--backend").arg("vm").arg(parser_prog);
        // Gate: pass script args to child parser program
        // - NYASH_NY_COMPILER_MIN_JSON=1 → "-- --min-json"
        // - NYASH_SELFHOST_READ_TMP=1   → "-- --read-tmp"
        // - NYASH_NY_COMPILER_CHILD_ARGS: additional raw args (split by whitespace)
        let min_json = std::env::var("NYASH_NY_COMPILER_MIN_JSON").ok().unwrap_or_else(|| "0".to_string());
        if min_json == "1" { cmd.arg("--").arg("--min-json"); }
        if std::env::var("NYASH_SELFHOST_READ_TMP").ok().as_deref() == Some("1") {
            cmd.arg("--").arg("--read-tmp");
        }
        if let Ok(raw) = std::env::var("NYASH_NY_COMPILER_CHILD_ARGS") {
            for tok in raw.split_whitespace() { cmd.arg(tok); }
        }
        // Propagate minimal env; disable plugins to reduce noise
        cmd.env_remove("NYASH_USE_NY_COMPILER");
        cmd.env_remove("NYASH_CLI_VERBOSE");
        // Suppress parent runner's result printing in child
        cmd.env("NYASH_JSON_ONLY", "1");
        // Propagate optional gates to child (if present)
        if let Ok(v) = std::env::var("NYASH_JSON_INCLUDE_USINGS") { cmd.env("NYASH_JSON_INCLUDE_USINGS", v); }
        if let Ok(v) = std::env::var("NYASH_ENABLE_USING") { cmd.env("NYASH_ENABLE_USING", v); }
        // Child timeout guard (Hotfix for potential infinite loop in child Ny parser)
        // Config: NYASH_NY_COMPILER_TIMEOUT_MS (default 2000ms)
        let timeout_ms: u64 = std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2000);
        let out = match super::common_util::io::spawn_with_timeout(cmd, timeout_ms) {
            Ok(o) => o,
            Err(e) => { eprintln!("[ny-compiler] spawn failed: {}", e); return false; }
        };
        if out.timed_out {
            let head = String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>();
            eprintln!("[ny-compiler] child timeout after {} ms; stdout(head)='{}'", timeout_ms, head.replace('\n', "\\n"));
        }
        let stdout = match String::from_utf8(out.stdout.clone()) { Ok(s) => s, Err(_) => String::new() };
        if timed_out {
            // Fall back path will be taken below when json_line remains empty
        } else if let Ok(s) = String::from_utf8(err_buf.clone()) {
            // If the child exited non-zero and printed stderr, surface it and fallback
            // We cannot easily access ExitStatus here after try_wait loop; rely on JSON detection path.
            if s.trim().len() > 0 && crate::config::env::cli_verbose() {
                eprintln!("[ny-compiler] parser stderr:\n{}", s);
            }
        }
        let mut json_line = String::new();
        for line in stdout.lines() {
            let t = line.trim();
            if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") { json_line = t.to_string(); break; }
        }
        if json_line.is_empty() {
        // Fallback: try Python MVP parser to produce JSON v0 from the same tmp source (unless skipped).
        if crate::config::env::cli_verbose() {
            let head: String = stdout.chars().take(200).collect();
            cli_v!("[ny-compiler] JSON not found in child stdout (head): {}", head.replace('\\n', "\\n"));
            cli_v!("[ny-compiler] falling back to tools/ny_parser_mvp.py for this input");
        }
        if std::env::var("NYASH_NY_COMPILER_SKIP_PY").ok().as_deref() != Some("1") {
        let py = which::which("python3").ok();
        if let Some(py3) = py {
            let script = std::path::Path::new("tools/ny_parser_mvp.py");
            if script.exists() {
                let out2 = std::process::Command::new(py3)
                    .arg(script)
                    .arg(tmp_path.as_os_str())
                    .output();
                match out2 {
                    Ok(o2) if o2.status.success() => {
                        if let Ok(s2) = String::from_utf8(o2.stdout) {
                            // pick the first JSON-ish line
                            for line in s2.lines() {
                                let t = line.trim();
                                if t.starts_with('{') && t.contains("\"version\"") && t.contains("\"kind\"") { json_line = t.to_string(); break; }
                            }
                        }
                    }
                    Ok(o2) => {
                        let msg = String::from_utf8_lossy(&o2.stderr);
                        eprintln!("[ny-compiler] python parser failed: {}", msg);
                    }
                    Err(e2) => {
                        eprintln!("[ny-compiler] spawn python3 failed: {}", e2);
                    }
                }
            }
        } }
            if json_line.is_empty() { return false; }
        }
        // Parse JSON v0 → MIR module
        match json_v0_bridge::parse_json_v0_to_module(&json_line) {
                    Ok(module) => {
                let emit_only_default = "1".to_string();
                let emit_only = if emit_only_default == "1" { true } else { crate::config::env::ny_compiler_emit_only() };
                println!("🚀 Ny compiler MVP (ny→json_v0) path ON");
                json_v0_bridge::maybe_dump_mir(&module);
                if emit_only {
                    // Do not execute; fall back to default path to keep final Result unaffected (Stage‑1 policy)
                    false
                } else {
                    // Prefer PyVM when requested (reference semantics)
                    let prefer_pyvm = crate::config::env::vm_use_py();
                    if prefer_pyvm {
                        if let Ok(py3) = which::which("python3") {
                            let runner = std::path::Path::new("tools/pyvm_runner.py");
                            if runner.exists() {
                                let tmp_dir = std::path::Path::new("tmp");
                                let _ = std::fs::create_dir_all(tmp_dir);
                                let mir_json_path = tmp_dir.join("nyash_pyvm_mir.json");
                                if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(&module, &mir_json_path) {
                                    eprintln!("❌ PyVM MIR JSON emit error: {}", e);
                                    return true; // prevent double-run fallback
                                }
                                if crate::config::env::cli_verbose() {
                                    eprintln!("[ny-compiler] using PyVM (mvp) → {}", mir_json_path.display());
                                }
                                // Determine entry function hint (prefer Main.main if present)
                                let entry = if module.functions.contains_key("Main.main") { "Main.main" }
                                            else if module.functions.contains_key("main") { "main" } else { "Main.main" };
                                let code = self.run_pyvm_harness(&module, "mvp").unwrap_or(1);
                                println!("Result: {}", code);
                                std::process::exit(code);
                            } else {
                                eprintln!("❌ PyVM runner not found: {}", runner.display());
                                std::process::exit(1);
                            }
                        } else {
                            eprintln!("❌ python3 not found in PATH. Install Python 3 to use PyVM.");
                            std::process::exit(1);
                        }
                    }
                    // Default: execute via MIR interpreter
                    self.execute_mir_module(&module);
                    true
                }
            }
            Err(e) => {
                eprintln!("[ny-compiler] JSON parse failed: {}", e);
                false
            }
        }
    }

    /// Execute Nyash file with interpreter (common helper)
    pub(crate) fn execute_nyash_file(&self, filename: &str) {
        let quiet_pipe = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1");
        // Ensure plugin host and provider mappings are initialized (idempotent)
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            // Call via lib crate to avoid referring to the bin crate root
            runner_plugin_init::init_bid_plugins();
        }
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        if crate::config::env::cli_verbose() && !quiet_pipe {
            println!("📝 File contents:\n{}", code);
            println!("\n🚀 Parsing and executing...\n");
        }

        // Optional Phase-15: strip `using` lines (gate) for minimal acceptance
        let mut code_ref: &str = &code;
        let enable_using = crate::config::env::enable_using();
        let cleaned_code_owned;
        if enable_using {
            let mut out = String::with_capacity(code.len());
            let mut used_names: Vec<(String, Option<String>)> = Vec::new();
            for line in code.lines() {
                let t = line.trim_start();
                if t.starts_with("using ") {
                    // Skip `using ns` or `using ns as alias` lines (MVP)
                    if crate::config::env::cli_verbose() {
                        eprintln!("[using] stripped line: {}", line);
                    }
                    // Parse namespace or path and optional alias
                    let rest0 = t.strip_prefix("using ").unwrap().trim();
                    // allow trailing semicolon
                    let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                    // Split alias
                    let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                        (rest0[..pos].trim().to_string(), Some(rest0[pos+4..].trim().to_string()))
                    } else { (rest0.to_string(), None) };
                    // If quoted or looks like relative/absolute path, treat as path; else as namespace
                    let is_path = target.starts_with('"') || target.starts_with("./") || target.starts_with('/') || target.ends_with(".nyash");
                    if is_path {
                        let mut path = target.trim_matches('"').to_string();
                        // existence check and strict handling
                        let missing = !std::path::Path::new(&path).exists();
                        if missing {
                            if std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1") {
                                eprintln!("❌ using: path not found: {}", path);
                                std::process::exit(1);
                            } else if crate::config::env::cli_verbose() {
                                eprintln!("[using] path not found (continuing): {}", path);
                            }
                        }
                        // choose alias or derive from filename stem
                        let name = alias.clone().unwrap_or_else(|| {
                            std::path::Path::new(&path)
                                .file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string()
                        });
                        // register alias only (path-backed)
                        used_names.push((name, Some(path)));
                    } else {
                        used_names.push((target, alias));
                    }
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            cleaned_code_owned = out;
            code_ref = &cleaned_code_owned;

            // Register modules with resolver (aliases/modules/paths)
            let using_ctx = self.init_using_context();
            let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
            let verbose = crate::config::env::cli_verbose();
            let ctx_dir = std::path::Path::new(filename).parent();
            for (ns_or_alias, alias_or_path) in used_names {
                if let Some(path) = alias_or_path {
                    let sb = crate::box_trait::StringBox::new(path);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                } else {
                    match resolve_using_target(&ns_or_alias, false, &using_ctx.pending_modules, &using_ctx.using_paths, &using_ctx.aliases, ctx_dir, strict, verbose) {
                        Ok(value) => {
                            let sb = crate::box_trait::StringBox::new(value);
                            crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                        }
                        Err(e) => {
                            eprintln!("❌ using: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        // Parse the code with debug fuel limit
        eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", self.config.debug_fuel);
        let ast = match NyashParser::parse_from_string_with_fuel(code_ref, self.config.debug_fuel) {
            Ok(ast) => { eprintln!("🔍 DEBUG: Parse completed, AST created"); ast },
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        // Stage-0: import loader (top-level only) — resolve path and register in modules registry
        if let nyash_rust::ast::ASTNode::Program { statements, .. } = &ast {
            for st in statements {
                if let nyash_rust::ast::ASTNode::ImportStatement { path, alias, .. } = st {
                    // resolve path relative to current file if not absolute
                    let mut p = std::path::PathBuf::from(path);
                    if p.is_relative() {
                        if let Some(dir) = std::path::Path::new(filename).parent() {
                            p = dir.join(&p);
                        }
                    }
                    let exists = p.exists();
                    if !exists {
                        if std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1") {
                            eprintln!("❌ import: path not found: {} (from {})", p.display(), filename);
                            process::exit(1);
                        } else if crate::config::env::cli_verbose() || std::env::var("NYASH_IMPORT_TRACE").ok().as_deref() == Some("1") {
                            eprintln!("[import] path not found (continuing): {}", p.display());
                        }
                    }
                    let key = if let Some(a) = alias { a.clone() } else {
                        std::path::Path::new(path)
                            .file_stem().and_then(|s| s.to_str())
                            .unwrap_or(path)
                            .to_string()
                    };
                    let value = if exists { p.to_string_lossy().to_string() } else { path.clone() };
                    let sb = nyash_rust::box_trait::StringBox::new(value);
                    nyash_rust::runtime::modules_registry::set(key, Box::new(sb));
                }
            }
        }

        if crate::config::env::cli_verbose() && !quiet_pipe {
            if crate::config::env::cli_verbose() { println!("✅ Parse successful!"); }
        }

        // Execute the AST
        let mut interpreter = NyashInterpreter::new();
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                if crate::config::env::cli_verbose() && !quiet_pipe {
                    if crate::config::env::cli_verbose() { println!("✅ Execution completed successfully!"); }
                }
                // Normalize display via semantics: prefer numeric, then string, then fallback
                let disp = {
                    // Special-case: plugin IntegerBox → call .get to fetch numeric value
                    if let Some(p) = result.as_any().downcast_ref::<nyash_rust::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if p.box_type == "IntegerBox" {
                            // Scope the lock strictly to this block
                            let fetched = {
                                let host = nyash_rust::runtime::get_global_plugin_host();
                                let res = if let Ok(ro) = host.read() {
                                    if let Ok(Some(vb)) = ro.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(ib) = vb.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            Some(ib.value.to_string())
                                        } else {
                                            Some(vb.to_string_box().value)
                                        }
                                    } else { None }
                                } else { None };
                                res
                            };
                            if let Some(s) = fetched { s } else {
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                    .map(|i| i.to_string())
                                    .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                    .unwrap_or_else(|| result.to_string_box().value)
                            }
                        } else {
                            nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                .map(|i| i.to_string())
                                .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                .unwrap_or_else(|| result.to_string_box().value)
                        }
                    } else {
                        nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            .map(|i| i.to_string())
                            .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                            .unwrap_or_else(|| result.to_string_box().value)
                    }
                };
                if !quiet_pipe { println!("Result: {}", disp); }
            },
            Err(e) => {
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }
}
