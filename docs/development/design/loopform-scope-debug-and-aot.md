# Loop‑Form Scopes Debug & AOT — Design (PoC → Stabilize)

Purpose
- Make method resolution (rewrite) and SSA/PHI decisions observable per structured scope (Loop/Join), then derive AOT needs/provides from the same structure. Keep it dev‑only by default, zero‑cost when off.

Scope Model (structured on Loop‑Form)
- Hierarchy: ProgramScope → FunctionScope → RegionScope*
- RegionScope kinds (only two):
  - LoopScope: preheader → header(φ) → body → latch → {header|exit}
  - JoinScope: if/elseif/else diamond join (then_exit, else_exit → join)
- No BlockScope (too fine‑grained). Region level is sufficient for diagnostics + AOT.

Invariants (builder verifies in dev)
- LoopScope
  - header φ has exactly two incomings: {preheader, latch}
  - backedge latch → header exists; critical edges are split
- JoinScope
  - join φ has incomings: {then_exit, else_exit}; when else is missing, else takes pre‑if value
- Pin completeness
  - Branch conditions, compare operands, and field receivers are slotified via `ensure_slotified_for_use`.

Events & Hub (single sink)
- DebugHub (central)
  - enable/disable kinds, set level, sink(file)
  - format: JSONL per event
    - common fields: `ts, phase(builder|vm|llvm), fn, region_id, cat(resolve|ssa|op|aot), kind, meta{...}`
  - metrics (dev only): `fallback_count, reentry_guard_hits, phi_pred_mismatch, birth_me_void_hits …`
  - env knobs (proposal):
    - `NYASH_DEBUG_ENABLE=1` (master gate)
    - `NYASH_DEBUG_KINDS=resolve,ssa[,op,aot]`
    - `NYASH_DEBUG_SINK=tmp/nyash_debug.jsonl`
    - `NYASH_DEBUG_RATE=1000/s` (optional), `NYASH_DEBUG_SAMPLE=0.1` (optional)

Inspectors (boxes/components)
- ResolveInspector (method resolution)
  - emit:
    - `resolve.try {recv_cls?, inferred_cls?, method, arity, candidates, unique}`
    - `resolve.choose {chosen, reason}`
    - `materialize.func {name, present_before, present_after}`
    - `module.index {function_count}` (coarse)
  - API (internal): `explain(recv, m, n), has(name), functions(prefix?)`

- SSAInspector (PHI and invariants)
  - emit:
    - `ssa.phi {dst, preds:[{bb,v,type,origin}], decided_type?, decided_origin?}`
    - `ssa.verify {rule, ok, detail}` for Loop/Join invariants

- OperatorInspector (later; dev only)
  - emit:
    - `op.apply {op, lhs_type, rhs_type, result_type, adopted?, fallback?, guard?}`

Optional (later)
- ExpressionBox: instrumented expression tree for a target function (heavy; function‑filtered)
- ProbeBox: dynamic proxy for object method observation (dev only)

RegionScope State (minimal)
- `env`: `ValueId → {type, origin_cls, pinned_slot?}` (surface at region boundaries)
- `calls_seen`: `Vec<CalleeSig>` where CalleeSig = UserMethod{cls,name,arity} | Plugin{box,method_id|name} | Global
- `phis`: `dst → {preds:[{bb,v,type,origin}], decided_type?, decided_origin?}`
- `rewrite_log`: `Vec<{recv_cls?, method, arity, candidates, chosen, reason}>`
- AOT roll‑up:
  - `requires_local`: referenced functions in this region (from calls)
  - `provides_local`: materialized functions in this region (from lowering)

Deriving AOT (natural path)
- Fold RegionScope → FunctionScope → ProgramScope:
  - `requires += child.requires - child.provides`
  - The folded graph gives a call graph; compute SCC → compile/link units and order
- Output (example `plan.json`):
```json
{
  "units": [
    {"scc": ["Foo.a/1","Foo.b/1"], "order": 0},
    {"scc": ["Main.main/0"], "order": 1}
  ],
  "plugins": ["StringBox.substring/2"]
}
```

Builder/VM Attachment Points (minimal)
- ScopeCtx stack in builder: `enter_scope(kind,id)`, `exit_scope()` at Loop/Join construction points
- Method resolution decisions: `src/mir/builder/method_call_handlers.rs` (try/choose, toString→stringify)
- PHI capture (+dev meta propagation): `src/mir/builder.rs: emit_instruction(Phi)`
- Materialize hooks: `lower_method_as_function` end, and/or module finalize (counts)
- Operator (optional Stage 2): `src/mir/builder/ops.rs` (Compare/Add/stringify)

Event Examples
```json
{"ts":"…","phase":"builder","fn":"Foo.main/0","region_id":"loop#12/header","cat":"ssa","kind":"phi","meta":{"dst":63,"preds":[{"bb":2067,"v":57,"type":"i64","origin":"Foo"},{"bb":2070,"v":62,"type":"i64","origin":"Const"}],"decided_type":"i64","decided_origin":"merge(Foo,Const)"}}

{"ts":"…","phase":"builder","fn":"Foo.main/0","region_id":"join#7/join","cat":"resolve","kind":"choose","meta":{"recv_cls":"Foo","method":"add2","arity":2,"candidates":["Foo.add2/2"],"unique":true,"chosen":"Foo.add2/2","reason":"unique-suffix"}}
```

PoC Plan (phased)
1) Phase‑1: Hub + Resolve/SSA (dev only; default OFF)
   - Add ScopeCtx stack (enter/exit at Loop/Join points)
   - Fire `resolve.try/choose` and `ssa.phi` events; keep existing dev logs intact
   - Smokes: run userbox_branch_phi_vm.sh, userbox_method_arity_vm.sh with `NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_SINK=…`

2) Phase‑2: Operator + materialize/module + AOT fold
   - Add OperatorInspector (Compare/Add/stringify)
   - Emit `materialize.func` and `module.index`; collect requires/provides per region
   - Fold to `plan.json` (AOT units order), dev only

3) Phase‑3: Options
   - ExpressionBox (function‑filtered) and ProbeBox (dev only)

Acceptance (PoC)
- Debug JSONL contains resolve/ssa lines with `region_id` and decisions
- SKIP中のケース（分岐/アリティ）で「どのregionで落ちたか」がログで一発特定可能
- 既存PASSケースは挙動不変（既定OFF）

Risks & Mitigations
- Log volume: kinds/level filters, sampling (`NYASH_DEBUG_SAMPLE`), rate limit (`NYASH_DEBUG_RATE`)
- Observation changing meaning: emit after values are finalized; keep zero‑cost OFF
- Scope drift: single Hub + 3 inspectors; optional boxes are late‑binding and dev‑only

