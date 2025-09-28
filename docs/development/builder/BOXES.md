# MIR Builder — Boxes Catalog (Phase 15.7)

Purpose
- Consolidate scattered responsibilities into small, focused “boxes” (modules) with clear APIs.
- Reduce regression surface by centralizing invariants and repeated patterns.
- Keep behavior unchanged (default-off for any new diagnostics). Adopt gradually.

Status (2025-09-28)
- S-tier (landed skeletons):
  - MetadataPropagationBox — type/origin propagation.
  - ConstantEmissionBox — Const emission helpers.
  - TypeAnnotationBox — minimal return-type annotation for known calls.
- S-tier (new in this pass):
  - RouterPolicyBox — route decision (Unified vs BoxCall).
  - EmitGuardBox — emit-time invariants (LocalSSA finalize + schedule verify).
  - NameConstBox — string Const for function names.
- A/B-tier: planned; do not implement by default.

Call Routing — Unification (2025‑09‑28)
- Standard method calls now delegate to `emit_unified_call` (single entry).
  - Receiver class hint (origin/type) is resolved inside unified; handlers no longer duplicate it.
  - RouterPolicy decides Unified vs BoxCall. Unknown/core/user‑instance → BoxCall (behavior‑preserving).
  - Rewrites apply centrally: `rewrite::special` (toString/stringify→str, equals/1) and `rewrite::known` (Known→function).
  - LocalSSA + BlockSchedule + EmitGuard enforce PHI→Copy→Call ordering and in‑block materialization.

Structure
```
src/mir/builder/
├── metadata/propagate.rs         # MetadataPropagationBox
├── emission/constant.rs          # ConstantEmissionBox
├── emission/compare.rs           # CompareEmissionBox (new)
├── emission/branch.rs            # BranchEmissionBox (new)
├── types/annotation.rs           # TypeAnnotationBox
├── router/policy.rs              # RouterPolicyBox
├── emit_guard/mod.rs             # EmitGuardBox
└── name_const.rs                 # NameConstBox
```

APIs (concise)
- metadata::propagate(builder, src, dst)
- metadata::propagate_with_override(builder, dst, MirType)
- emission::constant::{emit_integer, emit_string, emit_bool, emit_float, emit_null, emit_void}
- emission::compare::{emit_to, emit_eq_to, emit_ne_to}
- emission::branch::{emit_conditional, emit_jump}
- types::annotation::{set_type, annotate_from_function}
- router::policy::{Route, choose_route(box_name, method, certainty, arity)}
- emit_guard::{finalize_call_operands(builder, &mut Callee, &mut Vec<ValueId>), verify_after_call(builder)}
- name_const::{make_name_const_result(builder, &str) -> Result<ValueId, String>}

Adoption Plan (behavior-preserving)
1) Replace representative Const sites with `emission::constant`.
2) Replace ad-hoc type/origin copy with `metadata::propagate`.
3) Call `types::annotation` where return type is clearly known (string length/size/str etc.).
4) Use `router::policy::choose_route` in unified call path; later migrate utils’ prefer_legacy to it.
5) Use `emit_guard` to centralize LocalSSA finalize + schedule verify around calls; later extend to branch/compare.
6) Use `name_const` in rewrite paths to reduce duplication.

Diagnostics
- All new logs remain dev-only behind env toggles already present (e.g., NYASH_LOCAL_SSA_TRACE, NYASH_BLOCK_SCHEDULE_VERIFY).
 - Router trace: `NYASH_ROUTER_TRACE=1` prints route decisions (stderr, short, default OFF).

Guardrails
- Behavior must remain unchanged; only refactors/centralizations allowed.
- Keep diffs small; validate `make smoke-quick` and `make smoke-integration` stay green at each step.
