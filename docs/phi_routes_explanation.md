# PHI生成の2つの独立経路

## 📋 概要

Nyashコンパイラには**2つの完全に独立したPHI生成コード**があります：

## 🔵 経路1: 通常実行（MirBuilder）

### 使用条件
- 通常の`.nyash`ファイル実行
- `--backend vm` (デフォルト)
- `--dump-mir`

### 処理フロー
```
.nyash ファイル
  ↓
NyashParser (src/parser/)
  ↓
AST生成
  ↓
MirCompiler (src/mir/mod.rs)
  ↓
MirBuilder (src/mir/builder/)
  ↓
if_form.rs (line 148: normalize_if_else_phi)
  ↓
phi_merge_helper.rs (PHI命令発行)
```

### PHI生成コード
- **ファイル**: `src/mir/builder/if_form.rs`
- **関数**: `normalize_if_else_phi()`
- **ヘルパー**: `src/mir/builder/phi_merge_helper.rs`

---

## 🟢 経路2: JSON v0 Bridge（旧パーサー互換）

### 使用条件
- `--parser ny` フラグ使用時
- JSON形式のMIR入力（selfhost用）
- pipe_io経由

### 処理フロー
```
.nyash ファイル（--parser ny指定）
  ↓
json_v0_bridge::parse_source_v0_to_module() (src/runner/json_v0_bridge/)
  ↓
AST v0 パース
  ↓
lowering/expr.rs (line 397-398)
  ↓
lowering/ternary.rs (lower_ternary_expr_with_scope)
  ↓
PHI命令直接挿入 (line 51)
```

### PHI生成コード
- **ファイル**: `src/runner/json_v0_bridge/lowering/ternary.rs`
- **関数**: `lower_ternary_expr_with_scope()`
- **注意**: コメントに「not wired yet」とあるが、実際には使われている

---

## ⚠️ 重要な違い

| 項目 | MirBuilder経路 | JSON v0 Bridge経路 |
|-----|--------------|------------------|
| **使用頻度** | デフォルト（ほぼ全て） | `--parser ny`時のみ |
| **PHI生成方式** | PhiMergeHelper箱化 | 直接insert_instruction_after_phis |
| **コード場所** | src/mir/builder/ | src/runner/json_v0_bridge/ |
| **現在の問題** | if_form.rs空ブロック生成 | 問題なし（コメントは古い） |

---

## 🔍 今回のバグはどちら？

**MirBuilder経路（経路1）**のバグです！

- 通常の実行（`./target/release/hako test.nyash`）
- NyashParser → MirBuilder経路
- if_form.rsの問題で空ブロック生成
- PHI命令が欠落

JSON v0 Bridge経路は正しく動作している可能性が高い。

---

## 📝 備考

- json_v0_bridgeは「Phase 15セルフホスティング」用の特殊経路
- ternary.rsのコメント「not wired yet」は古い（実際には配線済み）
- 2つの経路を統一する計画はあるか？（要確認）

---

**作成日**: 2025-10-03
**作成者**: Claude Code
