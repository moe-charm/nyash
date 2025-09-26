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

    fn cfg() -> HintCfg {
        // New unified env: NYASH_MIR_HINTS="<target>|<filters>"
        // Examples:
        //   NYASH_MIR_HINTS=trace|all              -> stderr + all kinds
        //   NYASH_MIR_HINTS=tmp/hints.jsonl|loop   -> jsonl file + loop-only
        //   NYASH_MIR_HINTS=jsonl=tmp/h.jsonl|scope|join
        // Back-compat: NYASH_MIR_TRACE_HINTS=1 -> stderr + all kinds
        if let Ok(spec) = std::env::var("NYASH_MIR_HINTS") {
            return HintCfg::parse(&spec);
        }
        if std::env::var("NYASH_MIR_TRACE_HINTS").ok().as_deref() == Some("1") {
            return HintCfg { sink: HintSinkTarget::Stderr, kinds: HintKinds::All };
        }
        HintCfg { sink: HintSinkTarget::None, kinds: HintKinds::None }
    }

    #[inline]
    pub fn record(&mut self, hint: HintKind) {
        // Resolve config (env-based). Lightweight and robust; acceptable to parse per call.
        let cfg = Self::cfg();
        if matches!(cfg.sink, HintSinkTarget::None) { return; }
        // Filter kinds
        let k = hint_tag(&hint);
        if !cfg.kinds.contains(k) { return; }

        match cfg.sink {
            HintSinkTarget::None => {}
            HintSinkTarget::Stderr => {
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
            HintSinkTarget::Jsonl(ref path) => {
                // Append one JSON object per line. Best-effort; ignore errors.
                let _ = append_jsonl(path, &hint);
            }
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

// ---- Unified hint config parser ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HintTag { Scope, Join, Loop, Phi }

fn hint_tag(h: &HintKind) -> HintTag {
    match h {
        HintKind::ScopeEnter(_) | HintKind::ScopeLeave(_) | HintKind::Defer(_) => HintTag::Scope,
        HintKind::JoinResult(_) => HintTag::Join,
        HintKind::LoopCarrier(_) | HintKind::LoopHeader | HintKind::LoopLatch => HintTag::Loop,
        HintKind::NoEmptyPhi => HintTag::Phi,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HintKinds { None, Some { scope: bool, join: bool, loopk: bool, phi: bool }, All }

impl HintKinds {
    fn contains(&self, tag: HintTag) -> bool {
        match self {
            HintKinds::All => true,
            HintKinds::None => false,
            HintKinds::Some { scope, join, loopk, phi } => match tag {
                HintTag::Scope => *scope,
                HintTag::Join => *join,
                HintTag::Loop => *loopk,
                HintTag::Phi => *phi,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HintSinkTarget { None, Stderr, Jsonl(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
struct HintCfg { sink: HintSinkTarget, kinds: HintKinds }

impl HintCfg {
    fn parse(spec: &str) -> Self {
        let mut sink = HintSinkTarget::None;
        let mut kinds = HintKinds::None;
        let mut saw_filter = false;
        for tok in spec.split('|').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let tl = tok.to_ascii_lowercase();
            if tl == "off" { sink = HintSinkTarget::None; kinds = HintKinds::None; continue; }
            if tl == "trace" || tl == "stderr" { sink = HintSinkTarget::Stderr; continue; }
            if tl.starts_with("jsonl=") {
                sink = HintSinkTarget::Jsonl(tok[6..].trim().to_string());
                continue;
            }
            // Heuristic: token looks like a path → jsonl
            if tok.contains('/') || tok.contains('.') {
                sink = HintSinkTarget::Jsonl(tok.to_string());
                continue;
            }
            // Filters
            match tl.as_str() {
                "all" => { kinds = HintKinds::All; saw_filter = true; }
                "scope" => kinds = merge_kind(kinds, |k| HintKinds::Some { scope: true, join: matches!(k, HintKinds::Some{ join: true, .. } | HintKinds::All), loopk: matches!(k, HintKinds::Some{ loopk: true, .. } | HintKinds::All), phi: matches!(k, HintKinds::Some{ phi: true, .. } | HintKinds::All) }),
                "join" => kinds = merge_kind(kinds, |k| HintKinds::Some { scope: matches!(k, HintKinds::Some{ scope: true, .. } | HintKinds::All), join: true, loopk: matches!(k, HintKinds::Some{ loopk: true, .. } | HintKinds::All), phi: matches!(k, HintKinds::Some{ phi: true, .. } | HintKinds::All) }),
                "loop" => kinds = merge_kind(kinds, |k| HintKinds::Some { scope: matches!(k, HintKinds::Some{ scope: true, .. } | HintKinds::All), join: matches!(k, HintKinds::Some{ join: true, .. } | HintKinds::All), loopk: true, phi: matches!(k, HintKinds::Some{ phi: true, .. } | HintKinds::All) }),
                "phi" => kinds = merge_kind(kinds, |k| HintKinds::Some { scope: matches!(k, HintKinds::Some{ scope: true, .. } | HintKinds::All), join: matches!(k, HintKinds::Some{ join: true, .. } | HintKinds::All), loopk: matches!(k, HintKinds::Some{ loopk: true, .. } | HintKinds::All), phi: true }),
                _ => {}
            }
        }
        if !saw_filter && !matches!(kinds, HintKinds::All) {
            // default to all if no filter specified
            kinds = HintKinds::All;
        }
        // default sink if only filters appear
        if matches!(sink, HintSinkTarget::None) {
            sink = HintSinkTarget::Stderr;
        }
        HintCfg { sink, kinds }
    }
}

fn merge_kind<F: FnOnce(HintKinds) -> HintKinds>(k: HintKinds, f: F) -> HintKinds {
    match k {
        HintKinds::All => HintKinds::All,
        x => f(x),
    }
}

fn append_jsonl(path: &str, hint: &HintKind) -> std::io::Result<()> {
    use std::io::Write;
    let mut obj = serde_json::json!({ "kind": kind_name(hint) });
    match hint {
        HintKind::ScopeEnter(id) => obj["value"] = serde_json::json!({"enter": id}),
        HintKind::ScopeLeave(id) => obj["value"] = serde_json::json!({"leave": id}),
        HintKind::Defer(calls) => obj["value"] = serde_json::json!({"defer": calls}),
        HintKind::JoinResult(v) => obj["value"] = serde_json::json!({"join": v}),
        HintKind::LoopCarrier(vs) => obj["value"] = serde_json::json!({"carrier": vs}),
        HintKind::LoopHeader => obj["value"] = serde_json::json!({"loop": "header"}),
        HintKind::LoopLatch => obj["value"] = serde_json::json!({"loop": "latch"}),
        HintKind::NoEmptyPhi => obj["value"] = serde_json::json!({"phi": "no_empty"}),
    }
    let line = obj.to_string();
    if let Some(dir) = std::path::Path::new(path).parent() { let _ = std::fs::create_dir_all(dir); }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", line)?;
    Ok(())
}

fn kind_name(h: &HintKind) -> &'static str {
    match h {
        HintKind::ScopeEnter(_) => "ScopeEnter",
        HintKind::ScopeLeave(_) => "ScopeLeave",
        HintKind::Defer(_) => "Defer",
        HintKind::JoinResult(_) => "JoinResult",
        HintKind::LoopCarrier(_) => "LoopCarrier",
        HintKind::LoopHeader => "LoopHeader",
        HintKind::LoopLatch => "LoopLatch",
        HintKind::NoEmptyPhi => "NoEmptyPhi",
    }
}
