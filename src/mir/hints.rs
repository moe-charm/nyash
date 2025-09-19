#![allow(dead_code)]
//! MIR Hints — zero-cost structural guidance (scaffold)
//!
//! Hints guide lowering/verification without affecting semantics.
//! They must be stripped before final IR emission.

/// Lightweight set of hint kinds (scaffold).
#[derive(Debug, Clone)]
pub enum HintKind {
    ScopeEnter(u32),
    ScopeLeave(u32),
    Defer(Vec<String>),
    JoinResult(String),
    LoopCarrier(Vec<String>),
    LoopHeader,
    LoopLatch,
    NoEmptyPhi,
}

/// Hint sink (no-op). Backends/resolvers may hook into this later.
#[derive(Default)]
pub struct HintSink {
    enabled: bool,
}

impl HintSink {
    pub fn new() -> Self { Self { enabled: false } }
    pub fn with_enabled(mut self, enabled: bool) -> Self { self.enabled = enabled; self }

    #[inline]
    pub fn record(&mut self, hint: HintKind) {
        // Lightweight trace: print only when explicitly enabled via env
        let enabled = self.enabled
            || std::env::var("NYASH_MIR_TRACE_HINTS").ok().as_deref() == Some("1");
        if !enabled { return; }
        match hint {
            HintKind::ScopeEnter(id) => eprintln!("[mir][hint] ScopeEnter({})", id),
            HintKind::ScopeLeave(id) => eprintln!("[mir][hint] ScopeLeave({})", id),
            HintKind::Defer(calls) => eprintln!("[mir][hint] Defer({})", calls.join(";")),
            HintKind::JoinResult(var) => eprintln!("[mir][hint] JoinResult({})", var),
            HintKind::LoopCarrier(vars) => eprintln!("[mir][hint] LoopCarrier({})", vars.join(",")),
            HintKind::LoopHeader => eprintln!("[mir][hint] LoopHeader"),
            HintKind::LoopLatch => eprintln!("[mir][hint] LoopLatch"),
            HintKind::NoEmptyPhi => eprintln!("[mir][hint] NoEmptyPhi"),
        }
    }
    #[inline] pub fn scope_enter(&mut self, id: u32) { self.record(HintKind::ScopeEnter(id)); }
    #[inline] pub fn scope_leave(&mut self, id: u32) { self.record(HintKind::ScopeLeave(id)); }
    #[inline] pub fn defer_calls<S: Into<String>>(&mut self, calls: impl IntoIterator<Item = S>) {
        self.record(HintKind::Defer(calls.into_iter().map(|s| s.into()).collect()))
    }
    #[inline] pub fn join_result<S: Into<String>>(&mut self, var: S) { self.record(HintKind::JoinResult(var.into())); }
    #[inline] pub fn loop_carrier<S: Into<String>>(&mut self, vars: impl IntoIterator<Item = S>) {
        self.record(HintKind::LoopCarrier(vars.into_iter().map(|s| s.into()).collect()))
    }
    #[inline] pub fn loop_header(&mut self) { self.record(HintKind::LoopHeader); }
    #[inline] pub fn loop_latch(&mut self) { self.record(HintKind::LoopLatch); }
    #[inline] pub fn no_empty_phi(&mut self) { self.record(HintKind::NoEmptyPhi); }
}
