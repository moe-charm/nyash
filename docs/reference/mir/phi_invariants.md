# MIR PHI Invariants

Note
- Default policy is PHI‑off at MIR level. These invariants apply to the dev‑only PHI‑on mode and to how LLVM synthesizes PHIs from predecessor copies. See also `phi_policy.md`.

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
