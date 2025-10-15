# Unified Method Resolution — Design Note (Phase P4)

Purpose
- Document the unified pipeline for method resolution and how we will roll it out safely.
- Make behavior observable (dev-only) and gate any future default changes behind clear criteria.

Goals
- Single entry for all method calls via `emit_unified_call`.
- Behavior-preserving by default: Unknown/core/user‑instance receivers route to BoxCall.
- Known receivers may be rewritten to function calls (obj.m → Class.m(me,…)) under strict conditions.
- Keep invariants around SSA and instruction order to prevent sporadic undefined uses.
 - VM policy is explicit: user Instance BoxCall is a development fallback only; production defaults to disallow and expects builder rewrite.

String‑like APIs (ownership rule)
- Methods `length | len | substring | indexOf | lastIndexOf` belong to `StringBox`.
- Builder normalization:
  - If receiver hint/origin/type is `InstanceBox`/`ParserBox`/`DebugBox`/`FileBox` but method is string‑like, normalize receiver class to `StringBox` for resolution.
  - This prevents accidental dispatch to instance methods that do not exist and removes the need for VM fallbacks.
- Router policy (structural guard):
  - For `InstanceBox × string‑like` calls, prefer `Unified` route (not `BoxCall`). This lets inference land on `StringBox.*` reliably and keeps behavior stable across backends.

Pipeline (concept)
1) Entry: `emit_unified_call(dst, CallTarget::Method { box_type, method, receiver }, args)`
2) Special rewrites (early): toString/stringify → str, equals/1 consolidation.
3) Known/unique rewrite (user boxes only): if class is Known and a unique function exists, rewrite to `Call(Class.m/arity)`.
4) Routing: `RouterPolicy.choose_route` decides Unified vs BoxCall
   - Unknown/core/user‑instance → BoxCall（conservative）
   - Exception (string‑like): `InstanceBox × {length,len,substring,indexOf,lastIndexOf}` → Unified（normalize to `StringBox`）
5) Emit guard: LocalSSA finalize (recv/args in current block) + BlockSchedule order contract (PHI → Copy → Call).
6) MIR emit: `Call { callee=Method/Extern/Global }` or `BoxCall` as routed.

VM policy (Instance BoxCall)
- Rule: user Instance BoxCall is for development diagnostics only; production must not rely on it.
- Env: `NYASH_VM_USER_INSTANCE_BOXCALL={0|1}` (default: dev/ci=true, prod=false)
- Builder responsibility: perform Instance→Function rewrite for user boxes whenever `(Box,method,arity)` exists uniquely. When rewrite is not possible, RouterPolicy safely falls back to BoxCall and the VM may allow it only in dev.
 - Temporary dev valve (default OFF): `NYASH_VM_STRLIKE_INSTANCE_COERCE=1` may coerce instance receivers to string for string‑like calls during bring‑up. Remove when normalization is robust.

Invariants (dev-verified)
- SSA locality: All operands are materialized within the current basic block before use.
- Order: PHI group at block head, then materialize Copies, then body (Calls). Verified with `NYASH_BLOCK_SCHEDULE_VERIFY=1`.
- Rewrites do not change semantics: Known rewrite only when a concrete target exists and is unique for the arity.

Behavior flags (existing)
- `NYASH_ROUTER_TRACE=1`: short route decisions to stderr (reason, class, method, arity, certainty).
- `NYASH_LOCAL_SSA_TRACE=1`: LocalSSA ensure/finalize traces (recv/arg/cond/cmp).
- `NYASH_BLOCK_SCHEDULE_VERIFY=1`: warn when Copy/Call ordering does not follow the contract.
- KPI (dev-only):
  - `NYASH_DEBUG_KPI_KNOWN=1` → aggregate Known rate for `resolve.choose`.
  - `NYASH_DEBUG_SAMPLE_EVERY=N` → sample output every N events.

Flag (P4)
- `NYASH_REWRITE_KNOWN_DEFAULT` (default ON; set to 0/false/off to disable):
  - Enables Known→function rewrite by default for user boxes if and only if:
    - receiver is Known (origin), and
    - function exists, and
    - candidate is unique for the arity.
  - When disabled, behavior remains conservative; routing still handles BoxCall fallback.

Rollout note
- Default is ON with strict guards; set `NYASH_REWRITE_KNOWN_DEFAULT=0` to revert to conservative behavior.
- Continue to use `NYASH_ROUTER_TRACE=1` and KPI sampling to validate stability during development.

Key files
- Entry & routing: `src/mir/builder/builder_calls.rs`, `src/mir/builder/router/policy.rs`
- Rewrites: `src/mir/builder/rewrite/{special.rs, known.rs}`
- SSA & order: `src/mir/builder/ssa/local.rs`, `src/mir/builder/schedule/block.rs`, `src/mir/builder/emit_guard/`
- Observability: `src/mir/builder/observe/resolve.rs`

Acceptance for P4
- quick/integration stay green with flags OFF.
- With flags ON (dev), green remains; KPI reports sensible Known rates without mismatches.
- No noisy logs in default runs; all diagnostics behind flags.

Notes
- This design keeps Unknown/core/user‑instance on BoxCall for stability and parity with legacy behavior.
- Known rewrite is structurally safe because user box methods are lowered to standalone MIR functions during build.
 - VM enforcement (prod): instance-dispatch on user boxes is disabled by default so rewrite leaks are caught early in non‑dev runs.
