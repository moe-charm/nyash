#[cfg(feature = "jit-direct-only")]
pub struct JitManager;

#[cfg(feature = "jit-direct-only")]
impl JitManager {
    pub fn new(_threshold: u32) -> Self {
        Self
    }
    pub fn set_threshold(&mut self, _t: u32) {}
    pub fn record_entry(&mut self, _func: &str) {}
    pub fn should_jit(&self, _func: &str) -> bool {
        false
    }
    pub fn mark_compiled(&mut self, _func: &str, _handle: u64) {}
    pub fn maybe_compile(&mut self, _func: &str, _mir: &crate::mir::MirFunction) -> bool {
        false
    }
    pub fn is_compiled(&self, _func: &str) -> bool {
        false
    }
    pub fn handle_of(&self, _func: &str) -> Option<u64> {
        None
    }
    pub fn sites(&self) -> usize {
        0
    }
    pub fn compiled_count(&self) -> usize {
        0
    }
    pub fn total_hits(&self) -> u64 {
        0
    }
    pub fn exec_ok_count(&self) -> u64 {
        0
    }
    pub fn exec_trap_count(&self) -> u64 {
        0
    }
    pub fn record_lower_stats(
        &mut self,
        _func: &str,
        _phi_total: u64,
        _phi_b1: u64,
        _ret_bool_hint: bool,
    ) {
    }
    pub fn per_function_stats(&self) -> Vec<(String, u64, u64, u64, u32, bool, u64)> {
        Vec::new()
    }
    pub fn top_hits(&self, _n: usize) -> Vec<(String, u32, bool, u64)> {
        Vec::new()
    }
    pub fn print_summary(&self) {}
    pub fn maybe_dispatch(&mut self, _func: &str, _argc: usize) -> bool {
        false
    }
    pub fn execute_compiled(
        &mut self,
        _func: &str,
        _ret_ty: &crate::mir::MirType,
        _args: &[crate::backend::vm::VMValue],
    ) -> Option<crate::backend::vm::VMValue> {
        None
    }
}

#[cfg(not(feature = "jit-direct-only"))]
use std::collections::HashMap;

/// Minimal JIT manager skeleton for Phase 10_a
/// - Tracks per-function entry counts
/// - Decides when a function should be JIT-compiled (threshold)
/// - Records compiled functions for stats
#[cfg(not(feature = "jit-direct-only"))]
pub struct JitManager {
    threshold: u32,
    hits: HashMap<String, u32>,
    compiled: HashMap<String, u64>,
    engine: crate::jit::engine::JitEngine,
    exec_ok: u64,
    exec_trap: u64,
    // Per-function lowering stats (accumulated)
    func_phi_total: HashMap<String, u64>,
    func_phi_b1: HashMap<String, u64>,
    func_ret_bool_hint: HashMap<String, u64>,
}

#[cfg(not(feature = "jit-direct-only"))]
impl JitManager {
    pub fn new(threshold: u32) -> Self {
        Self {
            threshold,
            hits: HashMap::new(),
            compiled: HashMap::new(),
            engine: crate::jit::engine::JitEngine::new(),
            exec_ok: 0,
            exec_trap: 0,
            func_phi_total: HashMap::new(),
            func_phi_b1: HashMap::new(),
            func_ret_bool_hint: HashMap::new(),
        }
    }

    pub fn set_threshold(&mut self, t: u32) {
        self.threshold = t.max(1);
    }

    pub fn record_entry(&mut self, func: &str) {
        let c = self.hits.entry(func.to_string()).or_insert(0);
        *c = c.saturating_add(1);
    }

    pub fn should_jit(&self, func: &str) -> bool {
        let hot = self.hits.get(func).copied().unwrap_or(0) >= self.threshold;
        hot && !self.compiled.contains_key(func)
    }

    pub fn mark_compiled(&mut self, func: &str, handle: u64) {
        self.compiled.insert(func.to_string(), handle);
    }

    /// Ensure the function is compiled when hot; returns true if compiled now or already compiled
    pub fn maybe_compile(&mut self, func: &str, mir: &crate::mir::MirFunction) -> bool {
        if self.should_jit(func) {
            if let Some(handle) = self.engine.compile_function(func, mir) {
                self.mark_compiled(func, handle);
                // Record per-function lower stats captured by engine
                let (phi_t, phi_b1, ret_b) = self.engine.last_lower_stats();
                self.record_lower_stats(func, phi_t, phi_b1, ret_b);
                if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                    eprintln!("[JIT] compiled {} -> handle={}", func, handle);
                }
                return true;
            }
        }
        self.compiled.contains_key(func)
    }

    pub fn is_compiled(&self, func: &str) -> bool {
        self.compiled.contains_key(func)
    }
    pub fn handle_of(&self, func: &str) -> Option<u64> {
        self.compiled.get(func).copied()
    }

    // --- Stats accessors for unified reporting ---
    pub fn sites(&self) -> usize {
        self.hits.len()
    }
    pub fn compiled_count(&self) -> usize {
        self.compiled.len()
    }
    pub fn total_hits(&self) -> u64 {
        self.hits.values().map(|v| *v as u64).sum()
    }
    pub fn exec_ok_count(&self) -> u64 {
        self.exec_ok
    }
    pub fn exec_trap_count(&self) -> u64 {
        self.exec_trap
    }

    // --- Per-function stats ---
    pub fn record_lower_stats(
        &mut self,
        func: &str,
        phi_total: u64,
        phi_b1: u64,
        ret_bool_hint: bool,
    ) {
        if phi_total > 0 {
            *self.func_phi_total.entry(func.to_string()).or_insert(0) += phi_total;
        }
        if phi_b1 > 0 {
            *self.func_phi_b1.entry(func.to_string()).or_insert(0) += phi_b1;
        }
        if ret_bool_hint {
            *self.func_ret_bool_hint.entry(func.to_string()).or_insert(0) += 1;
        }
    }
    pub fn per_function_stats(&self) -> Vec<(String, u64, u64, u64, u32, bool, u64)> {
        // name, phi_total, phi_b1, ret_bool_hint, hits, compiled, handle
        let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        names.extend(self.hits.keys().cloned());
        names.extend(self.func_phi_total.keys().cloned());
        names.extend(self.func_phi_b1.keys().cloned());
        names.extend(self.func_ret_bool_hint.keys().cloned());
        let mut out = Vec::new();
        for name in names {
            let phi_t = self.func_phi_total.get(&name).copied().unwrap_or(0);
            let phi_b1 = self.func_phi_b1.get(&name).copied().unwrap_or(0);
            let rb = self.func_ret_bool_hint.get(&name).copied().unwrap_or(0);
            let hits = self.hits.get(&name).copied().unwrap_or(0);
            let compiled = self.compiled.contains_key(&name);
            let handle = self.compiled.get(&name).copied().unwrap_or(0);
            out.push((name, phi_t, phi_b1, rb, hits, compiled, handle));
        }
        out
    }

    /// Return top-N hot functions by hits, with compiled flag and handle
    pub fn top_hits(&self, n: usize) -> Vec<(String, u32, bool, u64)> {
        let mut v: Vec<(&String, &u32)> = self.hits.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        v.into_iter()
            .take(n)
            .map(|(k, h)| {
                let compiled = self.compiled.contains_key(k);
                let handle = self.compiled.get(k).copied().unwrap_or(0);
                (k.clone(), *h, compiled, handle)
            })
            .collect()
    }

    pub fn print_summary(&self) {
        if std::env::var("NYASH_JIT_STATS").ok().as_deref() != Some("1") {
            return;
        }
        let sites = self.hits.len();
        let total_hits: u64 = self.hits.values().map(|v| *v as u64).sum();
        let compiled = self.compiled.len();
        eprintln!(
            "[JIT] sites={} compiled={} hits_total={} exec_ok={} exec_trap={}",
            sites, compiled, total_hits, self.exec_ok, self.exec_trap
        );
        // Top 5 hot functions
        let mut v: Vec<(&String, &u32)> = self.hits.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (k, h)) in v.into_iter().take(5).enumerate() {
            let comp = if self.compiled.contains_key(k) {
                "*"
            } else {
                " "
            };
            let hdl = self.compiled.get(k).copied().unwrap_or(0);
            eprintln!("  #{}{} {} hits={} handle={}", i + 1, comp, k, h, hdl);
        }
    }

    /// Phase 10_c stub: attempt to dispatch to JIT if enabled; returns true if it would execute
    pub fn maybe_dispatch(&mut self, func: &str, argc: usize) -> bool {
        if std::env::var("NYASH_JIT_EXEC").ok().as_deref() == Some("1") {
            if let Some(h) = self.handle_of(func) {
                eprintln!(
                    "[JIT] executing handle={} argc={} (stub) for {}",
                    h, argc, func
                );
                // In 10_c proper, invoke engine with prepared args and return actual result
                // For now, execute with empty args to exercise the path, ignore result
                let _ = self.engine.execute_handle(h, &[]);
                return false; // keep VM path active for now
            }
        }
        false
    }

    /// 10_c: execute compiled function if present (stub: empty args). Returns Some(VMValue) if JIT path was taken.
    pub fn execute_compiled(
        &mut self,
        func: &str,
        ret_ty: &crate::mir::MirType,
        args: &[crate::backend::vm::VMValue],
    ) -> Option<crate::backend::vm::VMValue> {
        // Strict/Fail‑FastモードではJITは"コンパイル専用"（実行しない）
        if std::env::var("NYASH_JIT_STRICT").ok().as_deref() == Some("1") {
            // 観測のためイベントだけ出す
            crate::jit::events::emit_runtime(
                serde_json::json!({
                    "id": "jit_skip_execute_strict",
                    "func": func
                }),
                "jit",
                func,
            );
            return None;
        }
        if let Some(h) = self.handle_of(func) {
            // Expose args to both legacy VM hostcalls and new JIT ABI TLS
            crate::jit::rt::set_legacy_vm_args(args);
            let jit_args = crate::jit::abi::adapter::to_jit_values(args);
            crate::jit::rt::set_current_jit_args(&jit_args);
            let t0 = std::time::Instant::now();
            // Begin handle scope so temporary handles are reclaimed after the call
            crate::jit::rt::handles::begin_scope();
            let out = self.engine.execute_handle(h, &jit_args);
            if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                let dt = t0.elapsed();
                eprintln!("[JIT] exec_time_ms={} for {}", dt.as_millis(), func);
            }
            let res = match out {
                Some(v) => {
                    self.exec_ok = self.exec_ok.saturating_add(1);
                    // Use CallBoundaryBox to convert JitValue → VMValue with MIR ret type hint
                    let vmv = crate::jit::boundary::CallBoundaryBox::to_vm(ret_ty, v);
                    Some(vmv)
                }
                None => {
                    self.exec_trap = self.exec_trap.saturating_add(1);
                    // Emit a minimal trap event for observability (runtime only)
                    let dt = t0.elapsed();
                    crate::jit::events::emit_runtime(
                        serde_json::json!({
                            "kind": "trap",  // redundant with wrapper kind but explicit here for clarity
                            "reason": "jit_execute_failed",
                            "ms": dt.as_millis()
                        }),
                        "trap",
                        func,
                    );
                    None
                }
            };
            // Clear handles created during this call
            crate::jit::rt::handles::end_scope_clear();
            return res;
        }
        None
    }
}
