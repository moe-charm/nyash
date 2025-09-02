# Phase 12 リファクタリング計画（<= 1000行/ファイル方針）

目的: 可読性とAI/人間のレビュー効率を上げるため、大型ファイルを責務単位で分割。

## 対象候補（行数ベース）
- src/mir/builder.rs ~2100行
- src/jit/lower/builder.rs ~2050行
- src/backend/vm.rs ~1510行
- src/runtime/plugin_loader_v2.rs ~1430行
- src/jit/lower/core.rs ~1430行
- src/backend/vm_instructions.rs ~1360行
- src/backend/llvm/compiler.rs ~1230行
- src/interpreter/plugin_loader.rs ~1210行
- src/runner.rs ~1170行
- src/ast.rs ~1010行

## 分割提案（第1期: 安全にファイル分割のみ）
1) src/runner.rs
   - runner/mod.rs（エントリ）
   - runner/modes/{vm.rs,jit.rs,llvm.rs,mir_interpreter.rs}
   - runner/utils.rs（共通ヘルパ）

2) src/runtime/plugin_loader_v2.rs
   - runtime/plugin_loader/{mod.rs,registry.rs,encoder.rs,decoder.rs,invoke.rs}
   - 既存のTLV共通は `plugin_ffi_common.rs` へ残置

3) src/mir/builder.rs
   - mir/builder/{mod.rs,blocks.rs,phis.rs,lower_ops.rs}

4) src/jit/lower/{core.rs,builder.rs}
   - jit/lower/core/{mod.rs,graph.rs,stats.rs}
   - jit/lower/builder/{mod.rs,clif.rs,object.rs,stats.rs}

5) src/backend/vm*.rs
   - backend/vm/{mod.rs,values.rs,dispatch.rs,gc.rs}
   - backend/vm_instructions.rs → backend/vm/ops_*.rs (load/store/arith/call等で分割)

6) src/backend/llvm/compiler.rs
   - backend/llvm/{mod.rs,emit_object.rs,link.rs,passes.rs}

7) src/ast.rs
   - ast/{mod.rs,node.rs,visitor.rs,printer.rs}

## 実行順（トライ&スライス）
- 1) runner → 2) plugin_loader → 3) mir/builder
- 各ステップで `cargo build` ＋ 既存smoke確認

## 非目標（現段階で触らない）
- 機能変更や最適化（挙動は完全に不変）
- 命名や公開APIの変更

## 完了条件
- 1000行超のファイルを概ね収束（±50行は許容）
- CIスモーク（apps/tests）成功
- レビュー観点のチェックリスト合格
