# Phase 9.78h: MIRパイプライン前提整備（P2P/Cranelift前の全作業）

Status: In Progress（9.79 P2PBox前のゲート）
Last Updated: 2025-08-25

## 🎯 目的
P2PBox（Phase 9.79）に着手する前に、MIRパイプライン（Builder/SSA/MIR26/Verifier/Optimizer/VM整合）を完全に安定化し、26命令セットで凍結する。これにより、P2P/Craneliftの土台を強固にする。

## 📦 スコープ（MIRまわりの全タスク）
0) 命令セットの凍結（26命令が正）
- 命令セットの単一出典: `docs/reference/mir/INSTRUCTION_SET.md` を唯一の参照に統一
- コード側の列挙とテスト: `src/mir/instruction.rs` の列挙と一致、総数26のテストで保証（ドキュメント≧コードではなくコード≡ドキュメント）
- 25命令文献はアーカイブへ移動（本流は26命令）
1) Builder/Loweringの確定
- Builder移行完了: `builder.rs` → `builder_modularized/*`（命令フィールド名・効果一致: `function→func`, `arguments→args`）
- Loop SSA復帰: `loop_api` によるPhi挿入・seal・predecessor更新の段階適用、簡易lowering除去
- TypeOp早期lowering網羅: `is/as/isType/asType` の関数/メソッド両パスで確実に `TypeOp(Check/Cast)` 生成、`print(isType(...))` 直下もdst化

2) MIR26命令ダイエットの凍結
- TypeOp統合: Check/Castの意味論確定、Printer表示/エフェクト統一
- WeakRef/Barrier統合: flag ON/OFFで差分固定（PoC featureで比較可能に）
- 命令リストの合意化: 26命令でのPrinter/Verifier/Optimizer整合

3) Verifier/Printer/Optimizer整合
- Verifier: mergeでのphi未使用検知/支配関係、Barrier位置/WeakRef整合のチェック
- Printer: `--mir-verbose` で TypeOp/WeakRef/Barrier を明示、`--mir-verbose-effects` で `pure|readonly|side`
- Optimizer: 未lowering安全ネット（Call/BoxCall→TypeOp）強化、`NYASH_OPT_DIAG_FAIL=1` で診断をCIゲート化

4) VM整合（ホットパス健全化）
- BinOp: `and`/`or` 実装、BoxRef×BoxRefの数値演算サポート
- Compare/Branch: 既定のVoid/Bool/Intセマンティクスを維持、回帰テスト
- Array/Map/BoxCall: get/set/push/size のfast-path・identity shareの確認
- VM Stats: `--vm-stats`, `--vm-stats-json` の代表ケース更新

5) スナップショット/CI導線
- 軽量スナップショット: TypeOp/extern_call/loop/await/boxcall の含有チェックを代表ケースで固定
- ゴールデン比較: `tools/snapshot_mir.sh` + `tools/ci_check_golden.sh` の運用整備
- CLI分離テスト: `cargo test -p core` のみで回る構成（CLI変更で止まらない）

6) ランタイム/API整備
- ResultBox移行: `box_trait::ResultBox` → `boxes::ResultBox` へ全面置換、互換層の段階削除
- ドキュメント同期: CURRENT_TASK/CLAUDE/phase-docを更新し参照経路を一本化

## ✅ 受け入れ基準（P2P着手ゲート）
- [ ] MIR26整合完了（Printer/Verifier/Optimizer一致・効果表記統一）
- [ ] Loop SSA復帰（Phi/Seal/Pred更新がVerifierで合格）
- [ ] TypeOp網羅（is/as/isType/asTypeの早期lowering＋Optimizer診断ONで回帰ゼロ）
- [ ] 軽量スナップショット緑（TypeOp/extern_call/loop/await/boxcall）
- [ ] VM未実装の解消（And/Or・BoxRef演算）
- [ ] CLI分離テスト導線（`cargo test -p core`）安定
- [x] ResultBox移行完了（旧参照なし）
- [ ] 命令セットの単一出典化（INSTRUCTION_SET.md）と総数26のテストがCIで緑

## 🪜 タスク分解（実行順）
1. Builder移行完了（命令フィールド名・効果一致）
2. Loop SSA復帰（Phi/Seal/Pred更新の段階適用）
3. TypeOp早期loweringの網羅 + Optimizer安全ネットの強化
4. MIR26統合（TypeOp/WeakRef/Barrier）とPrinter/Verifier/Optimizer整合
5. VM補強（and/or, BoxRef演算, Array/Map fast-path確認）
6. 軽量スナップショット + CLI分離テスト + ResultBox移行の仕上げ

## 🔗 依存/参照
- 次フェーズ（P2P本体）: [phase_9_79_p2pbox_rebuild.md](phase_9_79_p2pbox_rebuild.md)
- CURRENT_TASKの「近々/中期」および「Phase 10 着手ゲート」
- `docs/reference/execution-backend/p2p_spec.md`（P2Pは9.79で実装）

## 🚫 非スコープ
- P2PBox/IntentBox/MessageBus実装（9.79で扱う）
- Cranelift JIT/LLVM AOT（Phase 10以降）

---
メモ: 9.78hは「MIRの足場を固める」段階。ここで26命令・SSA・検証・スナップショット・VM整合・ResultBox移行までを完了し、9.79(P2P)→Phase 10(Cranelift)へ安全に進む。
