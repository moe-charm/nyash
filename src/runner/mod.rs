/*!
 * Execution Runner Module - Nyash File and Mode Execution Coordinator
 * 
 * This module handles all execution logic, backend selection, and mode coordination,
 * separated from CLI parsing and the main entry point.
 */

use nyash_rust::cli::CliConfig;
use nyash_rust::{
    box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox, AddBox, BoxCore},
    tokenizer::{NyashTokenizer},
    ast::ASTNode,
    parser::NyashParser,
    interpreter::NyashInterpreter,
    mir::{MirCompiler, MirPrinter, MirInstruction},
    backend::VM,
};
use nyash_rust::runtime::{NyashRuntime, NyashRuntimeBuilder};
use nyash_rust::interpreter::SharedState;
use nyash_rust::box_factory::user_defined::UserDefinedBoxFactory;
use nyash_rust::core::model::BoxDeclaration as CoreBoxDecl;
use std::sync::Arc;

#[cfg(feature = "wasm-backend")]
use nyash_rust::backend::{wasm::WasmBackend, aot::AotBackend};

#[cfg(feature = "llvm")]
use nyash_rust::backend::{llvm_compile_and_execute};
use std::{fs, process};
mod modes;
mod demos;
mod json_v0_bridge;

// v2 plugin system imports
use nyash_rust::runtime;
use nyash_rust::runner_plugin_init;
use std::path::PathBuf;

/// Resolve a using target according to priority: modules > relative > using-paths
/// Returns Ok(resolved_path_or_token). On strict mode, ambiguous matches cause error.
fn resolve_using_target(
    tgt: &str,
    is_path: bool,
    modules: &[(String, String)],
    using_paths: &[String],
    context_dir: Option<&std::path::Path>,
    strict: bool,
    verbose: bool,
) -> Result<String, String> {
    if is_path { return Ok(tgt.to_string()); }
    // 1) modules mapping
    if let Some((_, p)) = modules.iter().find(|(n, _)| n == tgt) { return Ok(p.clone()); }
    // 2) build candidate list: relative then using-paths
    let rel = tgt.replace('.', "/") + ".nyash";
    let mut cand: Vec<String> = Vec::new();
    if let Some(dir) = context_dir { let c = dir.join(&rel); if c.exists() { cand.push(c.to_string_lossy().to_string()); } }
    for base in using_paths {
        let c = std::path::Path::new(base).join(&rel);
        if c.exists() { cand.push(c.to_string_lossy().to_string()); }
    }
    if cand.is_empty() {
        if verbose { eprintln!("[using] unresolved '{}' (searched: rel+paths)", tgt); }
        return Ok(tgt.to_string());
    }
    if cand.len() > 1 && strict {
        return Err(format!("ambiguous using '{}': {}", tgt, cand.join(", ")));
    }
    Ok(cand.remove(0))
}

/// Main execution coordinator
pub struct NyashRunner {
    config: CliConfig,
}

/// Minimal task runner: read nyash.toml [env] and [tasks], run the named task via shell
fn run_named_task(name: &str) -> Result<(), String> {
    let cfg_path = "nyash.toml";
    let text = fs::read_to_string(cfg_path).map_err(|e| format!("read {}: {}", cfg_path, e))?;
    let doc = toml::from_str::<toml::Value>(&text).map_err(|e| format!("parse {}: {}", cfg_path, e))?;
    // Apply [env]
    if let Some(env_tbl) = doc.get("env").and_then(|v| v.as_table()) {
        for (k, v) in env_tbl.iter() {
            if let Some(s) = v.as_str() { std::env::set_var(k, s); }
        }
    }
    // Lookup [tasks]
    let tasks = doc.get("tasks").and_then(|v| v.as_table()).ok_or("[tasks] not found in nyash.toml")?;
    let cmd = tasks.get(name).and_then(|v| v.as_str()).ok_or_else(|| format!("task '{}' not found", name))?;
    // Basic variable substitution
    let root = std::env::current_dir().unwrap_or(PathBuf::from(".")).display().to_string();
    let cmd = cmd.replace("{root}", &root);
    // Run via shell
    #[cfg(windows)]
    let status = std::process::Command::new("cmd").args(["/C", &cmd]).status().map_err(|e| e.to_string())?;
    #[cfg(not(windows))]
    let status = std::process::Command::new("sh").arg("-lc").arg(&cmd).status().map_err(|e| e.to_string())?;
    if !status.success() { return Err(format!("task '{}' failed with status {:?}", name, status.code())); }
    Ok(())
}

impl NyashRunner {
    /// Create a new runner with the given configuration
    pub fn new(config: CliConfig) -> Self {
        Self { config }
    }

    /// Run Nyash based on the configuration
    pub fn run(&self) {
        // Build system (MVP): nyash --build <nyash.toml>
        if let Some(cfg_path) = self.config.build_path.clone() {
            if let Err(e) = self.run_build_mvp(&cfg_path) {
                eprintln!("❌ build error: {}", e);
                std::process::exit(1);
            }
            return;
        }
        // Using/module overrides via env only (MVP)
        // Prepare shared accumulators for script/env processing
        let mut using_paths: Vec<String> = Vec::new();
        let mut pending_modules: Vec<(String, String)> = Vec::new();
        // Using-paths from env, with defaults
        if let Ok(p) = std::env::var("NYASH_USING_PATH") {
            for s in p.split(':') { let s=s.trim(); if !s.is_empty() { using_paths.push(s.to_string()); } }
        }
        if using_paths.is_empty() { using_paths.extend(["apps","lib","."].into_iter().map(|s| s.to_string())); }
        // Modules mapping from env (e.g., FOO=path)
        if let Ok(ms) = std::env::var("NYASH_MODULES") {
            for ent in ms.split(',') {
                if let Some((k,v)) = ent.split_once('=') {
                    let k=k.trim(); let v=v.trim();
                    if !k.is_empty() && !v.is_empty() { pending_modules.push((k.to_string(), v.to_string())); }
                }
            }
        }
        for (ns, path) in pending_modules.iter() {
            let sb = crate::box_trait::StringBox::new(path.clone());
            crate::runtime::modules_registry::set(ns.clone(), Box::new(sb));
        }
        // Stage-1: Optional dependency tree bridge (log-only)
        if let Ok(dep_path) = std::env::var("NYASH_DEPS_JSON") {
            match std::fs::read_to_string(&dep_path) {
                Ok(s) => {
                    let bytes = s.as_bytes().len();
                    // Try to extract quick hints without failing
                    let mut root_info = String::new();
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                        if let Some(r) = v.get("root_path").and_then(|x| x.as_str()) { root_info = format!(" root='{}'", r); }
                    }
                    eprintln!("[deps] loaded {} bytes from{} {}", bytes, if root_info.is_empty(){""} else {":"}, root_info);
                }
                Err(e) => {
                    eprintln!("[deps] read error: {}", e);
                }
            }
        }

        // Phase-15: JSON IR v0 bridge (stdin/file)
        if self.config.ny_parser_pipe || self.config.json_file.is_some() {
            let json = if let Some(path) = &self.config.json_file {
                match std::fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(e) => { eprintln!("❌ json-file read error: {}", e); std::process::exit(1); }
                }
            } else {
                use std::io::Read;
                let mut buf = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                    eprintln!("❌ stdin read error: {}", e); std::process::exit(1);
                }
                buf
            };
            match json_v0_bridge::parse_json_v0_to_module(&json) {
                Ok(module) => {
                    // Optional dump via env verbose
                    json_v0_bridge::maybe_dump_mir(&module);
                    // Execute via MIR interpreter
                    self.execute_mir_module(&module);
                    return;
                }
                Err(e) => {
                    eprintln!("❌ JSON v0 bridge error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        // Run named task from nyash.toml (MVP)
        if let Some(task) = self.config.run_task.clone() {
            if let Err(e) = run_named_task(&task) {
                eprintln!("❌ Task error: {}", e);
                process::exit(1);
            }
            return;
        }
        // Verbose CLI flag maps to env for downstream helpers/scripts
        if self.config.cli_verbose { std::env::set_var("NYASH_CLI_VERBOSE", "1"); }
        // Script-level env directives (special comments) — parse early
        // Supported:
        //   // @env KEY=VALUE
        //   // @jit-debug           (preset: exec, threshold=1, events+trace)
        //   // @plugin-builtins     (NYASH_USE_PLUGIN_BUILTINS=1)
        if let Some(ref filename) = self.config.file {
            if let Ok(code) = fs::read_to_string(filename) {
                // Scan first 128 lines for directives
                for (i, line) in code.lines().take(128).enumerate() {
                    let l = line.trim();
                    if !(l.starts_with("//") || l.starts_with("#!") || l.is_empty()) {
                        // Stop early at first non-comment line to avoid scanning full file
                        if i > 0 { break; }
                    }
                    // Shebang with envs: handled by shell normally; keep placeholder
                    if let Some(rest) = l.strip_prefix("//") { let rest = rest.trim();
                        if let Some(dir) = rest.strip_prefix("@env ") {
                            if let Some((k,v)) = dir.split_once('=') {
                                let key = k.trim(); let val = v.trim();
                                if !key.is_empty() { std::env::set_var(key, val); }
                            }
                        } else if rest == "@jit-debug" {
                            std::env::set_var("NYASH_JIT_EXEC", "1");
                            std::env::set_var("NYASH_JIT_THRESHOLD", "1");
                            std::env::set_var("NYASH_JIT_EVENTS", "1");
                            std::env::set_var("NYASH_JIT_EVENTS_COMPILE", "1");
                            std::env::set_var("NYASH_JIT_EVENTS_RUNTIME", "1");
                            std::env::set_var("NYASH_JIT_SHIM_TRACE", "1");
                        } else if rest == "@plugin-builtins" {
                            std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
                        } else if rest == "@jit-strict" {
                            std::env::set_var("NYASH_JIT_STRICT", "1");
                            std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
                            // In strict mode, default to JIT-only (no VM fallback)
                            if std::env::var("NYASH_JIT_ONLY").ok().is_none() { std::env::set_var("NYASH_JIT_ONLY", "1"); }
                        }
                    }
                }

                // Env overrides for using rules
                if let Ok(paths) = std::env::var("NYASH_USING_PATH") {
                    for p in paths.split(':') { let p = p.trim(); if !p.is_empty() { using_paths.push(p.to_string()); } }
                }
                if let Ok(mods) = std::env::var("NYASH_MODULES") {
                    for ent in mods.split(',') {
                        if let Some((k,v)) = ent.split_once('=') {
                            let k = k.trim(); let v = v.trim();
                            if !k.is_empty() && !v.is_empty() { pending_modules.push((k.to_string(), v.to_string())); }
                        }
                    }
                }

                // Apply pending modules to registry as StringBox (path or ns token)
                for (ns, path) in pending_modules.iter() {
                    let sb = nyash_rust::box_trait::StringBox::new(path.clone());
                    nyash_rust::runtime::modules_registry::set(ns.to_string(), Box::new(sb));
                }
                // Resolve pending using with clear precedence and ambiguity handling
                let pending_using: Vec<(String, Option<String>)> = Vec::new();
                let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
                let verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
                let ctx = std::path::Path::new(filename).parent();
                for (ns, alias) in pending_using.iter() {
                    let value = match resolve_using_target(ns, false, &pending_modules, &using_paths, ctx, strict, verbose) {
                        Ok(v) => v,
                        Err(e) => { eprintln!("❌ using: {}", e); std::process::exit(1); }
                    };
                    let sb = nyash_rust::box_trait::StringBox::new(value.clone());
                    nyash_rust::runtime::modules_registry::set(ns.to_string(), Box::new(sb));
                    if let Some(a) = alias {
                        let sb2 = nyash_rust::box_trait::StringBox::new(value);
                        nyash_rust::runtime::modules_registry::set(a.to_string(), Box::new(sb2));
                    }
                }
            }
        }

        // If strict mode requested via env, ensure handle-only shim behavior is enabled
        if std::env::var("NYASH_JIT_STRICT").ok().as_deref() == Some("1") {
            if std::env::var("NYASH_JIT_ARGS_HANDLE_ONLY").ok().is_none() {
                std::env::set_var("NYASH_JIT_ARGS_HANDLE_ONLY", "1");
            }
            // Enforce JIT-only by default in strict mode unless explicitly overridden
            if std::env::var("NYASH_JIT_ONLY").ok().is_none() {
                std::env::set_var("NYASH_JIT_ONLY", "1");
            }
        }

        // 🏭 Phase 9.78b: Initialize unified registry
        runtime::init_global_unified_registry();
        
        // Try to initialize BID plugins from nyash.toml (best-effort)
        // Allow disabling during snapshot/CI via NYASH_DISABLE_PLUGINS=1
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            runner_plugin_init::init_bid_plugins();
        }
        // Allow interpreter to create plugin-backed boxes via unified registry
        // Opt-in by default for FileBox/TOMLBox which are required by ny-config and similar tools.
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().is_none() {
            std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
        }
        // Merge FileBox,TOMLBox with defaults if present
        let mut override_types: Vec<String> = if let Ok(list) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
            list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        } else {
            vec!["ArrayBox".into(), "MapBox".into()]
        };
        for t in ["FileBox", "TOMLBox"] { if !override_types.iter().any(|x| x==t) { override_types.push(t.into()); } }
        std::env::set_var("NYASH_PLUGIN_OVERRIDE_TYPES", override_types.join(","));

        // Opt-in: load Ny script plugins listed in nyash.toml [ny_plugins]
        if self.config.load_ny_plugins || std::env::var("NYASH_LOAD_NY_PLUGINS").ok().as_deref() == Some("1") {
            if let Ok(text) = std::fs::read_to_string("nyash.toml") {
                if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                    if let Some(np) = doc.get("ny_plugins") {
                        let mut list: Vec<String> = Vec::new();
                        if let Some(arr) = np.as_array() {
                            for v in arr { if let Some(s) = v.as_str() { list.push(s.to_string()); } }
                        } else if let Some(tbl) = np.as_table() {
                            for (_k, v) in tbl { if let Some(s) = v.as_str() { list.push(s.to_string()); }
                                else if let Some(arr) = v.as_array() { for e in arr { if let Some(s) = e.as_str() { list.push(s.to_string()); } } }
                            }
                        }
                        if !list.is_empty() {
                            let list_only = std::env::var("NYASH_NY_PLUGINS_LIST_ONLY").ok().as_deref() == Some("1");
                            println!("🧩 Ny script plugins ({}):", list.len());
                            for p in list {
                                if list_only {
                                    println!("  • {}", p);
                                    continue;
                                }
                                // Execute each script best-effort via interpreter
                                match std::fs::read_to_string(&p) {
                                    Ok(code) => {
                                        match nyash_rust::parser::NyashParser::parse_from_string(&code) {
                                            Ok(ast) => {
                                                let mut interpreter = nyash_rust::interpreter::NyashInterpreter::new();
                                                match interpreter.execute(ast) {
                                                    Ok(_) => println!("[ny_plugins] {}: OK", p),
                                                    Err(e) => {
                                                        println!("[ny_plugins] {}: FAIL ({})", p, e);
                                                        // continue to next
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("[ny_plugins] {}: FAIL (parse: {})", p, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("[ny_plugins] {}: FAIL (read: {})", p, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Optional: enable VM stats via CLI flags
        if self.config.vm_stats {
            std::env::set_var("NYASH_VM_STATS", "1");
        }
        if self.config.vm_stats_json {
            // Prefer explicit JSON flag over any default
            std::env::set_var("NYASH_VM_STATS_JSON", "1");
        }
        // Optional: JIT controls via CLI flags (centralized)
        {
            // CLI opt-in for JSONL events
            if self.config.jit_events { std::env::set_var("NYASH_JIT_EVENTS", "1"); }
            if self.config.jit_events_compile { std::env::set_var("NYASH_JIT_EVENTS_COMPILE", "1"); }
            if self.config.jit_events_runtime { std::env::set_var("NYASH_JIT_EVENTS_RUNTIME", "1"); }
            if let Some(ref p) = self.config.jit_events_path { std::env::set_var("NYASH_JIT_EVENTS_PATH", p); }
            let mut jc = nyash_rust::jit::config::JitConfig::from_env();
            jc.exec |= self.config.jit_exec;
            jc.stats |= self.config.jit_stats;
            jc.stats_json |= self.config.jit_stats_json;
            jc.dump |= self.config.jit_dump;
            if self.config.jit_threshold.is_some() { jc.threshold = self.config.jit_threshold; }
            jc.phi_min |= self.config.jit_phi_min;
            jc.hostcall |= self.config.jit_hostcall;
            jc.handle_debug |= self.config.jit_handle_debug;
            jc.native_f64 |= self.config.jit_native_f64;
            jc.native_bool |= self.config.jit_native_bool;
            // If observability is enabled and no threshold is provided, force threshold=1 so lowering runs and emits events
            let events_on = std::env::var("NYASH_JIT_EVENTS").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_EVENTS_COMPILE").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_EVENTS_RUNTIME").ok().as_deref() == Some("1");
            if events_on && jc.threshold.is_none() { jc.threshold = Some(1); }
            if self.config.jit_only { std::env::set_var("NYASH_JIT_ONLY", "1"); }
            // Apply runtime capability probe (e.g., disable b1 ABI if unsupported)
            let caps = nyash_rust::jit::config::probe_capabilities();
            jc = nyash_rust::jit::config::apply_runtime_caps(jc, caps);
            // Optional DOT emit via CLI (ensures dump is on when path specified)
            if let Some(path) = &self.config.emit_cfg {
                std::env::set_var("NYASH_JIT_DOT", path);
                jc.dump = true;
            }
            // Persist to env (CLI parity) and set as current
            jc.apply_env();
            nyash_rust::jit::config::set_current(jc.clone());
        }
        // Architectural pivot: JIT is compiler-only (EXE/AOT). Ensure VM runtime does not dispatch to JIT
        // unless explicitly requested via independent JIT mode, or when emitting AOT objects.
        if !self.config.compile_native && !self.config.jit_direct {
            // When AOT object emission is requested, allow JIT to run for object generation
            let aot_obj = std::env::var("NYASH_AOT_OBJECT_OUT").ok();
            if aot_obj.is_none() || aot_obj.as_deref() == Some("") {
                // Force-disable runtime JIT execution path for VM/Interpreter flows
                std::env::set_var("NYASH_JIT_EXEC", "0");
            }
        }
        // Benchmark mode - can run without a file
        if self.config.benchmark {
            println!("📊 Nyash Performance Benchmark Suite");
            println!("====================================");
            println!("Running {} iterations per test...", self.config.iterations);
            println!();
            
            self.execute_benchmark_mode();
            return;
        }

        if let Some(ref filename) = self.config.file {
            // Independent JIT direct mode (no VM execute path)
            if self.config.jit_direct {
                self.run_file_jit_direct(filename);
                return;
            }
            // Delegate file-mode execution to modes::common dispatcher
            self.run_file(filename);
        } else {
            self.execute_demo_mode();
        }
    }

    // init_bid_plugins moved to runner_plugin_init.rs

    /// Execute file-based mode with backend selection

    /// Execute demo mode with all demonstrations
    fn execute_demo_mode(&self) {
        println!("🦀 Nyash Rust Implementation - Everything is Box! 🦀");
        println!("====================================================");
        demos::demo_basic_boxes();
        demos::demo_box_operations();
        demos::demo_box_collections();
        demos::demo_environment_system();
        demos::demo_tokenizer_system();
        demos::demo_parser_system();
        demos::demo_interpreter_system();
        println!("\n🎉 All Box operations completed successfully!");
        println!("Memory safety guaranteed by Rust's borrow checker! 🛡️");
    }

    /// Execute Nyash file with interpreter (moved to modes/common.rs)
    #[cfg(any())]
    fn execute_nyash_file(&self, filename: &str) {
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        
        println!("📝 File contents:\n{}", code);
        println!("\n🚀 Parsing and executing...\n");
        
        // Test: immediate file creation (use relative path to avoid sandbox issues)
        std::fs::create_dir_all("development/debug_hang_issue").ok();
        std::fs::write("development/debug_hang_issue/test.txt", "START").ok();
        
        // Parse the code with debug fuel limit
        eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", self.config.debug_fuel);
        let ast = match NyashParser::parse_from_string_with_fuel(&code, self.config.debug_fuel) {
            Ok(ast) => {
                eprintln!("🔍 DEBUG: Parse completed, AST created");
                ast
            },
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        
        eprintln!("🔍 DEBUG: About to print parse success message...");
        println!("✅ Parse successful!");
        eprintln!("🔍 DEBUG: Parse success message printed");
        
        // Debug log file write
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("development/debug_hang_issue/debug_trace.log") 
        {
            use std::io::Write;
            let _ = writeln!(file, "=== MAIN: Parse successful ===");
            let _ = file.flush();
        }
        
        eprintln!("🔍 DEBUG: Creating interpreter...");
        
        // Execute the AST
        let mut interpreter = NyashInterpreter::new();
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                println!("✅ Execution completed successfully!");
                println!("Result: {}", result.to_string_box().value);
                // Structured concurrency: best-effort join of spawned tasks at program end
                let join_ms: u64 = std::env::var("NYASH_JOIN_ALL_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(2000);
                nyash_rust::runtime::global_hooks::join_all_registered_futures(join_ms);
            },
            Err(e) => {
                // Use enhanced error reporting with source context
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }

    /// Execute MIR compilation and processing mode (moved to modes/mir.rs)
    #[cfg(any())]
    fn execute_mir_mode(&self, filename: &str) {
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
            let mut printer = if self.config.mir_verbose { MirPrinter::verbose() } else { MirPrinter::new() };
            if self.config.mir_verbose_effects { printer.set_show_effects_inline(true); }
            
            println!("🚀 MIR Output for {}:", filename);
            println!("{}", printer.print_module(&compile_result.module));
        }
    }

    /// Execute VM mode (moved to modes/vm.rs)
    #[cfg(any())]
    fn execute_vm_mode(&self, filename: &str) {
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

        // Prepare runtime and collect Box declarations for VM user-defined types
        let runtime = {
            let rt = NyashRuntimeBuilder::new()
                .with_builtin_groups(BuiltinGroups::native_full())
                .build();
            self.collect_box_declarations(&ast, &rt);
            // Register UserDefinedBoxFactory backed by the same declarations
            let mut shared = SharedState::new();
            shared.box_declarations = rt.box_declarations.clone();
            let udf = Arc::new(UserDefinedBoxFactory::new(shared));
            if let Ok(mut reg) = rt.box_registry.lock() {
                reg.register(udf);
            }
            rt
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

        // Execute with VM using prepared runtime
        let mut vm = VM::with_runtime(runtime);
        match vm.execute_module(&compile_result.module) {
            Ok(result) => {
                println!("✅ VM execution completed successfully!");
                if let Some(func) = compile_result.module.functions.get("main") {
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
            },
            Err(e) => {
                eprintln!("❌ VM execution error: {}", e);
                process::exit(1);
            }
        }
    }

    

    /// Collect Box declarations (moved to modes/vm.rs)
    #[cfg(any())]
    fn collect_box_declarations(&self, ast: &ASTNode, runtime: &NyashRuntime) {
        fn walk(node: &ASTNode, runtime: &NyashRuntime) {
            match node {
                ASTNode::Program { statements, .. } => {
                    for st in statements { walk(st, runtime); }
                }
                ASTNode::FunctionDeclaration { body, .. } => {
                    // Walk into function bodies to find nested box declarations
                    for st in body { walk(st, runtime); }
                }
                ASTNode::BoxDeclaration { name, fields, public_fields, private_fields, methods, constructors, init_fields, weak_fields, is_interface, extends, implements, type_parameters, .. } => {
                    // Walk into methods/constructors to find nested box declarations
                    for (_mname, mnode) in methods {
                        walk(mnode, runtime);
                    }
                    for (_ckey, cnode) in constructors {
                        walk(cnode, runtime);
                    }
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
                    if let Ok(mut map) = runtime.box_declarations.write() {
                        map.insert(name.clone(), decl);
                    }
                }
                _ => {}
            }
        }
        walk(ast, runtime);
    }

    // execute_wasm_mode moved to runner::modes::wasm

    // execute_aot_mode moved to runner::modes::aot

    /// Execute LLVM mode (moved to modes/llvm.rs)
    #[cfg(any())]
    fn execute_llvm_mode(&self, filename: &str) {
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

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ MIR compilation error: {}", e);
                process::exit(1);
            }
        };

        println!("📊 MIR Module compiled successfully!");
        println!("📊 Functions: {}", compile_result.module.functions.len());

        // Execute via LLVM backend (mock implementation)
        #[cfg(feature = "llvm")]
        {
            let temp_path = "nyash_llvm_temp";
            match llvm_compile_and_execute(&compile_result.module, temp_path) {
                Ok(result) => {
                    if let Some(int_result) = result.as_any().downcast_ref::<IntegerBox>() {
                        let exit_code = int_result.value;
                        println!("✅ LLVM execution completed!");
                        println!("📊 Exit code: {}", exit_code);
                        
                        // Exit with the same code for testing
                        process::exit(exit_code as i32);
                    } else {
                        println!("✅ LLVM execution completed (non-integer result)!");
                        println!("📊 Result: {}", result.to_string_box().value);
                    }
                },
                Err(e) => {
                    eprintln!("❌ LLVM execution error: {}", e);
                    process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "llvm"))]
        {
            // Mock implementation for demonstration
            println!("🔧 Mock LLVM Backend Execution:");
            println!("   This demonstrates the LLVM backend integration structure.");
            println!("   For actual LLVM compilation, build with --features llvm");
            println!("   and ensure LLVM 17+ development libraries are installed.");
            
            // Analyze the MIR to provide a meaningful mock result
            if let Some(main_func) = compile_result.module.functions.get("Main.main") {
                for (_block_id, block) in &main_func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            MirInstruction::Return { value: Some(_) } => {
                                println!("   📊 Found return instruction - would generate LLVM return 42");
                                println!("✅ Mock LLVM execution completed!");
                                println!("📊 Mock exit code: 42");
                                process::exit(42);
                            }
                            MirInstruction::Return { value: None } => {
                                println!("   📊 Found void return - would generate LLVM return 0");
                                println!("✅ Mock LLVM execution completed!");
                                println!("📊 Mock exit code: 0");
                                process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            println!("✅ Mock LLVM execution completed!");
            println!("📊 Mock exit code: 0");
            process::exit(0);
        }
    }

    /// Execute benchmark mode (moved to modes/bench.rs)
    #[cfg(any())]
    fn execute_benchmark_mode(&self) {
        println!("🏁 Running benchmark mode with {} iterations", self.config.iterations);
        
        // Simple benchmark test file
        let test_code = r#"
        local x
        x = 42
        local y 
        y = x + 58
        return y
        "#;

        println!("\n🧪 Test code:");
        println!("{}", test_code);
        
        // Benchmark interpreter
        println!("\n⚡ Interpreter Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut interpreter = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
                let _ = interpreter.execute(ast);
            }
        }
        let interpreter_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", 
            self.config.iterations, interpreter_time, 
            self.config.iterations as f64 / interpreter_time.as_secs_f64());

        // Benchmark VM if available
        println!("\n🚀 VM Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut mir_compiler = MirCompiler::new();
                if let Ok(compile_result) = mir_compiler.compile(ast) {
                    let mut vm = VM::new();
                    let _ = vm.execute_module(&compile_result.module);
                }
            }
        }
        let vm_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", 
            self.config.iterations, vm_time, 
            self.config.iterations as f64 / vm_time.as_secs_f64());

        // Performance comparison
        let speedup = interpreter_time.as_secs_f64() / vm_time.as_secs_f64();
        println!("\n📊 Performance Summary:");
        println!("  VM is {:.2}x {} than Interpreter", 
            if speedup > 1.0 { speedup } else { 1.0 / speedup },
            if speedup > 1.0 { "faster" } else { "slower" });
    }

    /// Execute a prepared MIR module via the interpreter (Phase-15 path)
    fn execute_mir_module(&self, module: &crate::mir::MirModule) {
        use crate::backend::MirInterpreter;
        use crate::mir::MirType;
        use crate::box_trait::{NyashBox, IntegerBox, BoolBox, StringBox};
        use crate::boxes::FloatBox;

        let mut interp = MirInterpreter::new();
        match interp.execute_module(module) {
            Ok(result) => {
                println!("✅ MIR interpreter execution completed!");
                if let Some(func) = module.functions.get("main") {
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
                std::process::exit(1);
            }
        }
    }

    /// Minimal AOT build pipeline driven by nyash.toml (MVP, single-platform, best-effort)
    fn run_build_mvp(&self, cfg_path: &str) -> Result<(), String> {
        use std::path::{Path, PathBuf};
        let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let cfg_abspath = if Path::new(cfg_path).is_absolute() { PathBuf::from(cfg_path) } else { cwd.join(cfg_path) };
        // 1) Load nyash.toml
        let text = std::fs::read_to_string(&cfg_abspath).map_err(|e| format!("read {}: {}", cfg_abspath.display(), e))?;
        let doc = toml::from_str::<toml::Value>(&text).map_err(|e| format!("parse {}: {}", cfg_abspath.display(), e))?;
        // 2) Apply [env]
        if let Some(env_tbl) = doc.get("env").and_then(|v| v.as_table()) {
            for (k, v) in env_tbl.iter() { if let Some(s) = v.as_str() { std::env::set_var(k, s); } }
        }
        // Derive options
        let profile = self.config.build_profile.clone().unwrap_or_else(|| "release".into());
        let aot = self.config.build_aot.clone().unwrap_or_else(|| "cranelift".into());
        let out = self.config.build_out.clone();
        let target = self.config.build_target.clone();
        // 3) Build plugins: read [plugins] values as paths and build each
        if let Some(pl_tbl) = doc.get("plugins").and_then(|v| v.as_table()) {
            for (name, v) in pl_tbl.iter() {
                if let Some(path) = v.as_str() {
                    let p = if Path::new(path).is_absolute() { PathBuf::from(path) } else { cwd.join(path) };
                    let mut cmd = std::process::Command::new("cargo");
                    cmd.arg("build");
                    if profile == "release" { cmd.arg("--release"); }
                    if let Some(t) = &target { cmd.args(["--target", t]); }
                    cmd.current_dir(&p);
                    println!("[build] plugin {} at {}", name, p.display());
                    let status = cmd.status().map_err(|e| format!("spawn cargo (plugin {}): {}", name, e))?;
                    if !status.success() {
                        return Err(format!("plugin build failed: {} (dir={})", name, p.display()));
                    }
                }
            }
        }
        // 4) Build nyash core (features)
        {
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            if profile == "release" { cmd.arg("--release"); }
            match aot.as_str() { "llvm" => { cmd.args(["--features","llvm"]); }, _ => { cmd.args(["--features","cranelift-jit"]); } }
            if let Some(t) = &target { cmd.args(["--target", t]); }
            println!("[build] nyash core ({}, features={})", profile, if aot=="llvm" {"llvm"} else {"cranelift-jit"});
            let status = cmd.status().map_err(|e| format!("spawn cargo (core): {}", e))?;
            if !status.success() { return Err("nyash core build failed".into()); }
        }
        // 5) Determine app entry
        let app = if let Some(a) = self.config.build_app.clone() { a } else {
            // try [build].app, else suggest
            if let Some(tbl) = doc.get("build").and_then(|v| v.as_table()) {
                if let Some(s) = tbl.get("app").and_then(|v| v.as_str()) { s.to_string() } else { String::new() }
            } else { String::new() }
        };
        let app = if !app.is_empty() { app } else {
            // collect candidates under apps/**/main.nyash
            let mut cand: Vec<String> = Vec::new();
            fn walk(dir: &Path, acc: &mut Vec<String>) {
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for e in rd.flatten() {
                        let p = e.path();
                        if p.is_dir() { walk(&p, acc); }
                        else if p.file_name().map(|n| n=="main.nyash").unwrap_or(false) {
                            acc.push(p.display().to_string());
                        }
                    }
                }
            }
            walk(&cwd.join("apps"), &mut cand);
            let msg = if cand.is_empty() {
                "no app specified (--app) and no apps/**/main.nyash found".to_string()
            } else {
                format!("no app specified (--app). Candidates:\n  - {}", cand.join("\n  - "))
            };
            return Err(msg);
        };
        // 6) Emit object
        let obj_dir = cwd.join("target").join("aot_objects");
        let _ = std::fs::create_dir_all(&obj_dir);
        let obj_path = obj_dir.join("main.o");
        if aot == "llvm" {
            if std::env::var("LLVM_SYS_180_PREFIX").ok().is_none() && std::env::var("LLVM_SYS_181_PREFIX").ok().is_none() {
                return Err("LLVM 18 not configured. Set LLVM_SYS_180_PREFIX or install LLVM 18 (llvm-config)".into());
            }
            std::env::set_var("NYASH_LLVM_OBJ_OUT", &obj_path);
            println!("[emit] LLVM object → {}", obj_path.display());
            let status = std::process::Command::new(cwd.join("target").join(profile.clone()).join(if cfg!(windows) {"nyash.exe"} else {"nyash"}))
                .args(["--backend","llvm", &app])
                .status().map_err(|e| format!("spawn nyash llvm: {}", e))?;
            if !status.success() { return Err("LLVM emit failed".into()); }
        } else {
            std::env::set_var("NYASH_AOT_OBJECT_OUT", &obj_dir);
            println!("[emit] Cranelift object → {} (directory)", obj_dir.display());
            let status = std::process::Command::new(cwd.join("target").join(profile.clone()).join(if cfg!(windows) {"nyash.exe"} else {"nyash"}))
                .args(["--backend","vm", &app])
                .status().map_err(|e| format!("spawn nyash jit-aot: {}", e))?;
            if !status.success() { return Err("Cranelift emit failed".into()); }
        }
        if !obj_path.exists() {
            // In Cranelift path we produce target/aot_objects/<name>.o; fall back to main.o default
            if !obj_dir.join("main.o").exists() { return Err(format!("object not generated under {}", obj_dir.display())); }
        }
        let out_path = if let Some(o) = out { PathBuf::from(o) } else { if cfg!(windows) { cwd.join("app.exe") } else { cwd.join("app") } };
        // 7) Link
        println!("[link] → {}", out_path.display());
        #[cfg(windows)]
        {
            // Prefer MSVC link.exe, then clang fallback
            if let Ok(link) = which::which("link") {
                let status = std::process::Command::new(&link).args(["/NOLOGO", &format!("/OUT:{}", out_path.display().to_string())])
                    .arg(&obj_path)
                    .arg(cwd.join("target").join("release").join("nyrt.lib"))
                    .status().map_err(|e| format!("spawn link.exe: {}", e))?;
                if status.success() { println!("OK"); return Ok(()); }
            }
            if let Ok(clang) = which::which("clang") {
                let status = std::process::Command::new(&clang)
                    .args(["-o", &out_path.display().to_string(), &obj_path.display().to_string()])
                    .arg(cwd.join("target").join("release").join("nyrt.lib").display().to_string())
                    .arg("-lntdll")
                    .status().map_err(|e| format!("spawn clang: {}", e))?;
                if status.success() { println!("OK"); return Ok(()); }
                return Err("link failed on Windows (tried link.exe and clang)".into());
            }
            return Err("no linker found (need Visual Studio link.exe or LLVM clang)".into());
        }
        #[cfg(not(windows))]
        {
            let status = std::process::Command::new("cc")
                .arg(&obj_path)
                .args(["-L", &cwd.join("target").join("release").display().to_string()])
                .args(["-Wl,--whole-archive", "-lnyrt", "-Wl,--no-whole-archive", "-lpthread", "-ldl", "-lm"])
                .args(["-o", &out_path.display().to_string()])
                .status().map_err(|e| format!("spawn cc: {}", e))?;
            if !status.success() { return Err("link failed (cc)".into()); }
        }
        println!("✅ Success: {}", out_path.display());
        Ok(())
    }
}

impl NyashRunner {
    /// Run a file through independent JIT engine (no VM execute loop)
    fn run_file_jit_direct(&self, filename: &str) {
        use std::fs;
        use nyash_rust::{parser::NyashParser, mir::MirCompiler};
        // Small helper for unified error output (text or JSON)
        let emit_err = |phase: &str, code: &str, msg: &str| {
            if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_ERROR_JSON").ok().as_deref() == Some("1") {
                let payload = serde_json::json!({
                    "kind": "jit_direct_error",
                    "phase": phase,
                    "code": code,
                    "message": msg,
                    "file": filename,
                });
                println!("{}", payload.to_string());
            } else {
                eprintln!("[JIT-direct][{}][{}] {}", phase, code, msg);
            }
        };
        // Require cranelift feature at runtime by attempting compile; if unavailable compile_function returns None
        let code = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(e) => { emit_err("read_file", "IO", &format!("{}", e)); std::process::exit(1); }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(a) => a, Err(e) => { emit_err("parse", "SYNTAX", &format!("{}", e)); std::process::exit(1); }
        };
        let mut mc = MirCompiler::new();
        let cr = match mc.compile(ast) { Ok(m) => m, Err(e) => { emit_err("mir", "MIR_COMPILE", &format!("{}", e)); std::process::exit(1); } };
        let func = match cr.module.functions.get("main") { Some(f) => f, None => { emit_err("mir", "NO_MAIN", "No main function found"); std::process::exit(1); } };

        // Guard: refuse write-effects in jit-direct when policy.read_only
        {
            use nyash_rust::mir::MirInstruction;
            use nyash_rust::mir::effect::Effect;
            let policy = nyash_rust::jit::policy::current();
            let mut writes = 0usize;
            for (_bbid, bb) in func.blocks.iter() {
                for inst in bb.instructions.iter() {
                    let mask = inst.effects();
                    if mask.contains(Effect::WriteHeap) {
                        writes += 1;
                    }
                }
                if let Some(term) = &bb.terminator {
                    if term.effects().contains(Effect::WriteHeap) { writes += 1; }
                }
            }
            if policy.read_only && writes > 0 {
                emit_err("policy", "WRITE_EFFECTS", &format!("write-effects detected ({} ops). jit-direct is read-only at this stage.", writes));
                std::process::exit(1);
            }
        }

        // jit-direct 安定化: 分岐合流(PHI)は明示ブロック引数で配線
        {
            let mut cfg = nyash_rust::jit::config::current();
            cfg.phi_min = true; // enable multi-PHI arg passing/join
            nyash_rust::jit::config::set_current(cfg);
        }
        // Prepare minimal runtime hooks so JIT externs (checkpoint/await) can reach GC/scheduler
        {
            let rt = nyash_rust::runtime::NyashRuntime::new();
            nyash_rust::runtime::global_hooks::set_from_runtime(&rt);
        }
        let mut engine = nyash_rust::jit::engine::JitEngine::new();
        match engine.compile_function("main", func) {
            Some(h) => {
                // Optional event: compile
                nyash_rust::jit::events::emit("compile", &func.signature.name, Some(h), None, serde_json::json!({}));
                // Parse JIT args from env: NYASH_JIT_ARGS (comma-separated), with optional type prefixes
                // Formats per arg: i:123, f:3.14, b:true/false, h:42 (handle), or bare numbers (int), true/false (bool)
                let mut jit_args: Vec<nyash_rust::jit::abi::JitValue> = Vec::new();
                if let Ok(s) = std::env::var("NYASH_JIT_ARGS") {
                    for raw in s.split(',') {
                        let t = raw.trim();
                        if t.is_empty() { continue; }
                        let v = if let Some(rest) = t.strip_prefix("i:") {
                            rest.parse::<i64>().ok().map(nyash_rust::jit::abi::JitValue::I64)
                        } else if let Some(rest) = t.strip_prefix("f:") {
                            rest.parse::<f64>().ok().map(nyash_rust::jit::abi::JitValue::F64)
                        } else if let Some(rest) = t.strip_prefix("b:") {
                            let b = matches!(rest, "1"|"true"|"True"|"TRUE");
                            Some(nyash_rust::jit::abi::JitValue::Bool(b))
                        } else if let Some(rest) = t.strip_prefix("h:") {
                            rest.parse::<u64>().ok().map(nyash_rust::jit::abi::JitValue::Handle)
                        } else if t.eq_ignore_ascii_case("true") || t == "1" { Some(nyash_rust::jit::abi::JitValue::Bool(true)) }
                          else if t.eq_ignore_ascii_case("false") || t == "0" { Some(nyash_rust::jit::abi::JitValue::Bool(false)) }
                          else if let Ok(iv) = t.parse::<i64>() { Some(nyash_rust::jit::abi::JitValue::I64(iv)) }
                          else if let Ok(fv) = t.parse::<f64>() { Some(nyash_rust::jit::abi::JitValue::F64(fv)) }
                          else { None };
                        if let Some(jv) = v { jit_args.push(jv); }
                    }
                }
                // Coerce args to expected MIR types
                use nyash_rust::mir::MirType;
                let expected = &func.signature.params;
                if expected.len() != jit_args.len() {
                    emit_err("args", "COUNT_MISMATCH", &format!("expected={}, passed={}", expected.len(), jit_args.len()));
                    eprintln!("Hint: set NYASH_JIT_ARGS as comma-separated values, e.g., i:42,f:3.14,b:true");
                    std::process::exit(1);
                }
                let mut coerced: Vec<nyash_rust::jit::abi::JitValue> = Vec::with_capacity(jit_args.len());
                for (i, (exp, got)) in expected.iter().zip(jit_args.iter()).enumerate() {
                    let cv = match exp {
                        MirType::Integer => match got {
                            nyash_rust::jit::abi::JitValue::I64(v) => nyash_rust::jit::abi::JitValue::I64(*v),
                            nyash_rust::jit::abi::JitValue::F64(f) => nyash_rust::jit::abi::JitValue::I64(*f as i64),
                            nyash_rust::jit::abi::JitValue::Bool(b) => nyash_rust::jit::abi::JitValue::I64(if *b {1} else {0}),
                            _ => { emit_err("args", "TYPE_MISMATCH", &format!("param#{} expects Integer", i)); std::process::exit(1); }
                        },
                        MirType::Float => match got {
                            nyash_rust::jit::abi::JitValue::F64(f) => nyash_rust::jit::abi::JitValue::F64(*f),
                            nyash_rust::jit::abi::JitValue::I64(v) => nyash_rust::jit::abi::JitValue::F64(*v as f64),
                            nyash_rust::jit::abi::JitValue::Bool(b) => nyash_rust::jit::abi::JitValue::F64(if *b {1.0} else {0.0}),
                            _ => { emit_err("args", "TYPE_MISMATCH", &format!("param#{} expects Float", i)); std::process::exit(1); }
                        },
                        MirType::Bool => match got {
                            nyash_rust::jit::abi::JitValue::Bool(b) => nyash_rust::jit::abi::JitValue::Bool(*b),
                            nyash_rust::jit::abi::JitValue::I64(v) => nyash_rust::jit::abi::JitValue::Bool(*v != 0),
                            nyash_rust::jit::abi::JitValue::F64(f) => nyash_rust::jit::abi::JitValue::Bool(*f != 0.0),
                            _ => { emit_err("args", "TYPE_MISMATCH", &format!("param#{} expects Bool", i)); std::process::exit(1); }
                        },
                        MirType::String | MirType::Box(_) | MirType::Array(_) | MirType::Future(_) => match got {
                            nyash_rust::jit::abi::JitValue::Handle(h) => nyash_rust::jit::abi::JitValue::Handle(*h),
                            _ => { emit_err("args", "TYPE_MISMATCH", &format!("param#{} expects handle (h:<id>)", i)); std::process::exit(1); }
                        },
                        MirType::Void | MirType::Unknown => {
                            // Keep as-is
                            *got
                        }
                    };
                    coerced.push(cv);
                }
                nyash_rust::jit::rt::set_current_jit_args(&coerced);
                let t0 = std::time::Instant::now();
                let out = engine.execute_handle(h, &coerced);
                match out {
                    Some(v) => {
                        let ms = t0.elapsed().as_millis();
                        nyash_rust::jit::events::emit("execute", &func.signature.name, Some(h), Some(ms), serde_json::json!({}));
                        // Normalize result according to MIR return type for friendly output
                        use nyash_rust::mir::MirType;
                        let ret_ty = &func.signature.return_type;
                        let vmv = match (ret_ty, v) {
                            (MirType::Bool, nyash_rust::jit::abi::JitValue::I64(i)) => nyash_rust::backend::vm::VMValue::Bool(i != 0),
                            (MirType::Bool, nyash_rust::jit::abi::JitValue::Bool(b)) => nyash_rust::backend::vm::VMValue::Bool(b),
                            (MirType::Float, nyash_rust::jit::abi::JitValue::F64(f)) => nyash_rust::backend::vm::VMValue::Float(f),
                            (MirType::Float, nyash_rust::jit::abi::JitValue::I64(i)) => nyash_rust::backend::vm::VMValue::Float(i as f64),
                            // Default adapter for other combos
                            _ => nyash_rust::jit::abi::adapter::from_jit_value(v),
                        };
                        println!("✅ JIT-direct execution completed successfully!");
                        // Pretty print with expected type tag
                        let (ety, sval) = match (ret_ty, &vmv) {
                            (MirType::Bool, nyash_rust::backend::vm::VMValue::Bool(b)) => ("Bool", b.to_string()),
                            (MirType::Float, nyash_rust::backend::vm::VMValue::Float(f)) => ("Float", format!("{}", f)),
                            (MirType::Integer, nyash_rust::backend::vm::VMValue::Integer(i)) => ("Integer", i.to_string()),
                            // Fallbacks
                            (_, nyash_rust::backend::vm::VMValue::Integer(i)) => ("Integer", i.to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Float(f)) => ("Float", format!("{}", f)),
                            (_, nyash_rust::backend::vm::VMValue::Bool(b)) => ("Bool", b.to_string()),
                            (_, nyash_rust::backend::vm::VMValue::String(s)) => ("String", s.clone()),
                            (_, nyash_rust::backend::vm::VMValue::BoxRef(arc)) => ("BoxRef", arc.type_name().to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Future(_)) => ("Future", "<future>".to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Void) => ("Void", "void".to_string()),
                        };
                        println!("ResultType(MIR): {}", ety);
                        println!("Result: {}", sval);
                        // Optional JSON stats
                        if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1") {
                            let cfg = nyash_rust::jit::config::current();
                            let caps = nyash_rust::jit::config::probe_capabilities();
                            let (phi_t, phi_b1, ret_b) = engine.last_lower_stats();
                            let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig { "b1_bool" } else { "i64_bool" };
                            let payload = serde_json::json!({
                                "version": 1,
                                "function": func.signature.name,
                                "abi_mode": abi_mode,
                                "abi_b1_enabled": cfg.native_bool_abi,
                                "abi_b1_supported": caps.supports_b1_sig,
                                "b1_norm_count": nyash_rust::jit::rt::b1_norm_get(),
                                "ret_bool_hint_count": nyash_rust::jit::rt::ret_bool_hint_get(),
                                "phi_total_slots": phi_t,
                                "phi_b1_slots": phi_b1,
                                "ret_bool_hint_used": ret_b,
                            });
                            println!("{}", payload.to_string());
                        }
                    }
                    None => {
                        nyash_rust::jit::events::emit("fallback", &func.signature.name, Some(h), None, serde_json::json!({"reason":"trap_or_missing"}));
                        emit_err("execute", "TRAP_OR_MISSING", "execution failed (trap or missing handle)");
                        std::process::exit(1);
                    }
                }
            }
            None => {
                emit_err("compile", "UNAVAILABLE", "Build with --features cranelift-jit");
                std::process::exit(1);
            }
        }
    }
}

// Demo functions (moved from main.rs)
// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs
