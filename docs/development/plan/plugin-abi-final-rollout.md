# Plugin ABI (Final) — Rollout Plan

Status: plan only (no default changes). Targets plugins boundary; user Boxes unaffected.

Principles
- Additive, default OFF, per‑flag gating. Backward compatible with v2.
- Enforce only in dev/ci first; prod enforcement is a future switch.

Phases
1) Phase A — Minimal Core
   - NyResult invoke support in runtime (prefer when `NYASH_PLUGIN_ABI_FINAL=1`, else fallback to v2.
   - Meta functions accepted (NULL tolerant): `get_method_meta/get_all_methods/get_type_info`.
   - Docs + PoC plugin (FileBox) updated; legacy plugins unaffected.

2) Phase B — Observability
   - Capability/effect/contract fields probed when present.
   - Logging only behind flags: `NYASH_TRACE_EFFECTS=1`, `NYASH_CHECK_CONTRACTS=1`.
   - `NYASH_PLUGIN_CAPS_ENFORCE=1` enforces required_capabilities (dev/ci recommended).

3) Phase C — Resolve/Invoke unification
   - Remove `method()` fallback; use `resolve`+`invoke` only (compat flag keeps fallback during transition).

4) Phase D — Component Model (optional)
   - Accept `component_info` and dump WIT schema on request; no default behavior changes.

Acceptance
- PoC plugin works end‑to‑end via NyResult path with v2 fallback.
- Meta queries do not break legacy plugins; logs only when enabled.
- Capability enforcement passes in dev/ci; prod remains unchanged.

Rollback
- Flip envs OFF to fully restore v2 behavior.
- Changes are small, scoped to plugin loader/invoker; revert per‑file.
