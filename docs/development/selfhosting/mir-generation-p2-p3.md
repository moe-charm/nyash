MIR Generation — Phase 15.7 (P2/P3)

Scope
- P2: binop/unary/loop の最小形を共有箱（MirSchema/BlockBuilder）で提供し、emit 経路を薄アダプタで接続（JSON形状は互換）。
- P3: Call 統一の最小導線（Extern/Global/Method/Constructor）を JSON 生成面に追加。VM/LLVM 実行は既存の registry/adapter に準拠。

Shared Boxes
- apps/selfhost/common/mir/mir_schema_box.hako
  - inst_const/ret/compare/branch/jump/binop
  - inst_mir_call_extern/global/method/constructor（argsは i64 wrap）
- apps/selfhost/common/mir/block_builder_box.hako
  - const_ret, compare_branch, binop, loop_counter
  - extern_call_ret/global_call_ret/method_call_ret/constructor_call_ret（最小形）

Emit Adaptation
- apps/selfhost-compiler/pipeline_v2/emit_mir_flow_map.hako（P1/P2）
- apps/selfhost-compiler/pipeline_v2/emit_mir_flow.hako（P1/P2/P3）
- 返却は `{ functions:[...] }` に制限（既存互換）。将来は module(version/kind) へ昇格可能。

Extern Path (P3 Minimal)
- op_eq: `mir_call callee=Extern("nyrt.ops.op_eq") args=[lhs,rhs] -> ret`
- 代表スモーク（rc‑only）:
  - selfhost_pipeline_v2_op_eq_vm（true）
  - selfhost_pipeline_v2_op_eq_false_vm（false）

Next
- Global/Method/Constructor の代表（rc-only）を追加（最小呼出）。
- Eq/Ne の Builder 正規化を導入（フラグ→安定後に既定ON）。

