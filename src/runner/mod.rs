/*!
 * Execution Runner Module - Nyash File and Mode Execution Coordinator
 *
 * This module handles all execution logic, backend selection, and mode coordination,
 * separated from CLI parsing and the main entry point.
 */

use nyash_rust::cli::CliConfig;
// prune heavy unused imports here; modules import what they need locally
// pruned unused runtime imports in this module

#[cfg(feature = "wasm-backend")]
use nyash_rust::backend::{aot::AotBackend, wasm::WasmBackend};

#[cfg(feature = "llvm-inkwell-legacy")]
use nyash_rust::backend::llvm_compile_and_execute;
use std::{fs, process};
mod box_index;
mod build;
mod cli_directives;
mod demos;
mod dispatch;
mod json_v0_bridge;
mod mir_json_emit;
pub mod modes;
mod pipe_io;
mod pipeline;
mod jit_direct;
mod selfhost;
mod tasks;
mod trace;

// v2 plugin system imports
use nyash_rust::runner_plugin_init;
use nyash_rust::runtime;
// use std::path::PathBuf; // not used in current runner

/// Resolve a using target according to priority: modules > relative > using-paths
/// Returns Ok(resolved_path_or_token). On strict mode, ambiguous matches cause error.
use pipeline::resolve_using_target;

/// Main execution coordinator
pub struct NyashRunner {
    config: CliConfig,
}

/// Minimal task runner: read nyash.toml [env] and [tasks], run the named task via shell
use tasks::run_named_task;

impl NyashRunner {
    /// Create a new runner with the given configuration
    pub fn new(config: CliConfig) -> Self {
        Self { config }
    }

    /// Run Nyash based on the configuration
    pub fn run(&self) {
        // Macro sandbox child mode: --macro-expand-child <file>
        if let Some(ref macro_file) = self.config.macro_expand_child {
            crate::runner::modes::macro_child::run_macro_child(macro_file);
            return;
        }
        // Build system (MVP): nyash --build <nyash.toml>
        let groups = self.config.as_groups();
        if let Some(cfg_path) = groups.build.path.clone() {
            if let Err(e) = self.run_build_mvp(&cfg_path) {
                eprintln!("❌ build error: {}", e);
                std::process::exit(1);
            }
            return;
        }
        // Using/module overrides pre-processing
        let mut using_ctx = self.init_using_context();
        let mut pending_using: Vec<(String, Option<String>)> = Vec::new();
        // CLI --using SPEC entries (SPEC: 'ns', 'ns as Alias', '"path" as Alias')
        for spec in &groups.input.cli_usings {
            let s = spec.trim();
            if s.is_empty() {
                continue;
            }
            let (target, alias) = if let Some(pos) = s.find(" as ") {
                (
                    s[..pos].trim().to_string(),
                    Some(s[pos + 4..].trim().to_string()),
                )
            } else {
                (s.to_string(), None)
            };
            // Normalize quotes for path
            let is_path = target.starts_with('"')
                || target.starts_with("./")
                || target.starts_with('/')
                || target.ends_with(".nyash");
            if is_path {
                let path = target.trim_matches('"').to_string();
                let name = alias.clone().unwrap_or_else(|| {
                    std::path::Path::new(&path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("module")
                        .to_string()
                });
                pending_using.push((name, Some(path)));
            } else {
                pending_using.push((target, alias));
            }
        }
        for (ns, path) in using_ctx.pending_modules.iter() {
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
                        if let Some(r) = v.get("root_path").and_then(|x| x.as_str()) {
                            root_info = format!(" root='{}'", r);
                        }
                    }
                    crate::cli_v!(
                        "[deps] loaded {} bytes from{} {}",
                        bytes,
                        if root_info.is_empty() { "" } else { ":" },
                        root_info
                    );
                }
                Err(e) => {
                    crate::cli_v!("[deps] read error: {}", e);
                }
            }
        }

        // Phase-15: JSON IR v0 bridge (stdin/file)
        if self.try_run_json_v0_pipe() {
            return;
        }
        // Run named task from nyash.toml (MVP)
        if let Some(task) = groups.run_task.clone() {
            if let Err(e) = run_named_task(&task) {
                eprintln!("❌ Task error: {}", e);
                process::exit(1);
            }
            return;
        }
        // Verbose CLI flag maps to env for downstream helpers/scripts
        if groups.debug.cli_verbose {
            std::env::set_var("NYASH_CLI_VERBOSE", "1");
        }
        // GC mode forwarding: map CLI --gc to NYASH_GC_MODE for downstream runtimes
        if let Some(ref m) = groups.gc_mode {
            if !m.trim().is_empty() {
                std::env::set_var("NYASH_GC_MODE", m);
            }
        }
        // Script-level env directives (special comments) — parse early
        // Supported:
        //   // @env KEY=VALUE
        //   // @jit-debug           (preset: exec, threshold=1, events+trace)
        //   // @plugin-builtins     (NYASH_USE_PLUGIN_BUILTINS=1)
        if let Some(ref filename) = groups.input.file {
            if let Ok(code) = fs::read_to_string(filename) {
                // Apply script-level directives and lint
                let strict_fields =
                    std::env::var("NYASH_FIELDS_TOP_STRICT").ok().as_deref() == Some("1");
                if let Err(e) = cli_directives::apply_cli_directives_from_source(
                    &code,
                    strict_fields,
                groups.debug.cli_verbose,
                ) {
                    eprintln!("❌ Lint/Directive error: {}", e);
                    std::process::exit(1);
                }

                // Env overrides for using rules
                // Merge late env overrides (if any)
                if let Ok(paths) = std::env::var("NYASH_USING_PATH") {
                    for p in paths.split(':') {
                        let p = p.trim();
                        if !p.is_empty() {
                            using_ctx.using_paths.push(p.to_string());
                        }
                    }
                }
                if let Ok(mods) = std::env::var("NYASH_MODULES") {
                    for ent in mods.split(',') {
                        if let Some((k, v)) = ent.split_once('=') {
                            let k = k.trim();
                            let v = v.trim();
                            if !k.is_empty() && !v.is_empty() {
                                using_ctx
                                    .pending_modules
                                    .push((k.to_string(), v.to_string()));
                            }
                        }
                    }
                }

                // Apply pending modules to registry as StringBox (path or ns token)
                for (ns, path) in using_ctx.pending_modules.iter() {
                    let sb = nyash_rust::box_trait::StringBox::new(path.clone());
                    nyash_rust::runtime::modules_registry::set(ns.clone(), Box::new(sb));
                }
                // Resolve pending using with clear precedence and ambiguity handling
                let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
                let verbose = crate::config::env::cli_verbose();
                let ctx = std::path::Path::new(filename).parent();
                for (ns, alias) in pending_using.iter() {
                    let value = match resolve_using_target(
                        ns,
                        false,
                        &using_ctx.pending_modules,
                        &using_ctx.using_paths,
                        &using_ctx.aliases,
                        ctx,
                        strict,
                        verbose,
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("❌ using: {}", e);
                            std::process::exit(1);
                        }
                    };
                    let sb = nyash_rust::box_trait::StringBox::new(value.clone());
                    nyash_rust::runtime::modules_registry::set(ns.clone(), Box::new(sb));
                    if let Some(a) = alias {
                        let sb2 = nyash_rust::box_trait::StringBox::new(value);
                        nyash_rust::runtime::modules_registry::set(a.clone(), Box::new(sb2));
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
            // Build BoxIndex after plugin host is initialized
            crate::runner::box_index::refresh_box_index();
        }
        // Allow interpreter to create plugin-backed boxes via unified registry
        // Opt-in by default for FileBox/TOMLBox which are required by ny-config and similar tools.
        if std::env::var("NYASH_USE_PLUGIN_BUILTINS").ok().is_none() {
            std::env::set_var("NYASH_USE_PLUGIN_BUILTINS", "1");
        }
        // Merge FileBox,TOMLBox with defaults if present
        let mut override_types: Vec<String> =
            if let Ok(list) = std::env::var("NYASH_PLUGIN_OVERRIDE_TYPES") {
                list.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                vec!["ArrayBox".into(), "MapBox".into()]
            };
        for t in ["FileBox", "TOMLBox"] {
            if !override_types.iter().any(|x| x == t) {
                override_types.push(t.into());
            }
        }
        std::env::set_var("NYASH_PLUGIN_OVERRIDE_TYPES", override_types.join(","));

        // Opt-in: load Ny script plugins listed in nyash.toml [ny_plugins]
        if groups.load_ny_plugins
            || std::env::var("NYASH_LOAD_NY_PLUGINS").ok().as_deref() == Some("1")
        {
            if let Ok(text) = std::fs::read_to_string("nyash.toml") {
                if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                    if let Some(np) = doc.get("ny_plugins") {
                        let mut list: Vec<String> = Vec::new();
                        if let Some(arr) = np.as_array() {
                            for v in arr {
                                if let Some(s) = v.as_str() {
                                    list.push(s.to_string());
                                }
                            }
                        } else if let Some(tbl) = np.as_table() {
                            for (_k, v) in tbl {
                                if let Some(s) = v.as_str() {
                                    list.push(s.to_string());
                                } else if let Some(arr) = v.as_array() {
                                    for e in arr {
                                        if let Some(s) = e.as_str() {
                                            list.push(s.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        if !list.is_empty() {
                            let list_only =
                                std::env::var("NYASH_NY_PLUGINS_LIST_ONLY").ok().as_deref()
                                    == Some("1");
                            println!("🧩 Ny script plugins ({}):", list.len());
                            for p in list {
                                if list_only {
                                    println!("  • {}", p);
                                    continue;
                                }
                                // Execute each script best-effort via interpreter
                                match std::fs::read_to_string(&p) {
                                    Ok(code) => {
                                        match nyash_rust::parser::NyashParser::parse_from_string(
                                            &code,
                                        ) {
                                            Ok(ast) => {
                                                let mut interpreter =
                                                    nyash_rust::interpreter::NyashInterpreter::new(
                                                    );
                                                match interpreter.execute(ast) {
                                                    Ok(_) => println!("[ny_plugins] {}: OK", p),
                                                    Err(e) => {
                                                        println!(
                                                            "[ny_plugins] {}: FAIL ({})",
                                                            p, e
                                                        );
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
        if groups.backend.vm_stats {
            std::env::set_var("NYASH_VM_STATS", "1");
        }
        if groups.backend.vm_stats_json {
            // Prefer explicit JSON flag over any default
            std::env::set_var("NYASH_VM_STATS_JSON", "1");
        }
        // Optional: JIT controls via CLI flags (centralized)
        {
            // CLI opt-in for JSONL events
            if groups.backend.jit.events {
                std::env::set_var("NYASH_JIT_EVENTS", "1");
            }
            if groups.backend.jit.events_compile {
                std::env::set_var("NYASH_JIT_EVENTS_COMPILE", "1");
            }
            if groups.backend.jit.events_runtime {
                std::env::set_var("NYASH_JIT_EVENTS_RUNTIME", "1");
            }
            if let Some(ref p) = groups.backend.jit.events_path {
                std::env::set_var("NYASH_JIT_EVENTS_PATH", p);
            }
            let mut jc = nyash_rust::jit::config::JitConfig::from_env();
            jc.exec |= groups.backend.jit.exec;
            jc.stats |= groups.backend.jit.stats;
            jc.stats_json |= groups.backend.jit.stats_json;
            jc.dump |= groups.backend.jit.dump;
            if groups.backend.jit.threshold.is_some() {
                jc.threshold = groups.backend.jit.threshold;
            }
            jc.phi_min |= groups.backend.jit.phi_min;
            jc.hostcall |= groups.backend.jit.hostcall;
            jc.handle_debug |= groups.backend.jit.handle_debug;
            jc.native_f64 |= groups.backend.jit.native_f64;
            jc.native_bool |= groups.backend.jit.native_bool;
            // If observability is enabled and no threshold is provided, force threshold=1 so lowering runs and emits events
            let events_on = std::env::var("NYASH_JIT_EVENTS").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_EVENTS_COMPILE").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_EVENTS_RUNTIME").ok().as_deref() == Some("1");
            if events_on && jc.threshold.is_none() {
                jc.threshold = Some(1);
            }
            if groups.backend.jit.only {
                std::env::set_var("NYASH_JIT_ONLY", "1");
            }
            // Apply runtime capability probe (e.g., disable b1 ABI if unsupported)
            let caps = nyash_rust::jit::config::probe_capabilities();
            jc = nyash_rust::jit::config::apply_runtime_caps(jc, caps);
            // Optional DOT emit via CLI (ensures dump is on when path specified)
            if let Some(path) = &groups.emit.emit_cfg {
                std::env::set_var("NYASH_JIT_DOT", path);
                jc.dump = true;
            }
            // Persist to env (CLI parity) and set as current
            jc.apply_env();
            nyash_rust::jit::config::set_current(jc.clone());
        }
        // Architectural pivot: JIT is compiler-only (EXE/AOT). Ensure VM runtime does not dispatch to JIT
        // unless explicitly requested via independent JIT mode, or when emitting AOT objects.
        if !groups.compile_native && !groups.backend.jit.direct {
            // When AOT object emission is requested, allow JIT to run for object generation
            let aot_obj = std::env::var("NYASH_AOT_OBJECT_OUT").ok();
            if aot_obj.is_none() || aot_obj.as_deref() == Some("") {
                // Force-disable runtime JIT execution path for VM/Interpreter flows
                std::env::set_var("NYASH_JIT_EXEC", "0");
            }
        }
        // Benchmark mode - can run without a file
        if groups.benchmark {
            println!("📊 Nyash Performance Benchmark Suite");
            println!("====================================");
            println!("Running {} iterations per test...", groups.iterations);
            println!();
            #[cfg(feature = "vm-legacy")]
            {
                self.execute_benchmark_mode();
                return;
            }
            #[cfg(not(feature = "vm-legacy"))]
            {
                eprintln!(
                    "❌ Benchmark mode requires VM backend. Rebuild with --features vm-legacy."
                );
                std::process::exit(1);
            }
        }

        if let Some(ref filename) = groups.input.file {
            // Independent JIT direct mode (no VM execute path)
            if groups.backend.jit.direct {
                self.run_file_jit_direct(filename);
                return;
            }
            // Delegate file-mode execution to modes::common dispatcher
            self.run_file(filename);
        } else {
            demos::run_all_demos();
        }
    }

    // init_bid_plugins moved to runner_plugin_init.rs

    /// Execute file-based mode with backend selection
    pub(crate) fn run_file(&self, filename: &str) {
        dispatch::execute_file_with_backend(self, filename);
    }

    /// Minimal AOT build pipeline driven by nyash.toml (mvp)
    fn run_build_mvp(&self, cfg_path: &str) -> Result<(), String> {
        build::run_build_mvp_impl(self, cfg_path)
    }
}

impl NyashRunner {
    /// Run a file through independent JIT engine (no VM execute loop)
    fn run_file_jit_direct(&self, filename: &str) {
        use nyash_rust::{mir::MirCompiler, parser::NyashParser};
        use std::fs;
        // Small helper for unified error output (text or JSON)
        let emit_err = |phase: &str, code: &str, msg: &str| {
            if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_ERROR_JSON").ok().as_deref() == Some("1")
            {
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
            Err(e) => {
                emit_err("read_file", "IO", &format!("{}", e));
                std::process::exit(1);
            }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(a) => a,
            Err(e) => {
                emit_err("parse", "SYNTAX", &format!("{}", e));
                std::process::exit(1);
            }
        };
        let mut mc = MirCompiler::new();
        let cr = match mc.compile(ast) {
            Ok(m) => m,
            Err(e) => {
                emit_err("mir", "MIR_COMPILE", &format!("{}", e));
                std::process::exit(1);
            }
        };
        let func = match cr.module.functions.get("main") {
            Some(f) => f,
            None => {
                emit_err("mir", "NO_MAIN", "No main function found");
                std::process::exit(1);
            }
        };

        // Guard: refuse write-effects in jit-direct when policy.read_only
        {
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
                    if term.effects().contains(Effect::WriteHeap) {
                        writes += 1;
                    }
                }
            }
            if policy.read_only && writes > 0 {
                emit_err(
                    "policy",
                    "WRITE_EFFECTS",
                    &format!(
                        "write-effects detected ({} ops). jit-direct is read-only at this stage.",
                        writes
                    ),
                );
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
                nyash_rust::jit::events::emit(
                    "compile",
                    &func.signature.name,
                    Some(h),
                    None,
                    serde_json::json!({}),
                );
                // Parse JIT args from env: NYASH_JIT_ARGS (comma-separated), with optional type prefixes
                // Formats per arg: i:123, f:3.14, b:true/false, h:42 (handle), or bare numbers (int), true/false (bool)
                let mut jit_args: Vec<nyash_rust::jit::abi::JitValue> = Vec::new();
                if let Ok(s) = std::env::var("NYASH_JIT_ARGS") {
                    for raw in s.split(',') {
                        let t = raw.trim();
                        if t.is_empty() {
                            continue;
                        }
                        let v = if let Some(rest) = t.strip_prefix("i:") {
                            rest.parse::<i64>()
                                .ok()
                                .map(nyash_rust::jit::abi::JitValue::I64)
                        } else if let Some(rest) = t.strip_prefix("f:") {
                            rest.parse::<f64>()
                                .ok()
                                .map(nyash_rust::jit::abi::JitValue::F64)
                        } else if let Some(rest) = t.strip_prefix("b:") {
                            let b = matches!(rest, "1" | "true" | "True" | "TRUE");
                            Some(nyash_rust::jit::abi::JitValue::Bool(b))
                        } else if let Some(rest) = t.strip_prefix("h:") {
                            rest.parse::<u64>()
                                .ok()
                                .map(nyash_rust::jit::abi::JitValue::Handle)
                        } else if t.eq_ignore_ascii_case("true") || t == "1" {
                            Some(nyash_rust::jit::abi::JitValue::Bool(true))
                        } else if t.eq_ignore_ascii_case("false") || t == "0" {
                            Some(nyash_rust::jit::abi::JitValue::Bool(false))
                        } else if let Ok(iv) = t.parse::<i64>() {
                            Some(nyash_rust::jit::abi::JitValue::I64(iv))
                        } else if let Ok(fv) = t.parse::<f64>() {
                            Some(nyash_rust::jit::abi::JitValue::F64(fv))
                        } else {
                            None
                        };
                        if let Some(jv) = v {
                            jit_args.push(jv);
                        }
                    }
                }
                // Coerce args to expected MIR types
                use nyash_rust::mir::MirType;
                let expected = &func.signature.params;
                if expected.len() != jit_args.len() {
                    emit_err(
                        "args",
                        "COUNT_MISMATCH",
                        &format!("expected={}, passed={}", expected.len(), jit_args.len()),
                    );
                    eprintln!("Hint: set NYASH_JIT_ARGS as comma-separated values, e.g., i:42,f:3.14,b:true");
                    std::process::exit(1);
                }
                let mut coerced: Vec<nyash_rust::jit::abi::JitValue> =
                    Vec::with_capacity(jit_args.len());
                for (i, (exp, got)) in expected.iter().zip(jit_args.iter()).enumerate() {
                    let cv = match exp {
                        MirType::Integer => match got {
                            nyash_rust::jit::abi::JitValue::I64(v) => {
                                nyash_rust::jit::abi::JitValue::I64(*v)
                            }
                            nyash_rust::jit::abi::JitValue::F64(f) => {
                                nyash_rust::jit::abi::JitValue::I64(*f as i64)
                            }
                            nyash_rust::jit::abi::JitValue::Bool(b) => {
                                nyash_rust::jit::abi::JitValue::I64(if *b { 1 } else { 0 })
                            }
                            _ => {
                                emit_err(
                                    "args",
                                    "TYPE_MISMATCH",
                                    &format!("param#{} expects Integer", i),
                                );
                                std::process::exit(1);
                            }
                        },
                        MirType::Float => match got {
                            nyash_rust::jit::abi::JitValue::F64(f) => {
                                nyash_rust::jit::abi::JitValue::F64(*f)
                            }
                            nyash_rust::jit::abi::JitValue::I64(v) => {
                                nyash_rust::jit::abi::JitValue::F64(*v as f64)
                            }
                            nyash_rust::jit::abi::JitValue::Bool(b) => {
                                nyash_rust::jit::abi::JitValue::F64(if *b { 1.0 } else { 0.0 })
                            }
                            _ => {
                                emit_err(
                                    "args",
                                    "TYPE_MISMATCH",
                                    &format!("param#{} expects Float", i),
                                );
                                std::process::exit(1);
                            }
                        },
                        MirType::Bool => match got {
                            nyash_rust::jit::abi::JitValue::Bool(b) => {
                                nyash_rust::jit::abi::JitValue::Bool(*b)
                            }
                            nyash_rust::jit::abi::JitValue::I64(v) => {
                                nyash_rust::jit::abi::JitValue::Bool(*v != 0)
                            }
                            nyash_rust::jit::abi::JitValue::F64(f) => {
                                nyash_rust::jit::abi::JitValue::Bool(*f != 0.0)
                            }
                            _ => {
                                emit_err(
                                    "args",
                                    "TYPE_MISMATCH",
                                    &format!("param#{} expects Bool", i),
                                );
                                std::process::exit(1);
                            }
                        },
                        MirType::String
                        | MirType::Box(_)
                        | MirType::Array(_)
                        | MirType::Future(_) => match got {
                            nyash_rust::jit::abi::JitValue::Handle(h) => {
                                nyash_rust::jit::abi::JitValue::Handle(*h)
                            }
                            _ => {
                                emit_err(
                                    "args",
                                    "TYPE_MISMATCH",
                                    &format!("param#{} expects handle (h:<id>)", i),
                                );
                                std::process::exit(1);
                            }
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
                        nyash_rust::jit::events::emit(
                            "execute",
                            &func.signature.name,
                            Some(h),
                            Some(ms),
                            serde_json::json!({}),
                        );
                        // Normalize result according to MIR return type for friendly output
                        use nyash_rust::mir::MirType;
                        let ret_ty = &func.signature.return_type;
                        let vmv = match (ret_ty, v) {
                            (MirType::Bool, nyash_rust::jit::abi::JitValue::I64(i)) => {
                                nyash_rust::backend::vm::VMValue::Bool(i != 0)
                            }
                            (MirType::Bool, nyash_rust::jit::abi::JitValue::Bool(b)) => {
                                nyash_rust::backend::vm::VMValue::Bool(b)
                            }
                            (MirType::Float, nyash_rust::jit::abi::JitValue::F64(f)) => {
                                nyash_rust::backend::vm::VMValue::Float(f)
                            }
                            (MirType::Float, nyash_rust::jit::abi::JitValue::I64(i)) => {
                                nyash_rust::backend::vm::VMValue::Float(i as f64)
                            }
                            // Default adapter for other combos
                            _ => nyash_rust::jit::abi::adapter::from_jit_value(v),
                        };
                        println!("✅ JIT-direct execution completed successfully!");
                        // Pretty print with expected type tag
                        let (ety, sval) = match (ret_ty, &vmv) {
                            (MirType::Bool, nyash_rust::backend::vm::VMValue::Bool(b)) => {
                                ("Bool", b.to_string())
                            }
                            (MirType::Float, nyash_rust::backend::vm::VMValue::Float(f)) => {
                                ("Float", format!("{}", f))
                            }
                            (MirType::Integer, nyash_rust::backend::vm::VMValue::Integer(i)) => {
                                ("Integer", i.to_string())
                            }
                            // Fallbacks
                            (_, nyash_rust::backend::vm::VMValue::Integer(i)) => {
                                ("Integer", i.to_string())
                            }
                            (_, nyash_rust::backend::vm::VMValue::Float(f)) => {
                                ("Float", format!("{}", f))
                            }
                            (_, nyash_rust::backend::vm::VMValue::Bool(b)) => {
                                ("Bool", b.to_string())
                            }
                            (_, nyash_rust::backend::vm::VMValue::String(s)) => {
                                ("String", s.clone())
                            }
                            (_, nyash_rust::backend::vm::VMValue::BoxRef(arc)) => {
                                ("BoxRef", arc.type_name().to_string())
                            }
                            (_, nyash_rust::backend::vm::VMValue::Future(_)) => {
                                ("Future", "<future>".to_string())
                            }
                            (_, nyash_rust::backend::vm::VMValue::Void) => {
                                ("Void", "void".to_string())
                            }
                        };
                        println!("ResultType(MIR): {}", ety);
                        println!("Result: {}", sval);
                        // Optional JSON stats
                        if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1") {
                            let cfg = nyash_rust::jit::config::current();
                            let caps = nyash_rust::jit::config::probe_capabilities();
                            let (phi_t, phi_b1, ret_b) = engine.last_lower_stats();
                            let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig {
                                "b1_bool"
                            } else {
                                "i64_bool"
                            };
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
                        nyash_rust::jit::events::emit(
                            "fallback",
                            &func.signature.name,
                            Some(h),
                            None,
                            serde_json::json!({"reason":"trap_or_missing"}),
                        );
                        emit_err(
                            "execute",
                            "TRAP_OR_MISSING",
                            "execution failed (trap or missing handle)",
                        );
                        std::process::exit(1);
                    }
                }
            }
            None => {
                emit_err(
                    "compile",
                    "UNAVAILABLE",
                    "Build with --features cranelift-jit",
                );
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
