Scope Hints (No-Op, Design Notes)

Purpose
- Provide zero-cost markers from front/macro to MIR builder so later passes can validate/control PHI placement and scope structure without changing semantics.

Current Hints (src/mir/hints.rs)
- HintKind::LoopHeader(id): at loop header block begin
- HintKind::LoopLatch(id): at loop backedge/latch
- HintKind::ScopeEnter(id), ScopeLeave(id): lexical scope begin/end
- HintKind::JoinResult(var): for if-join when both branches assign the same var

Producers (wired, no-op)
- Loop builder: emits LoopHeader/LoopLatch
- If builder: emits JoinResult for simple same-var assignments

Planned Injection Points
- Macro-normalized If/Match: emit JoinResult when we build branch assignments into the same LHS
- Statement blocks: on entering/leaving Local groups, emit ScopeEnter/Leave (id may be per-Program statement index)
- Function bodies: ScopeEnter at entry, ScopeLeave before Return/End

Policy
- Hints do not affect codegen; they are purely observational until a validator consumes them.
- Keep IDs stable within a single compilation unit; no cross-unit meaning required.

Next
- Wire ScopeEnter/Leave in MirBuilder for function entry/exit and block constructs.
- Add a simple debug dump when NYASH_MIR_TRACE_HINTS=1.

