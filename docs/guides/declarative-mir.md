# Declarative MIR with Map/Array Literals + JSON.stringify

Goal
- Build MIR(JSON v0) declaratively using Map/Array literals, then stringify via runtime JSON (Hakorune JSONBox). This replaces fragile string concatenation and reduces Builder boilerplate.

Status
- Default behavior unchanged. This guide shows the preferred style for new code. Existing emitters can migrate gradually.
- Pipeline V2 uses this style behind a dev flag.

Key Idea
- Author MIR as a nested Map/Array structure, then call `JSON.stringify(any)` or `.toJSON()` to obtain a JSON string.

Example: const → ret
```nyash
local mir = {
  functions: [{
    name: "main",
    params: [],
    blocks: [{
      id: 0,
      instructions: [
        { op: "const", dst: 1, value: { type: "i64", value: 42 } },
        { op: "ret", value: 1 }
      ]
    }]
  }]
}
local json = JSON.stringify(mir)  # First-class stringify (nested-safe)
```

Example: binop
```nyash
local kind = "Add"
local mir = {
  functions: [{ name: "main", params: [], blocks: [
    { id: 0, instructions: [
      { op: "const", dst: 1, value: { type: "i64", value: 2 } },
      { op: "const", dst: 2, value: { type: "i64", value: 5 } },
      { op: "binop", op_kind: kind, lhs: 1, rhs: 2, dst: 3 },
      { op: "ret", value: 3 }
    ] }
  ] }]
}
local json = JSON.stringify(mir)
```

Example: compare with minimal CFG
```nyash
local cmp = "Gt"
local mir = {
  functions: [{ name: "main", params: [], blocks: [
    { id: 0, instructions: [
      { op: "const", dst: 1, value: { type: "i64", value: 5 } },
      { op: "const", dst: 2, value: { type: "i64", value: 4 } },
      { op: "compare", cmp: cmp, lhs: 1, rhs: 2, dst: 3 },
      { op: "branch", cond: 3, "then": 1, "else": 2 }
    ] },
    { id: 1, instructions: [
      { op: "const", dst: 6, value: { type: "i64", value: 1 } },
      { op: "jump", target: 3 }
    ] },
    { id: 2, instructions: [
      { op: "const", dst: 6, value: { type: "i64", value: 0 } },
      { op: "jump", target: 3 }
    ] },
    { id: 3, instructions: [ { op: "ret", value: 6 } ] }
  ] }]
}
local json = mir.toJSON()
```

Notes
- Reserved words: when using keys like `then` or `else`, prefer quoting ("then"/"else").
- Map/Array literals support identifier keys and trailing commas; nested structures are allowed.
- `JSON.stringify(any)` is first-class and delegates to the same robust runtime stringify.
- `.toJSON()` is still available on MapBox/ArrayBox; both produce identical output.

Runtime JSON vs Ny helpers
- Ny helpers `JSON.stringify_map/array` remain available for legacy samples.

Migration Tips
- Replace string concatenation or ad‑hoc builders with declarative Map/Array literals.
- For Stage‑1→MIR lowerers, keep structure in Ny and stringify once at the boundary.
- Keep Builder around as a fallback during migration; remove gradually after parity smokes are green.

Dev Flags / Smokes
- Pipeline V2 examples are guarded by `NYASH_PIPELINE_V2=1` (dev only). See tools/smokes/v2/profiles/quick/core/selfhost_pipeline_v2_cmp_vm.sh.
- JSON stringify is now first-class. Legacy dev bridge docs remain for historical context; see tools/smokes/v2/profiles/quick/core/json_stringify_standard_vm.sh.
