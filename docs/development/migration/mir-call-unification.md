MIR Call Unification — Migration Plan (Phase 15.7 → 16)

Goal
- Freeze the MIR instruction set around MirCall (Callee) and retire legacy call forms.

Scope
- Builders, printer, verifier, VM/LLVM backends, and optimizer passes.

Steps
1) Builder emits MirCall by default
   - Replace direct `MirInstruction::Call/BoxCall/ExternCall/NewBox/NewClosure` emission with `MirCall` construction.
   - For compatibility, use `definitions::call_unified::migration::*` helpers to convert legacy forms to MirCall immediately after emission.

2) VM backend dispatch via Callee only
   - Introduce a thin adapter in the call path that converts legacy call-like instructions to MirCall, then match on Callee.
   - Keep behavior identical; add LocalSSA materialization just before the call.

3) Printer/Verifier/Optimizer
   - Printer: render MirCall first. Legacy branches go through MirCall internals to ensure consistent pretty-print.
   - Verifier: add checks for constructor flags/effects and receiver invariants.
   - Optimizer: strengthen legacy diagnostics. Gate with env flags.

4) Tests and smokes
   - Golden cases that cover: Global/Method/Constructor/Closure/Extern.
   - Legacy forbid job (opt-in): set `NYASH_OPT_DIAG_FORBID_LEGACY=1` and expect zero occurrences.

5) Deletion of legacy forms (post-green)
   - Remove PluginInvoke first (fully represented by Method).
   - Migrate NewBox/NewClosure to Constructor/Closure then remove the legacy instructions.
   - Finally, collapse Call/BoxCall/ExternCall into the MirCall surface entirely.

Guard rails
- Phase-by-phase removal with small diffs; enable quick revert.
- Behavior is unchanged; only representation and dispatch become uniform.

References
- Spec: docs/reference/mir/INSTRUCTION_SET.md
- Unified call: docs/reference/mir/call-unified.md
- Code: src/mir/instruction.rs, src/mir/definitions/call_unified.rs

Phases (recommended)

- Phase 1 (low risk, quick wins)
  - Keep `NYASH_MIR_UNIFIED_CALL=0` by default.
  - Populate `callee` only for Builtins/Extern where resolution is unambiguous.
  - Add a VM adapter function to convert legacy BoxCall/NewBox/ExternCall/NewClosure to Callee (guarded and opt‑in).
  - Keep module functions on legacy NameConst resolution; do not set `callee` for them yet.

- Phase 2 (module function unification)
  - Introduce `Callee::ModuleFunction(String)` for user/module functions (e.g., `Counter.inc/0`).
  - Builder: emit `ModuleFunction` when lowering known module functions.
  - VM: implement `resolve_module_function(name, args)` via the function table (exact match; arity suffix).
  - Begin pruning legacy NameConst paths once green.

- Phase 3 (cleanup)
  - Remove PluginInvoke (fully represented by Method with policy) and shrink legacy branches.
  - Keep Copy/Nop/Safepoint as meta/structural ops; no behavior change.
