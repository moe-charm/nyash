//! JIT Engine skeleton
//!
//! Phase 10_a: Provide a placeholder engine interface that later hosts
//! Cranelift contexts and compiled function handles.

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct JitEngine {
    // In the future: isa, module, context, fn table, etc.
    #[allow(dead_code)]
    initialized: bool,
    #[allow(dead_code)]
    next_handle: u64,
    /// Stub function table: handle -> callable closure
    fntab: HashMap<
        u64,
        Arc<dyn Fn(&[crate::jit::abi::JitValue]) -> crate::jit::abi::JitValue + Send + Sync>,
    >,
    /// Host externs by symbol name (Phase 10_d)
    externs: HashMap<
        String,
        Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>,
    >,
    #[cfg(feature = "cranelift-jit")]
    isa: Option<cranelift_codegen::isa::OwnedTargetIsa>,
    // Last lower stats (per function)
    last_phi_total: u64,
    last_phi_b1: u64,
    last_ret_bool_hint: bool,
}

impl JitEngine {
    pub fn new() -> Self {
        let mut this = Self {
            initialized: true,
            next_handle: 1,
            fntab: HashMap::new(),
            externs: HashMap::new(),
            #[cfg(feature = "cranelift-jit")]
            isa: None,
            last_phi_total: 0,
            last_phi_b1: 0,
            last_ret_bool_hint: false,
        };
        #[cfg(feature = "cranelift-jit")]
        {
            this.isa = None;
        }
        this.register_default_externs();
        this
    }

    /// Compile a function if supported; returns an opaque handle id
    pub fn compile_function(
        &mut self,
        func_name: &str,
        mir: &crate::mir::MirFunction,
    ) -> Option<u64> {
        let t0 = std::time::Instant::now();
        // Phase 10_b skeleton: walk MIR with LowerCore and report coverage
        // Reset compile-phase counters (e.g., fallback decisions) before lowering this function
        crate::jit::events::lower_counters_reset();
        let mut lower = crate::jit::lower::core::LowerCore::new();
        #[cfg(feature = "cranelift-jit")]
        let mut builder = crate::jit::lower::builder::CraneliftBuilder::new();
        #[cfg(not(feature = "cranelift-jit"))]
        let mut builder = crate::jit::lower::builder::NoopBuilder::new();
        if let Err(e) = lower.lower_function(mir, &mut builder) {
            eprintln!("[JIT] lower failed for {}: {}", func_name, e);
            return None;
        }
        // Strict: fail compile if any fallback decisions were taken during lowering
        let lower_fallbacks = crate::jit::events::lower_fallbacks_get();
        if lower_fallbacks > 0 && std::env::var("NYASH_JIT_STRICT").ok().as_deref() == Some("1") {
            eprintln!(
                "[JIT][strict] lower produced fallback decisions for {}: {} — failing compile",
                func_name, lower_fallbacks
            );
            return None;
        }
        // Capture per-function lower stats for manager to query later
        let (phi_t, phi_b1, ret_b) = lower.last_stats();
        self.last_phi_total = phi_t;
        self.last_phi_b1 = phi_b1;
        self.last_ret_bool_hint = ret_b;
        // Record per-function stats into manager via callback if available (handled by caller)
        let cfg_now = crate::jit::config::current();
        // Strict mode: any unsupported lowering must fail-fast
        if lower.unsupported > 0 && std::env::var("NYASH_JIT_STRICT").ok().as_deref() == Some("1") {
            eprintln!(
                "[JIT][strict] unsupported lowering ops for {}: {} — failing compile",
                func_name, lower.unsupported
            );
            return None;
        }
        if cfg_now.dump {
            let phi_min = cfg_now.phi_min;
            let native_f64 = cfg_now.native_f64;
            let native_bool = cfg_now.native_bool;
            #[cfg(feature = "cranelift-jit")]
            {
                let s = builder.stats;
                eprintln!("[JIT] lower {}: argc={} phi_min={} f64={} bool={} covered={} unsupported={} (consts={}, binops={}, cmps={}, branches={}, rets={})",
                    func_name, mir.params.len(), phi_min, native_f64, native_bool,
                    lower.covered, lower.unsupported,
                    s.0, s.1, s.2, s.3, s.4);
            }
            #[cfg(not(feature = "cranelift-jit"))]
            {
                eprintln!("[JIT] lower {}: argc={} phi_min={} f64={} bool={} covered={} unsupported={} (consts={}, binops={}, cmps={}, branches={}, rets={})",
                    func_name, mir.params.len(), phi_min, native_f64, native_bool,
                    lower.covered, lower.unsupported,
                    builder.consts, builder.binops, builder.cmps, builder.branches, builder.rets);
            }
            // Optional DOT export
            if let Ok(path) = std::env::var("NYASH_JIT_DOT") {
                if !path.is_empty() {
                    if let Err(e) = crate::jit::lower::core::dump_cfg_dot(mir, &path, phi_min) {
                        eprintln!("[JIT] DOT export failed: {}", e);
                    } else {
                        eprintln!("[JIT] DOT written to {}", path);
                    }
                }
            }
        }
        // If lowering left any unsupported instructions, do not register a closure.
        // This preserves VM semantics until coverage is complete for the function.
        if lower.unsupported > 0
            && std::env::var("NYASH_AOT_ALLOW_UNSUPPORTED").ok().as_deref() != Some("1")
        {
            if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") || cfg_now.dump {
                eprintln!(
                    "[JIT] skip compile for {}: unsupported={} (>0)",
                    func_name, lower.unsupported
                );
            }
            return None;
        }
        // Create a handle and register an executable closure if available
        #[cfg(feature = "cranelift-jit")]
        {
            let h = self.next_handle;
            self.next_handle = self.next_handle.saturating_add(1);
            if let Some(closure) = builder.take_compiled_closure() {
                self.fntab.insert(h, closure);
                if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                    let dt = t0.elapsed();
                    eprintln!("[JIT] compile_time_ms={} for {}", dt.as_millis(), func_name);
                }
                // Optional: also emit an object file for AOT if requested via env
                if let Ok(path) = std::env::var("NYASH_AOT_OBJECT_OUT") {
                    if !path.is_empty() {
                        let mut lower2 = crate::jit::lower::core::LowerCore::new();
                        let mut objb = crate::jit::lower::builder::ObjectBuilder::new();
                        if let Err(e) = lower2.lower_function(mir, &mut objb) {
                            eprintln!("[AOT] lower failed for {}: {}", func_name, e);
                        } else if let Some(bytes) = objb.take_object_bytes() {
                            use std::path::Path;
                            let p = Path::new(&path);
                            let out_path = if p.is_dir() || path.ends_with('/') {
                                p.join(format!("{}.o", func_name))
                            } else {
                                p.to_path_buf()
                            };
                            if let Some(parent) = out_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            match std::fs::write(&out_path, bytes) {
                                Ok(_) => {
                                    eprintln!("[AOT] wrote object: {}", out_path.display());
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[AOT] failed to write object {}: {}",
                                        out_path.display(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
                return Some(h);
            }
            // If Cranelift path did not produce a closure, treat as not compiled
            // Even if a closure was not produced, attempt AOT object emission when requested
            if let Ok(path) = std::env::var("NYASH_AOT_OBJECT_OUT") {
                if !path.is_empty() {
                    let mut lower2 = crate::jit::lower::core::LowerCore::new();
                    let mut objb = crate::jit::lower::builder::ObjectBuilder::new();
                    match lower2.lower_function(mir, &mut objb) {
                        Err(e) => eprintln!("[AOT] lower failed for {}: {}", func_name, e),
                        Ok(()) => {
                            if let Some(bytes) = objb.take_object_bytes() {
                                use std::path::Path;
                                let p = Path::new(&path);
                                let out_path = if p.is_dir() || path.ends_with('/') {
                                    p.join(format!("{}.o", func_name))
                                } else {
                                    p.to_path_buf()
                                };
                                if let Some(parent) = out_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }
                                match std::fs::write(&out_path, bytes) {
                                    Ok(_) => {
                                        eprintln!("[AOT] wrote object: {}", out_path.display())
                                    }
                                    Err(e) => eprintln!(
                                        "[AOT] failed to write object {}: {}",
                                        out_path.display(),
                                        e
                                    ),
                                }
                            } else {
                                eprintln!("[AOT] no object bytes available for {}", func_name);
                            }
                        }
                    }
                }
            }
            return None;
        }
        #[cfg(not(feature = "cranelift-jit"))]
        {
            // Without Cranelift, do not register a stub that alters program semantics.
            // Report as not compiled so VM path remains authoritative.
            if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                let dt = t0.elapsed();
                eprintln!(
                    "[JIT] compile skipped (no cranelift) for {} after {}ms",
                    func_name,
                    dt.as_millis()
                );
            }
            return None;
        }
    }

    /// Get statistics from the last lowered function
    pub fn last_lower_stats(&self) -> (u64, u64, bool) {
        (
            self.last_phi_total,
            self.last_phi_b1,
            self.last_ret_bool_hint,
        )
    }

    /// Execute compiled function by handle with trap fallback.
    /// Returns Some(VMValue) if executed successfully; None on missing handle or trap (panic).
    pub fn execute_handle(
        &self,
        handle: u64,
        args: &[crate::jit::abi::JitValue],
    ) -> Option<crate::jit::abi::JitValue> {
        let f = match self.fntab.get(&handle) {
            Some(f) => f,
            None => return None,
        };
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (f)(args)));
        match res {
            Ok(v) => Some(v),
            Err(_) => {
                if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1")
                    || std::env::var("NYASH_JIT_TRAP_LOG").ok().as_deref() == Some("1")
                {
                    eprintln!(
                        "[JIT] trap: panic during handle={} execution — falling back to VM",
                        handle
                    );
                }
                None
            }
        }
    }

    /// Register built-in externs (collections)
    fn register_default_externs(&mut self) {
        use crate::jit::r#extern::collections as c;
        use crate::jit::r#extern::host_bridge as hb;
        self.register_extern(c::SYM_ARRAY_LEN, Arc::new(|args| c::array_len(args)));
        self.register_extern(c::SYM_ARRAY_GET, Arc::new(|args| c::array_get(args)));
        self.register_extern(c::SYM_ARRAY_SET, Arc::new(|args| c::array_set(args)));
        self.register_extern(c::SYM_ARRAY_PUSH, Arc::new(|args| c::array_push(args)));
        self.register_extern(c::SYM_MAP_GET, Arc::new(|args| c::map_get(args)));
        self.register_extern(c::SYM_MAP_SET, Arc::new(|args| c::map_set(args)));
        self.register_extern(c::SYM_MAP_SIZE, Arc::new(|args| c::map_size(args)));
        // Host-bridge variants (by-slot via C symbol). Guarded by env opt-in for now.
        if std::env::var("NYASH_JIT_HOST_BRIDGE").ok().as_deref() == Some("1") {
            self.register_extern(hb::SYM_HOST_ARRAY_LEN, Arc::new(|args| hb::array_len(args)));
            self.register_extern(hb::SYM_HOST_ARRAY_GET, Arc::new(|args| hb::array_get(args)));
            self.register_extern(hb::SYM_HOST_ARRAY_SET, Arc::new(|args| hb::array_set(args)));
            self.register_extern(hb::SYM_HOST_MAP_SIZE, Arc::new(|args| hb::map_size(args)));
            self.register_extern(hb::SYM_HOST_MAP_GET, Arc::new(|args| hb::map_get(args)));
            self.register_extern(hb::SYM_HOST_MAP_SET, Arc::new(|args| hb::map_set(args)));
            self.register_extern(hb::SYM_HOST_MAP_HAS, Arc::new(|args| hb::map_has(args)));
            self.register_extern(
                hb::SYM_HOST_CONSOLE_LOG,
                Arc::new(|args| hb::console_log(args)),
            );
            self.register_extern(
                hb::SYM_HOST_CONSOLE_WARN,
                Arc::new(|args| hb::console_warn(args)),
            );
            self.register_extern(
                hb::SYM_HOST_CONSOLE_ERROR,
                Arc::new(|args| hb::console_error(args)),
            );
            self.register_extern(
                hb::SYM_HOST_INSTANCE_GETFIELD,
                Arc::new(|args| hb::instance_getfield(args)),
            );
            self.register_extern(
                hb::SYM_HOST_INSTANCE_SETFIELD,
                Arc::new(|args| hb::instance_setfield(args)),
            );
            self.register_extern(
                hb::SYM_HOST_STRING_LEN,
                Arc::new(|args| hb::string_len(args)),
            );
        }
    }

    pub fn register_extern(
        &mut self,
        name: &str,
        f: Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>,
    ) {
        self.externs.insert(name.to_string(), f);
    }

    /// Lookup an extern symbol (to be used by the lowering once call emission is added)
    pub fn lookup_extern(
        &self,
        name: &str,
    ) -> Option<
        Arc<dyn Fn(&[crate::backend::vm::VMValue]) -> crate::backend::vm::VMValue + Send + Sync>,
    > {
        self.externs.get(name).cloned()
    }
}
