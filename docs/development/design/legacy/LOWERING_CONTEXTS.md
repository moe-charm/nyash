# Lowering Contexts (Boxing the API)

Motivation
- Current lowering functions carry too many parameters (10+), scattering responsibilities and making invariants (Resolver-only, localization discipline, dispatch-only PHI) hard to enforce.
- Following Nyash’s “Everything is Box” philosophy, we group related data into small, composable context boxes to create clean boundaries and make invariants structural.

Core Context Boxes
- LowerFnCtx<'ctx, 'b>
  - Holds: `codegen`, `func`, `cursor`, `resolver`, `vmap`, `bb_map`, `preds`, `block_end_values`, `phis_by_block`, optional `const_strs`, `box_type_ids`.
  - Utilities: `ensure_i64/ptr/f64` (Resolver-only), `with_block(..)`, `with_pred_end(..)`, `guard_post_terminator`.
  - Effect: All lowering entrypoints can accept a single `&mut LowerFnCtx`.

- BlockCtx<'ctx>
  - Holds: `cur_bid`, `cur_llbb` (from `bb_map`), optionally `succs`.
  - Role: The canonical site for “where to insert” decisions; pred-end casts are routed via this box.

- InvokeCtx
  - Holds: `method_id: u16`, `type_id: i64`, `recv_h: IntValue<'ctx>`, `args: &[ValueId]`.
  - Role: Unifies plugin invoke by-id/by-name into a single API surface.

- StringOps (lightweight types/helpers)
  - Types: `StrHandle(IntValue<'ctx>)`, `StrPtr(PointerValue<'ctx>)`.
  - Rule: across blocks keep `StrHandle`; convert to `StrPtr` only at call sites in the same BB; return values of string ops are kept as `StrHandle`.
  - Entry: `StringOps::concat(ctx, blk, lhs, rhs) -> LlResult<StrHandle>` etc.

API Sketch
```
pub type LlResult<T> = Result<T, String>;

pub struct LowerFnCtx<'ctx, 'b> { /* see above */ }
pub struct BlockCtx<'ctx> { pub cur_bid: BasicBlockId, pub cur_llbb: BasicBlock<'ctx> }
pub struct InvokeCtx<'ctx> { pub method_id: u16, pub type_id: i64, pub recv_h: IntValue<'ctx>, pub args: Vec<ValueId> }

impl<'ctx, 'b> LowerFnCtx<'ctx, 'b> {
  pub fn ensure_i64(&mut self, blk: &BlockCtx<'ctx>, v: ValueId) -> LlResult<IntValue<'ctx>> { /* Resolver-only */ }
  pub fn ensure_ptr(&mut self, blk: &BlockCtx<'ctx>, v: ValueId) -> LlResult<PointerValue<'ctx>> { /* i64 PHI -> inttoptr here */ }
  pub fn with_pred_end<R>(&mut self, pred: BasicBlockId, f: impl FnOnce(&Builder) -> R) -> R { /* insert before terminator */ }
}

pub fn lower_boxcall(ctx: &mut LowerFnCtx, blk: &BlockCtx, inst: &BoxCallInst) -> LlResult<()> { /* … */ }
pub fn try_handle_tagged_invoke(ctx: &mut LowerFnCtx, blk: &BlockCtx, call: &InvokeCtx) -> LlResult<()> { /* … */ }

pub mod string_ops {
  pub struct StrHandle<'ctx>(pub IntValue<'ctx>);
  pub struct StrPtr<'ctx>(pub PointerValue<'ctx>);
  pub fn concat(ctx: &mut LowerFnCtx, blk: &BlockCtx, lhs: ValueId, rhs: ValueId) -> LlResult<StrHandle> { /* … */ }
}
```

Invariants Enforced by Design
- Resolver-only: `VMap` is not exposed; all value access goes through `LowerFnCtx` utilities.
- Localization discipline: PHIs at BB head; casts at pred-end via `with_pred_end`.
- Strings handle rule: Only `StrHandle` crosses block boundaries; `StrPtr` generated only in the same BB at call sites.
- LoopForm rule: preheader mandatory, header condition built via Resolver; dispatch-only PHI. Dev guard checks enforce these.

Dev Guards (optional, recommended)
- `PhiGuard::assert_dispatch_only(&LowerFnCtx)` to fail fast when non-dispatch PHIs appear.
- `LoopGuard::assert_preheader(&LowerFnCtx)` to ensure preheader presence and header i1 formation point.
- CI Deny-Direct: `rg -n "vmap\.get\(" src/backend/llvm/compiler/codegen/instructions | wc -l` must be `0`.

Migration Plan
1) Introduce `LowerFnCtx`/`BlockCtx`/`InvokeCtx`; migrate `lower_boxcall` and invoke path first.
2) Move string ops to `StringOps` with handle-only rule; clean call sites.
3) Migrate BinOp/Compare/ExternCall to `LowerFnCtx + BlockCtx` API.
4) Turn on dev guards; remove remaining fallback paths; simplify code.

Acceptance
- Refactored entrypoints accept at most three boxed parameters.
- Deny-Direct passes (no direct `vmap.get` in lowering/instructions).
- Dominance: verifier green on representative functions (e.g., dep_tree_min_string).

## Context Stack Guide (If/Loop)

Purpose
- Make control-flow merge/loop boundaries explicit and uniform across builders (MIR) and the JSON v0 bridge.

Stacks in MirBuilder
- If merge: `if_merge_stack: Vec<BasicBlockId>`
  - Push the merge target before lowering branches, pop after wiring edge copies or Phi at the merge.
  - PHI-off: emit per-predecessor edge copies into the merge pred blocks; merge block itself must not add a self-copy.
  - PHI-on: place Phi(s) at the merge block head; inputs must cover all predecessors.
- Loop context: `loop_header_stack` / `loop_exit_stack`
  - Header = re-check condition; Exit = after-loop block.
  - `continue` → jump to Header; `break` → jump to Exit. Both add predecessor metadata from the current block.
  - Builder captures variable-map snapshots on `continue` to contribute latch-like inputs when sealing the header.

JSON v0 Bridge parity
- Bridge `LoopContext { cond_bb, exit_bb }` mirrors MIR loop stacks.
- `continue` lowers to `Jump { target: cond_bb }`; `break` lowers to `Jump { target: exit_bb }`.

Verification hints
- Use-before-def: delay copies that would reference later-defined values or route via Phi at block head.
- Pred consistency: for every `Jump`/`Branch`, record the predecessor on the successor block.
- PHI-off invariant: all merged values reach the merge via predecessor copies; the merge block contains no extra Copy to the same dst.

Snapshot rules (Loop/If)
- Loop: take the latch snapshot at the actual latch block (end of body, after nested if merges). Use it as the backedge source when sealing header.
- If: capture `pre_if_snapshot` before entering then/else; restore at merge and only bind merged variables (diff-based). Avoid self-copy at merge.
