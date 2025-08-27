use std::collections::HashMap;

/// Minimal JIT manager skeleton for Phase 10_a
/// - Tracks per-function entry counts
/// - Decides when a function should be JIT-compiled (threshold)
/// - Records compiled functions for stats
pub struct JitManager {
    threshold: u32,
    hits: HashMap<String, u32>,
    compiled: HashMap<String, u64>,
    engine: crate::jit::engine::JitEngine,
}

impl JitManager {
    pub fn new(threshold: u32) -> Self {
        Self { threshold, hits: HashMap::new(), compiled: HashMap::new(), engine: crate::jit::engine::JitEngine::new() }
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
                if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                    eprintln!("[JIT] compiled {} -> handle={}", func, handle);
                }
                return true;
            }
        }
        self.compiled.contains_key(func)
    }

    pub fn is_compiled(&self, func: &str) -> bool { self.compiled.contains_key(func) }
    pub fn handle_of(&self, func: &str) -> Option<u64> { self.compiled.get(func).copied() }

    pub fn print_summary(&self) {
        if std::env::var("NYASH_JIT_STATS").ok().as_deref() != Some("1") { return; }
        let sites = self.hits.len();
        let total_hits: u64 = self.hits.values().map(|v| *v as u64).sum();
        let compiled = self.compiled.len();
        eprintln!("[JIT] sites={} compiled={} hits_total={}", sites, compiled, total_hits);
        // Top 5 hot functions
        let mut v: Vec<(&String, &u32)> = self.hits.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (k, h)) in v.into_iter().take(5).enumerate() {
            let comp = if self.compiled.contains_key(k) { "*" } else { " " };
            let hdl = self.compiled.get(k).copied().unwrap_or(0);
            eprintln!("  #{}{} {} hits={} handle={}", i+1, comp, k, h, hdl);
        }
    }

    /// Phase 10_c stub: attempt to dispatch to JIT if enabled; returns true if it would execute
    pub fn maybe_dispatch(&mut self, func: &str, argc: usize) -> bool {
        if std::env::var("NYASH_JIT_EXEC").ok().as_deref() == Some("1") {
            if let Some(h) = self.handle_of(func) {
                eprintln!("[JIT] executing handle={} argc={} (stub) for {}", h, argc, func);
                // In 10_c proper, invoke engine with prepared args and return actual result
                // For now, execute with empty args to exercise the path, ignore result
                let _ = self.engine.execute_handle(h, &[]);
                return false; // keep VM path active for now
            }
        }
        false
    }

    /// 10_c: execute compiled function if present (stub: empty args). Returns Some(VMValue) if JIT path was taken.
    pub fn execute_compiled(&self, func: &str, args: &[crate::backend::vm::VMValue]) -> Option<crate::backend::vm::VMValue> {
        if let Some(h) = self.handle_of(func) {
            // Expose current args to hostcall shims
            crate::jit::rt::set_current_args(args);
            let t0 = std::time::Instant::now();
            let out = self.engine.execute_handle(h, args);
            if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1") {
                let dt = t0.elapsed();
                eprintln!("[JIT] exec_time_ms={} for {}", dt.as_millis(), func);
            }
            return out;
        }
        None
    }
}
