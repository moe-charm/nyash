# Task 1: メソッド降下経路選択メカニズム調査 - 完了報告

**調査期間**: 2025-10-17
**調査者**: Task Agent 1
**ステータス**: ✅ 完了

---

## 📋 調査目的

なぜ `s.substring(j, j+1)` が BoxCall と Extern の間で不安定に降下するのかを特定する

---

## 🎯 核心発見

### 根本原因

**substring は3つの異なる経路を通る可能性があり、receiver の起源情報の有無で降下先が決まる**

1. **Early Extern 経路** (最速・安定): `origin_get(recv) == Some("StringBox")` → 即座に `Extern("nyrt.string.substring")`
2. **BoxCall 経路** (中間): 起源が StringBox だが `resolve_builtin_method_id()` が **None** → Unified へフォールバック
3. **Unified 経路** (不安定): 起源不明 → `infer_receiver()` → "UnknownBox" → `Route::BoxCall` → **実行時エラー**

**重要な事実**: substring は `method_id` が**存在しない**ため、BoxCall 経路を通らず、常に Extern か Unified に依存する。

---

## 📊 決定木要約

```
Method Call: s.substring(j, j+1)
│
├─ origin_get(s) == Some("StringBox")
│  └─ try_lower_via_table("StringBox", "substring", 2)
│     └─ lowering::lower_builtin_method マッチ
│        └─ ✅ Extern("nyrt.string.substring") [最速・安定]
│
├─ value_types[s] == MirType::String or MirType::Box("StringBox")
│  └─ try_lower_via_table("StringBox", "substring", 2)
│     └─ ✅ Extern("nyrt.string.substring") [安定]
│
└─ 上記すべて失敗 (起源不明)
   └─ emit_unified_call
      ├─ infer_receiver() → "UnknownBox"
      │  └─ choose_route("UnknownBox", ...) → Route::BoxCall
      │     └─ ❌ BoxCall → 実行時エラー (method_id なし)
      │
      └─ infer_receiver() → "StringBox" (heuristic)
         └─ choose_route("StringBox", ...) → Route::BoxCall (is_core_box)
            └─ ❌ BoxCall → 実行時エラー (method_id なし)
```

---

## 🔍 コード位置詳細

### Entry Points

| ファイル | 行番号 | 関数 | 役割 |
|---------|--------|------|------|
| `src/mir/builder/method_call_handlers.rs` | 106-229 | `handle_standard_method_call` | メソッド呼び出しの最初の振り分け |
| `src/mir/builder/builder_calls/emit.rs` | 10-402 | `emit_unified_call` | 統一呼び出し経路 |

### Key Decision Points

| ファイル | 行番号 | 関数 | 決定内容 |
|---------|--------|------|----------|
| `src/mir/builder/lowering/mod.rs` | 17-47 | `lower_builtin_method` | Early Extern テーブル |
| `src/mir/builder/lowering/mod.rs` | 26 | (table entry) | `("substring", 2) → "nyrt.string.substring"` |
| `src/mir/builder/method_call_handlers.rs` | 127-163 | `try_lower_via_table` | Early Table 適用 |
| `src/mir/builder/method_call_handlers.rs` | 169 | (inline) | origin_cls マッチ → Early return |
| `src/mir/builder/method_call_handlers.rs` | 172-185 | (inline) | method_id → BoxCall (substring は None) |
| `src/mir/builder/method_call_handlers.rs` | 188-191 | (inline) | inferred_string → Table 適用 |
| `src/mir/builder/infer/receiver.rs` | 9-61 | `infer_receiver` | Receiver クラス推論 |
| `src/mir/builder/router/policy.rs` | 16-80 | `choose_route` | Route::Unified vs Route::BoxCall |
| `src/mir/builder/rewrite/gate.rs` | 8-26 | `should_rewrite` | Known rewrite 除外ルール |
| `src/mir/builder/normalize/mod.rs` | 65-78 | `apply_all` | 正規化 (substring は対象外) |

---

## 📝 substring の特殊扱い箇所

### 1. Lowering Table (Early Extern)

**ファイル**: `src/mir/builder/lowering/mod.rs:26`

```rust
"StringBox" => match (method, arity) {
    ("substring", 2) => Some(LoweredExternSpec {
        extern_name: "nyrt.string.substring",
        prepend_recv: true
    }),
    // ...
}
```

**動作**: `recv_cls == Some("StringBox")` && `arity == 2` → 即座に Extern 化

### 2. Rewrite Gate (Known rewrite 除外)

**ファイル**: `src/mir/builder/rewrite/gate.rs:15`

```rust
if cls == "StringBox" && matches!(method,
    "length" | "len" | "substring" | "indexOf" | "lastIndexOf"
) {
    return false; // rewrite しない
}
```

**理由**: StringBox の substring は Extern 経路専用。ModuleFunction 化を防ぐ。

### 3. Normalize (対象外)

**ファイル**: `src/mir/builder/normalize/mod.rs:65-78`

```rust
pub fn apply_all(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) {
    let _ = string_length::normalize_string_length_call(...); // length のみ
    // substring は normalize 対象外
}
```

**結果**: substring は Unified 経路で Callee::Method のまま VM へ → 不安定

---

## 🚨 不安定降下の条件

### ❌ BoxCall → 実行時エラーになる条件

1. **引数由来の値**: `parse(text: StringBox)` の `text` → origin 未設定
2. **メソッド戻り値**: `s = path.substring(0, idx)` の `s` → origin 伝播なし
3. **複雑な式**: 型推論が失敗 → value_types 未設定
4. **推論結果が UnknownBox**: `infer_receiver()` が "UnknownBox" を返す
5. **RouterPolicy が BoxCall を選択**: `choose_route("UnknownBox", ...)` → `Route::BoxCall`

### ✅ Extern → 安定する条件

1. **origin 設定あり**: `origin_get(s) == Some("StringBox")`
2. **value_types 設定あり**: `value_types[s] == MirType::String` または `MirType::Box("StringBox")`
3. **Early Table が作動**: Phase 1 で `try_lower_via_table()` がマッチ

---

## 🔧 問題解決の方向性 (他タスクで検討)

### Option 1: Origin Propagation (最も根本的) ⭐推奨

**内容**: substring/indexOf の戻り値に `origin="StringBox"` を設定

**実装箇所**: `src/mir/builder/types/annotation.rs`

**利点**:
- 全メソッドで一貫性
- 追加ロジック不要 (Early Table が自動的に作動)
- 根本解決

**欠点**:
- 大規模変更
- 他のメソッドにも影響

### Option 2: value_types Propagation (中間)

**内容**: substring/indexOf の戻り値に `MirType::String` を設定

**実装箇所**: `src/mir/builder/types/annotation.rs` or `emit_unified_call`

**利点**:
- 比較的小規模
- inferred_string パスで Early Table 作動

**欠点**:
- value_types 管理が複雑化
- origin ほど明確でない

### Option 3: Normalize 拡張 (最も局所的)

**内容**: `normalize/string_methods.rs` を追加、substring/indexOf を Extern 化

**実装箇所**: `src/mir/builder/normalize/` (新規モジュール)

**利点**:
- 局所的変更
- 既存コードに影響なし

**欠点**:
- Phase 1 の Early Routing を逃したケースのみ対応
- 根本解決でない

### Option 4: Fallback Heuristic (最小変更)

**内容**: `infer_receiver()` の heuristic を拡張、substring/indexOf → "StringBox"

**実装箇所**: `src/mir/builder/infer/receiver.rs:50-59`

**利点**:
- 1箇所の変更
- 即効性あり

**欠点**:
- 応急処置
- 他のメソッドには効かない

---

## 📚 生成ドキュメント

1. **完全調査レポート**: `/docs/development/analysis/method-routing-mechanism.md`
   - 決定木詳細
   - コード位置一覧
   - 解決策の比較

2. **フローチャート**: `/docs/development/analysis/method-routing-flowchart.md`
   - Mermaid 図
   - 視覚的経路
   - 3経路比較表

3. **本サマリー**: `/docs/development/analysis/task1-summary.md`

---

## 🎓 学び・Insights

1. **3層の降下経路**: Early Table (最速) → RouterPolicy (中間) → Normalize (最終)
2. **origin の決定的重要性**: origin があれば即座に Early Table で安定化
3. **substring の method_id 欠如**: BoxCall 経路を通らない → Unified に完全依存
4. **推論の脆弱性**: UnknownBox → BoxCall → 実行時エラー
5. **normalize の限界**: substring は対象外 → Method のまま VM へ

---

## ✅ 調査完了チェックリスト

- [x] メソッド降下の決定木を特定
- [x] BoxCall vs Extern の分岐条件を特定
- [x] substring 専用の処理を全て特定
- [x] コード位置 (ファイル名:行番号) を記録
- [x] フローチャート作成
- [x] 完全ドキュメント作成
- [x] 問題解決の方向性を提示

---

## 🚀 次のステップ

Task 2-4 の調査結果と統合し、最適な修正方針を決定する。

**推奨**: Option 1 (Origin Propagation) を Task 2 で詳細調査

---

**調査完了日**: 2025-10-17
**総調査時間**: ~30分
**生成ドキュメント数**: 3
**調査対象ファイル数**: 8
