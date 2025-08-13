# Phase 6: Box ops minimal in MIR/VM (RefNew/RefGet/RefSet, WeakNew/WeakLoad)

## Summary
- MIR/VM に Box 参照操作の最小セットを導入。Barrier はダミーで開始。

## Scope
- MIR 命令追加: `RefNew`, `RefGet`, `RefSet`, `WeakNew`, `WeakLoad`, `BarrierRead`/`BarrierWrite`(no-op)
- Lowering: `New`/`FieldAccess`/`MethodCall` の最小対応（`BoxCall` は後続でも可）
- VM: 上記命令の最小実行（参照テーブル/マップでOK）

## Tasks
- [ ] `src/mir/instruction.rs`: 命令追加 + `printer`/`verification` 対応
- [ ] `src/mir/builder.rs`: lowering（最小ケース）
- [ ] `src/backend/vm.rs`: 命令実装
- [ ] サンプル/スナップショットの追加

## Acceptance Criteria
- 新規サンプルで `--dump-mir`/`--backend vm` が成功（Ref/Weak の基本動作）
- weak の自動 null と `fini()` 後使用禁止の不変を壊さない

## References
- `docs/nyash_core_concepts.md`（weak/fini の不変条件）
- Phase 5/5.1/5.2 Issues（control flow/exception/static Main lowering）

## Copilot Notes
- まず `RefNew`/`RefGet`/`RefSet` → `WeakNew`/`WeakLoad` の順で実装。Barrier は no-op でOK。
- サンプルはトップレベル/関数内で回せる形から。`BoxDeclaration` 依存は避けても良い。
