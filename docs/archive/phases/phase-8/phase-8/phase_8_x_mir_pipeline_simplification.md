# Phase 8.x: MIR/Builder/Optimizer 簡略化計画（責務分離・効果正規化・可視化）

## 🎯 目的
- AST→MIR→Optimizer→Backend の多層化で増した複雑さを段階的に解消し、堅牢性と見通しを改善する。
- DCEの誤削除や多重防御（重複lowering）を撲滅し、責務を明確化する。

## ✅ 方針（要約）
- **責務分離**: Builder=変換のみ / Optimizer=最適化のみ（変換しない）。
- **効果正規化**: `Pure / ReadOnly / SideEffect` の3分類で最適化判定を一元化。
- **可視化**: 段階別のMIRダンプと効果サマリ、スナップショットテストを整備。

## Phase 1: エフェクト見直し（短期）✅ 完了
- `EffectMask::primary_category()` を唯一の根拠に `is_pure()` を再定義（実装済）。
- DCE/並べ替えは `primary_category` で判定（Pureのみ削除/自由移動、ReadOnlyは限定、SideEffectは不可）。
- 単体テスト: 効果判定（PURE|IO は pure ではない）、DCEが副作用命令（print使用のTypeOp等）を削除しないこと（追加済）。
- 可視化: `--mir-verbose-effects` で per-instruction 効果カテゴリ表示（追加済）。
- CI導線（任意）: `NYASH_OPT_DIAG_FAIL=1` で未lowering検出時にfail（診断ゲート・追加済）。

## Phase 2: 変換の一本化（中期）✅ 完了
- **TypeOp loweringをBuilderに集約**（関数/メソッド/print直下/多重StringBox対応は実装済）。
- OptimizerのTypeOp安全ネット（実変換）を削除。診断のみ存続（`NYASH_OPT_DIAG_FAIL`連携）。
- スナップショット: 代表 `*_is_as_*` のMIR（Builder出力）を固定化（`tools/compare_mir.sh`）。

## Phase 3: デバッグ支援強化（短期〜中期）
- `--dump-ast`（実装済）に加え、`--dump-mir --no-optimize`（Builder出力の生MIR）を追加。
- MIRプリンタの詳細表示: 命令行末に `pure/readonly/side` の効果表示（オプション）。
- ゴールデンMIR: 代表サンプルのMIRダンプを保存し差分検出（CI/ローカル）。

## タスク一覧（実装順）
1) OptimizerのTypeOp安全ネットを機能フラグでデフォルトOFF（`mir_typeop_safety_net`）
2) `MirCompiler` に `--no-optimize` 経由のBuilder直ダンプを実装
3) MIRプリンタに効果簡易表示オプション（`--mir-verbose-effects` 等）
4) 効果判定の単体テスト / DCE安全テストの追加
5) Optimizer診断パス（未lowering検知）追加

## 期待効果
- 変換責務の一本化でバグ源の排除・デバッグ容易化
- エフェクト判定の一貫性でDCE/最適化の安全性向上
- 可視化/スナップショットにより回帰を早期検知

---
最終更新: 2025-08-24（Phase 1完了: is_pure修正/テスト/効果可視化/診断ゲート）
