# MIR PHI Invariants

Note
- Phase‑15 では PHI‑on が既定だよ。この資料の不変条件は MIR ビルダーが生成する PHI と、レガシーで `NYASH_MIR_NO_PHI=1` を指定したときに LLVM が補完するケースの両方へ適用するよ。詳しくは `phi_policy.md` を参照してね。

Scope: Builder/Bridge, PyVM, llvmlite (AOT)

Goal: Ensure deterministic PHI formation at control-flow merges so that
PyVM and LLVM backends agree for nested short-circuit, loop-if merges,
and chained ternary expressions.

Invariants
- If-merge ordering: Record incoming as [then, else] in this order when
  both branches reach the merge. When a branch is structurally absent,
  synthesize a carry-over from the pre-merge value.
- Loop latch snapshot: The latch (backedge) snapshot must be taken after
  per-iteration merges (i.e., after any phi binding for variables assigned
  in the loop body or nested if). Builder must bind the merged value to the
  loop-carried variable map before capturing the end-of-body state.
- Self-carry handling: A PHI with self-carry is allowed only when there is
  at least one non-self incoming. At finalize, map self-carry to the most
  recent non-self source visible at the predecessor end.

Representative Cases
- Nested short-circuit: `a && (b || c)` with selective assignments in nested
  branches. Expect single-eval per operand and deterministic merge order.
- Loop + if merge: A running sum updated in only one branch inside a while
  loop. Expect the latch to capture the phi-merged value, not a pre-merge
  temporary.
- Chained ternary: `cond1 ? (cond2 ? x : y) : z`. Expect linearized branches
  with merge ordering preserved at each join.

Diagnostics
- Enable `NYASH_LLVM_TRACE_PHI=1` to record per-block snapshots and PHI
  wiring in the LLVM path.
- Bridge verifier may allow `verify_allow_no_phi()` in PHI-off mode, but
  the invariants above still apply to resolver synthesis order.

Dev-only verification
- Enable `NYASH_VERIFY_PHI_STRICT=1` to verify that each PHI's inputs cover all
  reachable predecessors of its block. Missing inputs are reported as `InvalidPhi`.
- Quick profile enables this by default to catch builder regressions early.

## JSON Encoding (PHI values)

- Output format is unified to `values[]` only. Do not emit legacy `incoming`.
- Each entry records `{ "value": <ValueId>, "block": <BlockId> }`.
- Readers must prefer `values`; accepting `incoming` remains optional for
  backward compatibility during the transition.

Example
```json
{
  "kind": "phi",
  "dst": 6,
  "values": [
    { "value": 4, "block": 1 },
    { "value": 5, "block": 2 }
  ]
}
```
