# Phase‑15.7 — Single Route, Single Face (Collections/Plugins)

## Goal
- Collapse initialization, routing, and semantics into single, well‑defined boxes.
- Remove ad‑hoc branching and duplicate init; keep Fail‑Fast and observability.

## Structure (target)
- PluginBootBox: one init path (OnceLock). Config ingest → loader → providers.
- MethodRouterBox: receiver → Resolver(TypeRegistry) → Invoker(builtin/plugin).
- TypeRegistry: single source of truth for (type_id, method_id, arity) + slots.
- Core crates: hako_core_string/array/map → only place for semantics.
- HostAPIBox: reverse‑call (slot/name) with grow logic and error policy.
- EnvGateBox: HAKO_/NYASH_ aliases and truthy/values normalization.
- SpecIngestBox: nyash_box.toml/hako_box.toml ingestion facade.
- DiagnosticsBox: structured JSON logs (dev aid; filterable).
- HandleCacheBox: (type_id, instance_id) → Arc reuse for identity.

## Milestones (this patch)
- Runner: remove legacy secondary plugin init; PluginBootBox is the single entry.
- Legacy VM: remove early Array.slice intercept; route through one path.
- Router: MethodRouterBox now invokes PluginBox v2 via plugin host; builtin path remains via builtin executor for now.
- Array.slice Stage‑2 kept behind env `NYASH_PLUGIN_ARRAY_SLICE_HANDLE=1` (HostHandle return). Build with `HAKO_EXPORT_HOST=1`.

## Next steps
1) Elevate Router: move builtin path to Router; interpreter calls Router only.
2) TypeRegistry: wire IDs for Builder/VM/LLVM parity; facade old registry.
3) HostAPI/EnvGate adoption: replace direct C ABI/env reads across runtime.
4) Core semantics as single truth: enforce slice/indexOf/… policies in core and reflect in docs/Builder lowering.
5) Strict plugin‑on: ensure NewBox→birth resolves providers deterministically; add re‑probe util.

## Testing
- plugin‑on quick/selfhost stays green for collections; plugin‑off quick validates Extern lowering for string builtins.
