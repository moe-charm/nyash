# Registry SSOT Plan (slots / arity / aliases)

Goal
- Maintain one single source of truth (SSOT) for TypeBox method slots, arities, and aliases.
- Runtime reads the table; builders/routers reference it; tests validate it.

Principles
- SSOT lives in a compact, audit-friendly TOML (or minimal Rust table) driven by build.rs codegen.
- vm_ops is read-only; no hard-coded per-type arity strings.
- Diagnostics use centralized helpers to produce consistent messages.

Initial Steps (Phase 15.7)
- Centralize diagnostics: arity_guard_for() and unknown_method_err() (vm_ops/boxcall) — done.
- Add smoke tests to lock VM↔LLVM op_eq parity at boundaries — done (integration-core).
- Author this doc to set direction.

Runtime Status (implemented)
- specs/type_registry.toml is embedded at build-time as REGISTRY_SSOT_RAW (OUT_DIR/registry_ssot_generated.rs).
- At runtime, type_registry parses REGISTRY_SSOT_RAW lazily into OnceCell (toml → HashMap).
- resolve_slot_by_name / known_arities_for prefer SSOT; fallback to builtin static tables.
- Diagnostics unification:
  - msg::no_method_arity for arity errors
  - msg::unknown_slot for unknown slot errors (router)

Tests (integration-core)
- vm_llvm_op_eq_primitives_core.sh — Eq/Ne baseline
- vm_llvm_op_eq_box_reflexive_core.sh — BoxRef reflexive (=)
- vm_llvm_compare_le_ge_boundary_core.sh — <=/>= integer boundary
- vm_llvm_compare_le_ge_float_boundary_core.sh — <=/>= float boundary
- vm_llvm_json_stringify_boundary_core.sh — ArrayBox.toJSON simple parity

Migration Plan
1) Author `specs/type_registry.toml` with entries:
   - type_name, type_id, methods[{ name, slot, arities[] , aliases[] }]
2) build.rs generates a Rust module consumed by `runtime/type_registry`.
3) Replace ad-hoc error strings with diagnostics::msg::no_method_arity() in routers.
4) Keep router thin: resolve → preflight (unborn/arity) → dispatch.

Acceptance
- quick: unchanged; rc-only stability.
- integration-core: new parity tests pass.
- plugins: identity path unaffected.

Rollback
- Codegen gated; fallback to current runtime table if spec missing.
