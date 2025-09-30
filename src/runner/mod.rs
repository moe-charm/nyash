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
mod plugins;
pub mod vm_iface;

// v2 plugin system imports
use nyash_rust::runner_plugin_init;
use nyash_rust::runtime;
// use std::path::PathBuf; // not used in current runner

/// Resolve a using target according to priority: modules > relative > using-paths
/// Returns Ok(resolved_path_or_token). On strict mode, ambiguous matches cause error.
// use pipeline::resolve_using_target; // resolved within helpers; avoid unused warning

/// Main execution coordinator
pub struct NyashRunner {
    config: CliConfig,
}

/// Minimal task runner: read nyash.toml [env] and [tasks], run the named task via shell
use tasks::run_named_task;

#[cfg(not(feature = "jit-direct-only"))]
impl NyashRunner {
    /// Create a new runner with the given configuration
    pub fn new(config: CliConfig) -> Self {
        Self { config }
    }

    /// Run Nyash based on the configuration
    pub fn run(&self) {
        // New behavior-preserving delegator
        self.run_refactored();
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

#[cfg(not(feature = "jit-direct-only"))]
impl NyashRunner {
    /// New behavior-preserving refactor of run(): structured into smaller helpers
    fn run_refactored(&self) {
        // Early: macro child
        if let Some(ref macro_file) = self.config.macro_expand_child {
            crate::runner::modes::macro_child::run_macro_child(macro_file);
            return;
        }
        let groups = self.config.as_groups();
        // Early: build
        if let Some(cfg_path) = groups.build.path.clone() {
            if let Err(e) = self.run_build_mvp(&cfg_path) {
                eprintln!("❌ build error: {}", e);
                std::process::exit(1);
            }
            return;
        }
        // Preprocess usings and directives (includes dep-tree log)
        self.preprocess_usings_and_directives(&groups);
        // JSON v0 bridge
        if self.try_run_json_v0_pipe() { return; }
        // Named task
        if let Some(task) = groups.run_task.clone() {
            if let Err(e) = run_named_task(&task) {
                eprintln!("❌ Task error: {}", e);
                process::exit(1);
            }
            return;
        }
        // Common env + runtime/plugins
        self.apply_common_env(&groups);
        self.init_runtime_and_plugins(&groups);
        // Backend config + policy
        self.configure_backend(&groups);
        self.enforce_runtime_jit_policy(&groups);
        // Benchmark
        if self.maybe_run_benchmark(&groups) { return; }
        // Dispatch
        self.dispatch_entry(&groups);
    }

    // ---- Helpers (extracted from original run) ----

    fn preprocess_usings_and_directives(&self, groups: &crate::cli::CliGroups) {
        use pipeline::resolve_using_target;
        // Initialize UsingContext (defaults + nyash.toml + env)
        let mut using_ctx = self.init_using_context();
        // Collect CLI --using SPEC into (target, alias)
        let mut pending_using: Vec<(String, Option<String>)> = Vec::new();
        for spec in &groups.input.cli_usings {
            let s = spec.trim();
            if s.is_empty() { continue; }
            let (target, alias) = if let Some(pos) = s.find(" as ") {
                (s[..pos].trim().to_string(), Some(s[pos + 4..].trim().to_string()))
            } else { (s.to_string(), None) };
    let is_path = target.starts_with('"') || target.starts_with("./") || target.starts_with('/') || target.ends_with(".hako") || target.ends_with(".nyash");
            if is_path {
                let path = target.trim_matches('"').to_string();
                let name = alias.clone().unwrap_or_else(|| {
                    std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string()
                });
                pending_using.push((name, Some(path)));
            } else {
                pending_using.push((target, alias));
            }
        }
        // Apply pending modules (from context) to registry as StringBox
        for (ns, path) in using_ctx.pending_modules.iter() {
            let sb = crate::box_trait::StringBox::new(path.clone());
            crate::runtime::modules_registry::set(ns.clone(), Box::new(sb));
        }
        // Phase: plugins – proactively load dylib packages from nyash.toml [using.*]
        // Behavior: best-effort, quiet on failure; keeps default behavior unchanged.
        // Rationale: v2 plugins profile expects `using <name> { kind="dylib", path=..., bid=... }`
        // to autoload libraries without additional CLI flags.
        if !using_ctx.packages.is_empty() {
            for (pkg_name, pkg) in using_ctx.packages.iter() {
                if matches!(pkg.kind, crate::using::spec::PackageKind::Dylib) {
                    let path = pkg.path.as_str();
                    // Boxes list: prefer declared `bid`, else empty (TypeBox FFI may populate later)
                    let boxes: Vec<String> = pkg
                        .bid
                        .as_ref()
                        .map(|b| vec![b.clone()])
                        .unwrap_or_else(|| Vec::new());
                    // Use package name as library handle (stable within this process)
                    let host = nyash_rust::runtime::get_global_plugin_host();
                    if let Ok(ro) = host.read() {
                        let _ = ro.load_library_direct(pkg_name, path, &boxes);
                    }
                    // Also record marker token so suggesters/diagnostics can see it
                    let sb = nyash_rust::box_trait::StringBox::new(format!("dylib:{}", path));
                    nyash_rust::runtime::modules_registry::set(pkg_name.clone(), Box::new(sb));
                }
            }
        }
        // Optional dependency tree bridge (log-only)
        if let Ok(dep_path) = std::env::var("NYASH_DEPS_JSON") {
            match std::fs::read_to_string(&dep_path) {
                Ok(s) => {
                    let bytes = s.as_bytes().len();
                    let mut root_info = String::new();
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                        if let Some(r) = v.get("root_path").and_then(|x| x.as_str()) {
                            root_info = format!(" root='{}'", r);
                        }
                    }
                    crate::cli_v!("[deps] loaded {} bytes from{} {}", bytes, if root_info.is_empty() { "" } else { ":" }, root_info);
                }
                Err(e) => { crate::cli_v!("[deps] read error: {}", e); }
            }
        }
        // If a file is provided, apply script-level directives and late using/env merges
        if let Some(ref filename) = groups.input.file {
            if let Ok(code) = fs::read_to_string(filename) {
                // Apply directives and lint
                let strict_fields = std::env::var("NYASH_FIELDS_TOP_STRICT").ok().as_deref() == Some("1");
                if let Err(e) = cli_directives::apply_cli_directives_from_source(&code, strict_fields, groups.debug.cli_verbose) {
                    eprintln!("❌ Lint/Directive error: {}", e);
                    std::process::exit(1);
                }
                // Late env overrides (paths/modules)
                if let Ok(paths) = std::env::var("NYASH_USING_PATH") {
                    for p in paths.split(':') { let p = p.trim(); if !p.is_empty() { using_ctx.using_paths.push(p.to_string()); } }
                }
                if let Ok(mods) = std::env::var("NYASH_MODULES") {
                    for ent in mods.split(',') {
                        if let Some((k, v)) = ent.split_once('=') {
                            let k = k.trim(); let v = v.trim();
                            if !k.is_empty() && !v.is_empty() { using_ctx.pending_modules.push((k.to_string(), v.to_string())); }
                        }
                    }
                }
                // Re-apply pending modules in case env added more (idempotent)
                for (ns, path) in using_ctx.pending_modules.iter() {
                    let sb = nyash_rust::box_trait::StringBox::new(path.clone());
                    nyash_rust::runtime::modules_registry::set(ns.clone(), Box::new(sb));
                }
                // Resolve CLI --using entries against context and register values (with aliasing)
                let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
                let verbose = crate::config::env::cli_verbose();
                let ctx = std::path::Path::new(filename).parent();
                for (ns, alias) in pending_using.iter() {
                    let value = match resolve_using_target(ns, false, &using_ctx.pending_modules, &using_ctx.using_paths, &using_ctx.aliases, &using_ctx.packages, ctx, strict, verbose) {
                        Ok(v) => v,
                        Err(e) => { eprintln!("❌ using: {}", e); std::process::exit(1); }
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
    }

    /// Apply early environment toggles that affect CLI behavior and VM stats.
    /// Side effects: sets `NYASH_CLI_VERBOSE`, `NYASH_GC_MODE` when specified by CLI groups.
    fn apply_common_env(&self, groups: &crate::cli::CliGroups) {
        if groups.debug.cli_verbose { std::env::set_var("NYASH_CLI_VERBOSE", "1"); }
        if let Some(ref m) = groups.gc_mode { if !m.trim().is_empty() { std::env::set_var("NYASH_GC_MODE", m); } }
    }

    // init_runtime_and_plugins moved to runner/plugins.rs

    /// Configure backend knobs (VM) from CLI flags and env.
    /// JIT configuration removed with JIT/Cranelift archival.
    fn configure_backend(&self, groups: &crate::cli::CliGroups) {
        if groups.backend.vm_stats { std::env::set_var("NYASH_VM_STATS", "1"); }
        if groups.backend.vm_stats_json { std::env::set_var("NYASH_VM_STATS_JSON", "1"); }
        // JIT configuration removed - archived to archive/jit-cranelift/
    }

    /// JIT runtime policy enforcement removed with JIT/Cranelift archival.
    fn enforce_runtime_jit_policy(&self, _groups: &crate::cli::CliGroups) {
        // JIT policy enforcement removed - archived to archive/jit-cranelift/
    }

    /// Optionally run the benchmark suite and exit, depending on CLI flags.
    /// Returns true when a benchmark run occurred.
    fn maybe_run_benchmark(&self, groups: &crate::cli::CliGroups) -> bool {
        if groups.benchmark {
            println!("📊 Nyash Performance Benchmark Suite");
            println!("====================================");
            println!("Running {} iterations per test...", groups.iterations);
            println!();
            // VM-legacy removed - benchmark mode no longer available
            eprintln!("❌ Benchmark mode removed with vm-legacy. Use regular execution modes instead.");
            std::process::exit(1);
        }
        false
    }

    /// Final dispatch to selected execution mode (file/JIT-direct or demos).
    fn dispatch_entry(&self, groups: &crate::cli::CliGroups) {
        if let Some(ref filename) = groups.input.file {
            if groups.backend.jit.direct { self.run_file_jit_direct(filename); return; }
            self.run_file(filename);
        } else { demos::run_all_demos(); }
    }
}

impl NyashRunner {
    /// Run a file through independent JIT engine (no VM execute loop)
    /// ARCHIVED: JIT/Cranelift functionality disabled for Phase 15
    fn run_file_jit_direct(&self, filename: &str) {
        eprintln!("❌ JIT-direct mode is archived for Phase 15. JIT/Cranelift moved to archive/jit-cranelift/");
        eprintln!("   Use VM backend instead: nyash {}", filename);
        eprintln!("   Or use LLVM backend: nyash --backend llvm {}", filename);
        std::process::exit(1);
    }
}
// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs

// moved to demos.rs
