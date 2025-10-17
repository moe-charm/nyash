# substring 不安定降下問題 - 調査インデックス

**問題**: `s.substring(j, j+1)` が BoxCall と Extern の間で不安定に降下
**影響**: 実行時エラー、予測不可能な動作
**調査日**: 2025-10-17

---

## 📚 調査ドキュメント一覧

### 1. **Task 1: メソッド降下経路選択メカニズム調査** ✅ 完了

**担当**: Task Agent 1
**ドキュメント**:
- **完全調査レポート**: [method-routing-mechanism.md](./method-routing-mechanism.md)
- **視覚的フローチャート**: [method-routing-flowchart.md](./method-routing-flowchart.md)
- **サマリー**: [task1-summary.md](./task1-summary.md)

**主要発見**:
- substring は3つの異なる経路を通る (Early Table / BoxCall / Unified)
- origin 情報の有無で降下先が決まる
- method_id が存在しないため BoxCall 経路を通らない
- 起源不明時は UnknownBox → BoxCall → 実行時エラー

**キーファイル**:
- `src/mir/builder/method_call_handlers.rs:106-229` - Entry point
- `src/mir/builder/lowering/mod.rs:26` - Early Extern テーブル
- `src/mir/builder/infer/receiver.rs:9-61` - Receiver 推論
- `src/mir/builder/router/policy.rs:16-80` - Route 決定

---

### 2. **Task 2: Origin 伝播メカニズム調査** 🚧 進行中

**担当**: Task Agent 2
**調査対象**:
- `annotate_call_result_from_func_name()` の実装
- substring/indexOf の戻り値に origin が伝播しない理由
- Origin Propagation 実装の影響範囲

**期待される成果物**:
- Origin 伝播の現状分析
- 修正箇所の特定
- 影響範囲の評価

---

### 3. **Task 3: value_types 設定メカニズム調査** 🚧 進行中

**担当**: Task Agent 3
**調査対象**:
- `value_types` の設定タイミング
- substring/indexOf の戻り値型推論
- inferred_string パスの作動条件

**期待される成果物**:
- value_types 伝播フロー
- 修正箇所の特定
- Option 2 の実現可能性評価

---

### 4. **Task 4: 既存の修正事例・パターン調査** 🚧 進行中

**担当**: Task Agent 4
**調査対象**:
- 過去の origin/value_types 修正事例
- 類似問題の解決パターン
- 既存の workaround

**期待される成果物**:
- 修正事例リスト
- ベストプラクティス抽出
- 推奨修正パターン

---

## 🎯 問題の核心 (Task 1 発見)

### 不安定降下の根本原因

```
引数/戻り値の値 (origin 不明)
  ↓
infer_receiver() → "UnknownBox"
  ↓
choose_route("UnknownBox", ...) → Route::BoxCall
  ↓
emit_box_or_plugin_call() → BoxCall instruction
  ↓
VM で method_id 解決失敗
  ↓
実行時エラー ❌
```

### 安定化の条件

```
NewBox or 明示的設定 (origin あり)
  ↓
try_lower_via_table("StringBox", "substring", 2)
  ↓
lowering::lower_builtin_method マッチ
  ↓
Extern("nyrt.string.substring") ✅ 安定
```

---

## 🔧 解決策の選択肢 (Task 1 提示)

### Option 1: Origin Propagation ⭐推奨

**実装**: substring/indexOf の戻り値に `origin="StringBox"` を設定

**メリット**:
- 根本解決 (全メソッドで一貫)
- Early Table が自動作動
- 追加ロジック不要

**デメリット**:
- 大規模変更
- 他メソッドへの影響

**調査タスク**: Task 2 で詳細調査

---

### Option 2: value_types Propagation

**実装**: substring/indexOf の戻り値に `MirType::String` を設定

**メリット**:
- 比較的小規模
- inferred_string パス活用

**デメリット**:
- value_types 管理が複雑化
- origin ほど明確でない

**調査タスク**: Task 3 で詳細調査

---

### Option 3: Normalize 拡張

**実装**: `normalize/string_methods.rs` を追加、substring/indexOf を Extern 化

**メリット**:
- 局所的変更
- 既存コードに影響なし

**デメリット**:
- Phase 1 逃したケースのみ対応
- 根本解決でない

---

### Option 4: Fallback Heuristic

**実装**: `infer_receiver()` の heuristic 拡張、substring/indexOf → "StringBox"

**メリット**:
- 1箇所の変更
- 即効性

**デメリット**:
- 応急処置
- 他メソッドに効かない

---

## 📊 調査進捗

| Task | 担当 | ステータス | 完了度 | 成果物 |
|------|------|-----------|--------|--------|
| Task 1 | Agent 1 | ✅ 完了 | 100% | 3 docs |
| Task 2 | Agent 2 | 🚧 進行中 | 0% | - |
| Task 3 | Agent 3 | 🚧 進行中 | 0% | - |
| Task 4 | Agent 4 | 🚧 進行中 | 0% | - |

---

## 🎓 重要な学び (Task 1)

1. **3層の降下経路**: Early Table → RouterPolicy → Normalize
2. **origin の決定的重要性**: origin があれば Early Table で即座に安定化
3. **substring の method_id 欠如**: BoxCall 経路を通らない → Unified に依存
4. **推論の脆弱性**: UnknownBox → BoxCall → 実行時エラー
5. **normalize の限界**: substring は対象外 → Method のまま VM へ

---

## 📁 関連ファイル (Task 1 調査)

### Entry Points
- `src/mir/builder/method_call_handlers.rs:106-229` - handle_standard_method_call
- `src/mir/builder/builder_calls/emit.rs:10-402` - emit_unified_call

### Lowering & Routing
- `src/mir/builder/lowering/mod.rs:17-47` - lower_builtin_method
- `src/mir/builder/router/policy.rs:16-80` - choose_route
- `src/mir/builder/rewrite/gate.rs:8-26` - should_rewrite

### Inference & Normalization
- `src/mir/builder/infer/receiver.rs:9-61` - infer_receiver
- `src/mir/builder/normalize/mod.rs:65-78` - apply_all

### Type Annotation (Task 2/3 調査対象)
- `src/mir/builder/types/annotation.rs` - annotate_call_result_from_func_name

---

## 🚀 次のステップ

1. **Task 2-4 の調査完了を待つ**
2. **4つの調査結果を統合**
3. **最適な修正方針を決定**
4. **実装計画を策定**

---

**インデックス作成日**: 2025-10-17
**最終更新**: 2025-10-17
**総合ステータス**: 🚧 進行中 (25% 完了)
