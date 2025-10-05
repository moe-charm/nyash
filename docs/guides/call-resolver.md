# CallResolver — Global→ModuleFunction name resolution (VM fallback)

Purpose
- Centralize ambiguous Global callee resolution into a single, testable module.
- Make VM fallback robust when Builders emit Global("Box.method/Arity") while
  actual functions are materialized as ModuleFunction (possibly alias‑prefixed).

Location
- Shared core: `src/mir/resolve/call_resolver_core.rs` (SSOT)
- VM wrapper: `src/backend/mir_interpreter/resolve/call_resolver.rs`
- Used by: `handlers/calls/function.rs::handle_callee_global`, Builder (ModuleFunction fallback)

Strategy (ordered)
1. Exact match
2. Arity append when missing: `name` → `name/argc`
3. Tail match: keys ending with `.method/argc` and starting with `Class.` or `Class_`
4. Alias‑alias heuristic: `Alias_Alias.method/argc`
5. Final fallback: any unique key that ends with `.method/argc`

Notes
- SSOT: The shared MIR core is used by VM and Builder to avoid drift.
- Call sites normalize raw names by stripping trailing `"/N"` before resolving.
- Behavior remains conservative: ambiguous matches are not auto‑picked by Builder; VM keeps the same strategy and emits a clear error on unknown names.

Testing
- Unit tests live next to the module (exact/append/tail). Wider tests are covered
  by smokes under `tools/smokes/v2/profiles/quick/apps/`.

Next
- Optional trace (dev‑only): `NYASH_VM_RESOLVE_TRACE=1` to emit one‑line JSON
  showing `raw → pick` for difficult cases.
- Consider sharing the resolver in Builder for ModuleFunction lowering.
