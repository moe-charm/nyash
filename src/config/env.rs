//! Global environment configuration aggregator (管理棟)
//!
//! Consolidates NYASH_* environment variables across subsystems and
//! optionally applies overrides from `nyash.toml`.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct NyashEnv {
    // ARCHIVED: JIT-related configuration moved to archive/jit-cranelift/ during Phase 15
    // pub jit: crate::jit::config::JitConfig,
    /// Arbitrary key-value overrides loaded from nyash.toml [env]
    pub overrides: BTreeMap<String, String>,
}

impl NyashEnv {
    pub fn from_env() -> Self {
        Self {
            // ARCHIVED: JIT config during Phase 15
            // jit: crate::jit::config::JitConfig::from_env(),
            overrides: BTreeMap::new(),
        }
    }
    /// Apply current struct values into process environment
    pub fn apply_env(&self) {
        // ARCHIVED: JIT config during Phase 15
        // self.jit.apply_env();
        for (k, v) in &self.overrides {
            std::env::set_var(k, v);
        }
    }
}

// Global current env config (thread-safe)
use once_cell::sync::OnceCell;
use std::sync::RwLock;

static GLOBAL_ENV: OnceCell<RwLock<NyashEnv>> = OnceCell::new();
// フェーズM.2: PHI_ON_GATED_WARNED削除（phi-legacy簡略化により不要）

pub fn current() -> NyashEnv {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(cfg) = lock.read() {
            return cfg.clone();
        }
    }
    NyashEnv::from_env()
}

pub fn set_current(cfg: NyashEnv) {
    if let Some(lock) = GLOBAL_ENV.get() {
        if let Ok(mut w) = lock.write() {
            *w = cfg;
            return;
        }
    }
    let _ = GLOBAL_ENV.set(RwLock::new(cfg));
}

/// Load overrides from nyash.toml `[env]` table and apply them to process env.
///
/// Example:
/// [env]
/// NYASH_JIT_THRESHOLD = "1"
/// NYASH_CLI_VERBOSE = "1"
pub fn bootstrap_from_toml_env() {
    // Allow disabling nyash.toml env bootstrapping for isolated smokes/CI
    if std::env::var("NYASH_SKIP_TOML_ENV").ok().as_deref() == Some("1") {
        return;
    }
    // Accept brand alias environment variables before reading files
    alias_prefixes_bootstrap();
    // Resolve hako.toml/nyash.toml/hakorune.toml from CWD, fallback to *_ROOT if necessary.
    #[allow(unused_assignments)]
    let mut content = String::new();
    // Prefer hako.toml first
    let path_hako = std::path::Path::new("hako.toml");
    if path_hako.exists() {
        match std::fs::read_to_string(path_hako) {
            Ok(s) => { content = s; }
            Err(_) => return,
        }
    } else {
        // Try nyash.toml in CWD next
        let path_ny = std::path::Path::new("nyash.toml");
        if path_ny.exists() {
            match std::fs::read_to_string(path_ny) {
                Ok(s) => { content = s; }
                Err(_) => return,
            }
        } else {
            // Try hakorune.toml in CWD next
        let path_alt = std::path::Path::new("hakorune.toml");
        if path_alt.exists() {
            match std::fs::read_to_string(path_alt) {
                Ok(s) => { content = s; }
                Err(_) => return,
            }
        } else if let Some(root) = env_root_any() {
            // Try $ROOT/nyash.toml then $ROOT/hakorune.toml
            // Also prefer hako.toml under root
            let ph = std::path::Path::new(&root).join("hako.toml");
            if let Ok(s) = std::fs::read_to_string(&ph) { content = s; }
            else {
                let p = std::path::Path::new(&root).join("nyash.toml");
                if let Ok(s) = std::fs::read_to_string(&p) { content = s; }
                else {
                    let p2 = std::path::Path::new(&root).join("hakorune.toml");
                    match std::fs::read_to_string(p2) {
                    Ok(s) => { content = s; }
                    Err(_) => return,
                    }
                }
            }
        } else {
            return;
        }
        }
    }
    let Ok(value) = toml::from_str::<toml::Value>(&content) else {
        return;
    };
    let Some(env_tbl) = value.get("env").and_then(|v| v.as_table()) else {
        return;
    };
    let mut overrides: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in env_tbl {
        if let Some(s) = v.as_str() {
            std::env::set_var(k, s);
            overrides.insert(k.clone(), s.to_string());
        } else if let Some(b) = v.as_bool() {
            let sv = if b { "1" } else { "0" };
            std::env::set_var(k, sv);
            overrides.insert(k.clone(), sv.to_string());
        } else if let Some(n) = v.as_integer() {
            let sv = n.to_string();
            std::env::set_var(k, &sv);
            overrides.insert(k.clone(), sv);
        }
    }
    // Merge into global
    let mut cur = current();
    cur.overrides.extend(overrides);
    set_current(cur);
}

/// Copy HAKU_*/HRN_* to NYASH_* if unset (non-destructive), incl. *_ROOT.
pub fn alias_prefixes_bootstrap() {
    // Root alias first
    if std::env::var("NYASH_ROOT").ok().is_none() {
        if let Ok(v) = std::env::var("HAKO_ROOT") { std::env::set_var("NYASH_ROOT", v); }
        else if let Ok(v) = std::env::var("HAKU_ROOT") { std::env::set_var("NYASH_ROOT", v); }
        else if let Ok(v) = std::env::var("HRN_ROOT") { std::env::set_var("NYASH_ROOT", v); }
    }
    // General mapping
    let snapshot: Vec<(String, String)> = std::env::vars().collect();
    for (k, v) in snapshot.iter() {
        if let Some(tail) = k.strip_prefix("HAKO_")
            .or_else(|| k.strip_prefix("HAKU_"))
            .or_else(|| k.strip_prefix("HRN_")) {
            let ny = format!("NYASH_{}", tail);
            if std::env::var(&ny).ok().is_none() {
                std::env::set_var(ny, v);
            }
        }
    }
}

/// Return NYASH_ROOT or its aliases (HAKU_ROOT/HRN_ROOT)
fn env_root_any() -> Option<String> {
    std::env::var("NYASH_ROOT").ok()
        .or_else(|| std::env::var("HAKO_ROOT").ok())
        .or_else(|| std::env::var("HAKU_ROOT").ok())
        .or_else(|| std::env::var("HRN_ROOT").ok())
}

/// Get await maximum milliseconds, centralized here for consistency.
pub fn await_max_ms() -> u64 {
    std::env::var("NYASH_AWAIT_MAX_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000)
}

// ---- MIR PHI / PHI-less (edge-copy) mode ----
/// Enable MIR PHI non-generation for Bridge compatibility mode only.
/// フェーズM.2: MirBuilder/LoopBuilderでPHI統一済み、Bridge層の互換性制御のみ
/// Default: PHI-ON (Phase 15 direction), override with NYASH_MIR_NO_PHI=1
pub fn mir_no_phi() -> bool {
    match std::env::var("NYASH_MIR_NO_PHI").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true, // フェーズM.2: デフォルトPHI-ON統一
    }
}

/// Allow verifier to skip SSA/dominance/merge checks for PHI-less MIR.
pub fn verify_allow_no_phi() -> bool {
    std::env::var("NYASH_VERIFY_ALLOW_NO_PHI").ok().as_deref() == Some("1") || mir_no_phi()
}

/// Enable strict edge-copy policy verification in PHI-off mode.
/// When enabled, merge blocks must receive merged values via predecessor copies only,
/// and the merge block itself must not introduce a self-copy to the merged destination.
pub fn verify_edge_copy_strict() -> bool {
    std::env::var("NYASH_VERIFY_EDGE_COPY_STRICT").ok().as_deref() == Some("1")
}

// ---- LLVM harness toggle (llvmlite) ----
pub fn llvm_use_harness() -> bool {
    // Phase 15: デフォルトON（LLVMバックエンドはPythonハーネス使用）
    // NYASH_LLVM_USE_HARNESS=0 で明示的に無効化可能
    match std::env::var("NYASH_LLVM_USE_HARNESS").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true, // デフォルト: ON（ハーネス使用）
    }
}

// ---- Phase 11.8 MIR cleanup toggles ----
/// Core-13 minimal MIR mode toggle
/// Default: ON (unless explicitly disabled with NYASH_MIR_CORE13=0)
pub fn mir_core13() -> bool {
    match std::env::var("NYASH_MIR_CORE13").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => true,
    }
}
pub fn mir_ref_boxcall() -> bool {
    std::env::var("NYASH_MIR_REF_BOXCALL").ok().as_deref() == Some("1") || mir_core13()
}
pub fn mir_array_boxcall() -> bool {
    std::env::var("NYASH_MIR_ARRAY_BOXCALL").ok().as_deref() == Some("1") || mir_core13()
}
pub fn mir_plugin_invoke() -> bool {
    std::env::var("NYASH_MIR_PLUGIN_INVOKE").ok().as_deref() == Some("1")
}
pub fn plugin_only() -> bool {
    if let Some(pol) = std::env::var("NYASH_PLUGIN_POLICY").ok() {
        if pol.eq_ignore_ascii_case("force") { return true; }
    }
    std::env::var("NYASH_PLUGIN_ONLY").ok().as_deref() == Some("1")
}

// ---- Plugin ABI Final (Phase A minimal) ----
/// Prefer probing Final ABI (NyValue/NyResult) in loader.
/// Default: OFF. Enable with NYASH_PLUGIN_ABI_FINAL=1
pub fn plugin_abi_final() -> bool {
    std::env::var("NYASH_PLUGIN_ABI_FINAL").ok().as_deref() == Some("1")
}
/// Enable optional metadata probing/logging for plugins (quiet when absent).
/// Default: OFF. Enable with NYASH_PLUGIN_META=1
pub fn plugin_meta() -> bool {
    std::env::var("NYASH_PLUGIN_META").ok().as_deref() == Some("1")
}
/// Development-only: trace side effects from plugin calls (no behavior change).
/// Default: OFF. Enable with NYASH_TRACE_EFFECTS=1
pub fn trace_effects() -> bool {
    std::env::var("NYASH_TRACE_EFFECTS").ok().as_deref() == Some("1")
}
/// Development-time contracts checks (unborn guard instrumentation, arity logs).
/// Default: ON. Disable explicitly with NYASH_CHECK_CONTRACTS=0|false|off
pub fn check_contracts() -> bool {
    match std::env::var("NYASH_CHECK_CONTRACTS").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => true,
    }
}
/// Enforce capability declarations for plugin methods (dev/ci only).
/// Default: OFF. Enable with NYASH_PLUGIN_CAPS_ENFORCE=1
pub fn plugin_caps_enforce() -> bool {
    std::env::var("NYASH_PLUGIN_CAPS_ENFORCE").ok().as_deref() == Some("1")
}

/// Core-13 "pure" mode: after normalization, only the 13 canonical ops are allowed.
/// If enabled, the optimizer will try lightweight rewrites for Load/Store/NewBox/Unary,
/// and the final verifier will reject any remaining non-Core-13 ops.
pub fn mir_core13_pure() -> bool {
    // Deprecated: Core-13 pure mode has been removed (no-op).
    // If explicitly set, emit a deprecation note only when verbose, then return false.
    if let Some(v) = std::env::var("NYASH_MIR_CORE13_PURE").ok() {
        let on = matches!(v.as_str(), "1" | "true" | "on" | "TRUE" | "ON");
        if on && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[deprecated] NYASH_MIR_CORE13_PURE is ignored (feature removed)");
        }
    }
    false
}

/// Enable heuristic pre-pin of comparison operands in if/loop headers.
/// Default: OFF (0). Set NYASH_MIR_PREPIN=1 to enable.
pub fn mir_pre_pin_compare_operands() -> bool {
    match std::env::var("NYASH_MIR_PREPIN").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => false,
    }
}

// ---- Optimizer diagnostics ----
pub fn opt_debug() -> bool {
    std::env::var("NYASH_OPT_DEBUG").is_ok()
}
pub fn opt_diag() -> bool {
    std::env::var("NYASH_OPT_DIAG").is_ok()
}
pub fn opt_diag_forbid_legacy() -> bool {
    std::env::var("NYASH_OPT_DIAG_FORBID_LEGACY").is_ok()
}
pub fn opt_diag_fail() -> bool {
    std::env::var("NYASH_OPT_DIAG_FAIL").is_ok()
}

// ---- GC/Runtime tracing (execution-affecting visibility) ----
pub fn gc_trace() -> bool {
    std::env::var("NYASH_GC_TRACE").ok().as_deref() == Some("1")
}
pub fn gc_barrier_trace() -> bool {
    std::env::var("NYASH_GC_BARRIER_TRACE").ok().as_deref() == Some("1")
}
pub fn runtime_checkpoint_trace() -> bool {
    std::env::var("NYASH_RUNTIME_CHECKPOINT_TRACE")
        .ok()
        .as_deref()
        == Some("1")
}
pub fn vm_pic_stats() -> bool {
    std::env::var("NYASH_VM_PIC_STATS").ok().as_deref() == Some("1")
}
pub fn vm_vt_trace() -> bool {
    std::env::var("NYASH_VM_VT_TRACE").ok().as_deref() == Some("1")
}
pub fn vm_pic_trace() -> bool {
    std::env::var("NYASH_VM_PIC_TRACE").ok().as_deref() == Some("1")
}
pub fn gc_barrier_strict() -> bool {
    std::env::var("NYASH_GC_BARRIER_STRICT").ok().as_deref() == Some("1")
}

/// Return 0 (off) to 3 (max) for `NYASH_GC_TRACE`.
pub fn gc_trace_level() -> u8 {
    match std::env::var("NYASH_GC_TRACE").ok().as_deref() {
        Some("1") => 1,
        Some("2") => 2,
        Some("3") => 3,
        Some(_) => 1,
        None => 0,
    }
}

// ---- GC mode and instrumentation ----
/// Return current GC mode string (auto default = "rc+cycle").
/// Allowed: "auto", "rc+cycle", "minorgen", "stw", "rc", "off"
pub fn gc_mode() -> String {
    match std::env::var("NYASH_GC_MODE").ok() {
        Some(m) if !m.trim().is_empty() => m,
        _ => "rc+cycle".to_string(),
    }
}
/// Brief metrics emission (text)
pub fn gc_metrics() -> bool {
    std::env::var("NYASH_GC_METRICS").ok().as_deref() == Some("1")
}
/// JSON metrics emission (single line)
pub fn gc_metrics_json() -> bool {
    std::env::var("NYASH_GC_METRICS_JSON").ok().as_deref() == Some("1")
}
/// Leak diagnostics on exit
pub fn gc_leak_diag() -> bool {
    std::env::var("NYASH_GC_LEAK_DIAG").ok().as_deref() == Some("1")
}
/// Optional allocation threshold; if Some(n) and exceeded, print warning
pub fn gc_alloc_threshold() -> Option<u64> {
    std::env::var("NYASH_GC_ALLOC_THRESHOLD").ok()?.parse().ok()
}

// ---- Cleanup (method-level postfix) policy toggles ----
/// Allow `return` inside a cleanup block. Default: false (0)
pub fn cleanup_allow_return() -> bool {
    match std::env::var("NYASH_CLEANUP_ALLOW_RETURN").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => false,
    }
}

/// Allow `throw` inside a cleanup block. Default: false (0)
pub fn cleanup_allow_throw() -> bool {
    match std::env::var("NYASH_CLEANUP_ALLOW_THROW").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => false,
    }
}

/// Run a collection every N safepoints (if Some)
pub fn gc_collect_sp_interval() -> Option<u64> {
    std::env::var("NYASH_GC_COLLECT_SP").ok()?.parse().ok()
}
/// Run a collection when allocated bytes since last >= N (if Some)
pub fn gc_collect_alloc_bytes() -> Option<u64> {
    std::env::var("NYASH_GC_COLLECT_ALLOC").ok()?.parse().ok()
}

// ---- Rewriter flags (optimizer transforms)
pub fn rewrite_debug() -> bool {
    std::env::var("NYASH_REWRITE_DEBUG").ok().as_deref() == Some("1")
}
pub fn rewrite_safepoint() -> bool {
    std::env::var("NYASH_REWRITE_SAFEPOINT").ok().as_deref() == Some("1")
}
pub fn rewrite_future() -> bool {
    std::env::var("NYASH_REWRITE_FUTURE").ok().as_deref() == Some("1")
}

// ---- Phase 12: Nyash ABI (vtable) toggles ----
pub fn abi_vtable() -> bool {
    std::env::var("NYASH_ABI_VTABLE").ok().as_deref() == Some("1")
}
pub fn abi_strict() -> bool {
    std::env::var("NYASH_ABI_STRICT").ok().as_deref() == Some("1")
}

// ---- ExternCall strict diagnostics ----
pub fn extern_strict() -> bool {
    std::env::var("NYASH_EXTERN_STRICT").ok().as_deref() == Some("1")
}
pub fn extern_trace() -> bool {
    std::env::var("NYASH_EXTERN_TRACE").ok().as_deref() == Some("1")
}

// ---- Operator Boxes adopt defaults ----
/// CompareOperator.apply adopt: default ON (prod/devともに採用)
pub fn operator_box_compare_adopt() -> bool {
    match std::env::var("NYASH_OPERATOR_BOX_COMPARE_ADOPT").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => false, // default OFF (avoid reentry until VM hook parity is ensured)
    }
}
/// AddOperator.apply adopt: default OFF（順次昇格のため）
pub fn operator_box_add_adopt() -> bool {
    match std::env::var("NYASH_OPERATOR_BOX_ADD_ADOPT").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => false, // default OFF
    }
}

// ---- Null/Missing Boxes (dev-only observe → adopt) ----
/// Enable NullBox/MissingBox observation path (no behavior change by default).
/// Default: OFF. Turn ON with `NYASH_NULL_MISSING_BOX=1`. May be auto-enabled in --dev later.
pub fn null_missing_box_enabled() -> bool {
    std::env::var("NYASH_NULL_MISSING_BOX").ok().as_deref() == Some("1")
}
/// Strict null policy for operators (when enabled): null in arithmetic/compare is an error.
/// Default: OFF (null propagates). Effective only when `null_missing_box_enabled()` is true.
pub fn null_strict() -> bool {
    std::env::var("NYASH_NULL_STRICT").ok().as_deref() == Some("1")
}

// ---- Phase 12: thresholds and routing policies ----
/// PIC hotness threshold before promoting to mono cache.
pub fn vm_pic_threshold() -> u32 {
    std::env::var("NYASH_VM_PIC_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
}

/// Route VM ExternCall via name→slot handlers when available
pub fn extern_route_slots() -> bool {
    std::env::var("NYASH_EXTERN_ROUTE_SLOTS").ok().as_deref() == Some("1")
}

// ---- Runner/CLI common toggles (hot-path centralization)
pub fn cli_verbose() -> bool {
    std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1")
}
/// Global quiet mode for CLI/logging. True when any of the following:
/// - NYASH_JSON_ONLY=1 (child pipelines printing JSON to stdout)
/// - NYASH_QUIET=1 (explicit quiet)
/// - NYASH_CLI_VERBOSE=0 (explicitly disable verbose)
pub fn cli_quiet() -> bool {
    std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1")
        || std::env::var("NYASH_QUIET").ok().as_deref() == Some("1")
        || std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("0")
}
/// Resolver textual trace
pub fn resolve_trace() -> bool {
    std::env::var("NYASH_RESOLVE_TRACE").ok().as_deref() == Some("1")
}
/// Resolver JSON trace
pub fn resolve_trace_json() -> bool {
    std::env::var("NYASH_RESOLVE_TRACE_JSON").ok().as_deref() == Some("1")
}
/// Import textual trace
pub fn import_trace() -> bool {
    std::env::var("NYASH_IMPORT_TRACE").ok().as_deref() == Some("1")
}
pub fn enable_using() -> bool {
    // Phase 15: デフォルトON（using systemはメイン機能）
    // 優先順: NYASH_USING → NYASH_ENABLE_USING（後方互換）。0/false/off で明示無効化可能。
    let alias = std::env::var("NYASH_ENABLE_USING").ok();
    if alias.is_some() && crate::config::env::cli_verbose() {
        eprintln!("[env] NYASH_ENABLE_USING is deprecated; use NYASH_USING instead");
    }
    match std::env::var("NYASH_USING").ok().or(alias).as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true, // デフォルト: ON
    }
}

// ---- Using profiles (dev|ci|prod) ----
/// Return using profile string; default is "dev".
pub fn using_profile() -> String {
    std::env::var("NYASH_USING_PROFILE").unwrap_or_else(|_| "dev".to_string())
}
pub fn using_is_prod() -> bool { using_profile().eq_ignore_ascii_case("prod") }
pub fn using_is_ci() -> bool { using_profile().eq_ignore_ascii_case("ci") }
pub fn using_is_dev() -> bool { using_profile().eq_ignore_ascii_case("dev") }
/// Allow `using "path"` statements in source (dev-only by default).
pub fn allow_using_file() -> bool {
    // SSOT 徹底: 全プロファイルで既定禁止（nyash.toml を唯一の真実に）
    // 明示オーバーライドでのみ許可（開発用緊急時）
    match std::env::var("NYASH_ALLOW_USING_FILE").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}
/// Determine whether AST prelude merge for `using` is enabled.
/// Precedence:
/// 1) Explicit env `NYASH_USING_AST` = 1/true/on → enabled, = 0/false/off → disabled
/// 2) Default by profile: dev/ci → ON, prod → OFF
pub fn using_ast_enabled() -> bool {
    // 優先順: NYASH_USING_STRATEGY / NYASH_USING_IMPL → NYASH_USING_AST（後方互換）。
    if let Some(mode) = std::env::var("NYASH_USING_STRATEGY").ok()
        .or_else(|| std::env::var("NYASH_USING_IMPL").ok())
    {
        let m = mode.to_ascii_lowercase();
        return match m.as_str() {
            "prelude" => true,
            "resolver" => false,
            _ => !using_is_prod(),
        };
    }
    match std::env::var("NYASH_USING_AST").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        _ => !using_is_prod(), // dev/ci → true, prod → false
    }
}
/// Policy: allow VM to fallback-dispatch user Instance BoxCall (dev only by default).
/// - prod: default false (disallow)
/// - dev/ci: default true (allow, with WARN)
/// Override with NYASH_VM_USER_INSTANCE_BOXCALL={0|1}
pub fn using_strict() -> bool {
    match std::env::var("NYASH_USING_STRICT").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        _ => true,
    }
}

pub fn vm_allow_user_instance_boxcall() -> bool {
    match std::env::var("NYASH_VM_USER_INSTANCE_BOXCALL").ok().as_deref().map(|v| v.to_ascii_lowercase()) {
        Some(ref s) if s == "0" || s == "false" || s == "off" => false,
        Some(ref s) if s == "1" || s == "true" || s == "on" => true,
        _ => !using_is_prod(),
    }
}
// Legacy resolve_fix_braces() removed (Phase 15 cleanup)
// AST-based integration handles syntax properly without text-level brace fixing
pub fn vm_use_py() -> bool {
    std::env::var("NYASH_VM_USE_PY").ok().as_deref() == Some("1")
}
pub fn pipe_use_pyvm() -> bool {
    std::env::var("NYASH_PIPE_USE_PYVM").ok().as_deref() == Some("1")
}
pub fn vm_use_dispatch() -> bool {
    std::env::var("NYASH_VM_USE_DISPATCH").ok().as_deref() == Some("1")
}

/// Policy: prefer routing BoxCall to PluginInvoke when receiver is a plugin box.
/// Default: OFF. Enable with NYASH_VM_BOXCALL_PLUGIN_FIRST=1 (dev/experiments only).
pub fn vm_boxcall_plugin_first() -> bool {
    match std::env::var("NYASH_VM_BOXCALL_PLUGIN_FIRST").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

/// Phase C scaffold: prefer plugin path for ArrayBox (default OFF)
pub fn vm_plugin_prefer_array() -> bool {
    match std::env::var("NYASH_VM_PLUGIN_PREFER_ARRAY").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}
/// Phase C scaffold: prefer plugin path for StringBox (default OFF)
pub fn vm_plugin_prefer_string() -> bool {
    match std::env::var("NYASH_VM_PLUGIN_PREFER_STRING").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}
/// Phase C scaffold: prefer plugin path for MapBox (default OFF)
pub fn vm_plugin_prefer_map() -> bool {
    match std::env::var("NYASH_VM_PLUGIN_PREFER_MAP").ok().as_deref() {
        Some("0") | Some("false") | Some("off") => false,
        _ => true,
    }
}

// Self-host compiler knobs
pub fn ny_compiler_timeout_ms() -> u64 {
    std::env::var("NYASH_NY_COMPILER_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000)
}
pub fn ny_compiler_emit_only() -> bool {
    std::env::var("NYASH_NY_COMPILER_EMIT_ONLY").unwrap_or_else(|_| "1".to_string()) == "1"
}
pub fn ny_compiler_skip_py() -> bool {
    std::env::var("NYASH_NY_COMPILER_SKIP_PY").ok().as_deref() == Some("1")
}
pub fn use_ny_compiler_exe() -> bool {
    std::env::var("NYASH_USE_NY_COMPILER_EXE").ok().as_deref() == Some("1")
}
pub fn ny_compiler_exe_path() -> Option<String> {
    std::env::var("NYASH_NY_COMPILER_EXE_PATH").ok()
}
pub fn ny_compiler_min_json() -> bool {
    std::env::var("NYASH_NY_COMPILER_MIN_JSON").ok().as_deref() == Some("1")
}
pub fn selfhost_read_tmp() -> bool {
    std::env::var("NYASH_SELFHOST_READ_TMP").ok().as_deref() == Some("1")
}
pub fn ny_compiler_stage3() -> bool {
    std::env::var("NYASH_NY_COMPILER_STAGE3").ok().as_deref() == Some("1")
}
/// Core (Rust) parser Stage-3 gate
/// When enabled, the Rust parser accepts Stage-3 surface (try/catch/finally, throw).
/// Default is OFF to keep Stage-2 stable.
pub fn parser_stage3() -> bool {
    std::env::var("NYASH_PARSER_STAGE3").ok().as_deref() == Some("1")
}

/// Parser gate for Block‑Postfix Catch acceptance
/// Enabled when either NYASH_BLOCK_CATCH=1 or Stage‑3 gate is on.
/// Phase 15.5 allows parsing a standalone `{ ... }` block optionally followed by
/// a single `catch (...) { ... }` and/or `finally { ... }`, which is folded into
/// ASTNode::TryCatch with the preceding block as the try body.
pub fn block_postfix_catch() -> bool {
    std::env::var("NYASH_BLOCK_CATCH").ok().as_deref() == Some("1") || parser_stage3()
}

/// Bridge lowering: use Result-style try/throw lowering instead of MIR Catch/Throw
/// When on, try/catch is lowered using structured blocks and direct jumps,
/// without emitting MIR Throw/Catch. The thrown value is routed to catch via
/// block parameters (PHI-off uses edge-copy).
pub fn try_result_mode() -> bool {
    std::env::var("NYASH_TRY_RESULT_MODE").ok().as_deref() == Some("1")
}

/// Parser gate for method-level postfix catch/finally acceptance on method definitions.
/// Enabled when either NYASH_METHOD_CATCH=1 or Stage‑3 gate is on.
pub fn method_catch() -> bool {
    std::env::var("NYASH_METHOD_CATCH").ok().as_deref() == Some("1") || parser_stage3()
}

/// Entry policy: allow top-level `main` resolution in addition to `Main.main`.
/// Default: true (prefer `Main.main` when both exist; otherwise accept `main`).
pub fn entry_allow_toplevel_main() -> bool {
    match std::env::var("NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN").ok() {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        }
        None => true,
    }
}

/// Entry policy: prefer a unique static `<Box>.main` when `Main.main` is absent.
///
/// Behavior:
/// - If `Main.main` exists, it is always preferred.
/// - Otherwise, when enabled, if exactly one function named like `Box.main` or
///   `Box.main/0` exists, that function is chosen as the entry point.
/// - If multiple candidates are found, this preference is ignored to avoid
///   ambiguity and the resolver falls back to top-level `main` (when allowed).
///
/// Default: true (helps benchmarks and WASM harnesses that use a static box
/// other than `Main` as the entry container).
pub fn entry_prefer_static_main() -> bool {
    match std::env::var("NYASH_ENTRY_PREFER_STATIC_MAIN").ok() {
        Some(v) => {
            let v = v.to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        }
        None => true,
    }
}

/// Parser gate for expression-level postfix catch/cleanup acceptance.
/// Enabled when Stage-3 gate is on (NYASH_PARSER_STAGE3=1). Separate gate can
/// be introduced in future if needed, but we keep minimal toggles now.
pub fn expr_postfix_catch() -> bool {
    parser_stage3()
}
/// Parser gate for `flow` (stateless namespace) acceptance.
/// Default: ON. Disable with NYASH_ENABLE_FLOW=0|false|off
pub fn parser_flow_enabled() -> bool {
    match std::env::var("NYASH_ENABLE_FLOW").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => true,
    }
}
/// Parser gate for Unified Members (stored/computed/once/birth_once).
/// Default: ON during Phase-15 (set NYASH_ENABLE_UNIFIED_MEMBERS=0|false|off to disable).
pub fn unified_members() -> bool {
    match std::env::var("NYASH_ENABLE_UNIFIED_MEMBERS").ok() {
        Some(v) => {
            let lv = v.to_ascii_lowercase();
            !(lv == "0" || lv == "false" || lv == "off")
        }
        None => true,
    }
}
pub fn ny_compiler_child_args() -> Option<String> {
    std::env::var("NYASH_NY_COMPILER_CHILD_ARGS").ok()
}
pub fn ny_compiler_use_tmp_only() -> bool {
    std::env::var("NYASH_NY_COMPILER_USE_TMP_ONLY")
        .ok()
        .as_deref()
        == Some("1")
}



/// Emit scope statistics on scope pop (dev diagnostics).
/// Default: OFF. Enable with NYASH_SCOPE_STATS=1
pub fn scope_stats_log() -> bool {
    std::env::var("NYASH_SCOPE_STATS").ok().as_deref() == Some("1")
}
/// Emit JSON release/finalize logs (dev diagnostics).
/// Default: OFF. Enable with NYASH_RELEASE_TRACE=1
pub fn release_trace() -> bool {
    std::env::var("NYASH_RELEASE_TRACE").ok().as_deref() == Some("1")
}

// ---- Runner/Compiler dev toggles (selfhost/pipeline helpers) ----
/// Trace CallResolver decisions in VM (development helper)
pub fn vm_resolve_trace() -> bool {
    std::env::var("NYASH_VM_RESOLVE_TRACE").ok().as_deref() == Some("1")
}
/// Map NYASH_EMIT_TRACE=1 to compiler --emit-trace (dev aid)
pub fn emit_trace() -> bool {
    std::env::var("NYASH_EMIT_TRACE").ok().as_deref() == Some("1")
}
/// Prefer CFG lowering (v2) in selfhost compiler
pub fn prefer_cfg2() -> bool {
    std::env::var("NYASH_PREFER_CFG2").ok().as_deref() == Some("1")
}
/// Prefer CFG lowering (v1)
pub fn prefer_cfg() -> bool {
    std::env::var("NYASH_PREFER_CFG").ok().as_deref() == Some("1")
}
/// Enable ScopeBox prepass
pub fn scopebox_enable() -> bool {
    std::env::var("NYASH_SCOPEBOX_ENABLE").ok().as_deref() == Some("1")
}
/// Enable LoopForm normalization prepass
pub fn loopform_normalize() -> bool {
    std::env::var("NYASH_LOOPFORM_NORMALIZE").ok().as_deref() == Some("1")
}
/// Optional macro pre‑expand mode for selfhost ("1" or "auto")
pub fn macro_selfhost_pre_expand() -> Option<String> {
    std::env::var("NYASH_MACRO_SELFHOST_PRE_EXPAND").ok()
}
