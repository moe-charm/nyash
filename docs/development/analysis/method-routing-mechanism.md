# メソッド降下経路選択メカニズム完全調査

**調査対象**: `s.substring(j, j+1)` が BoxCall と Extern の間で不安定に降下する理由
**調査日**: 2025-10-17
**調査者**: Task Agent 1

## 🎯 Executive Summary

`substring` のような String メソッドは、**receiver の起源情報の有無**により3つの異なる経路を通る：

1. **Early Extern 経路** (最速): 起源が判明している場合 → `try_lower_via_table` → 即座に `Extern("nyrt.string.substring")` へ
2. **BoxCall 経路** (中間): 起源が StringBox だが method_id が見つかる場合 → `BoxCall` (VM で再解決)
3. **Unified 経路** (フォールバック): 起源不明 → `emit_unified_call` → RouterPolicy → 推論 → 最終的に Extern or BoxCall

**不安定の根本原因**: substring は method_id が**存在しない**ため、起源が StringBox でも BoxCall 経路を通らず、Unified 経路にフォールバック → 推論の結果次第で Extern/BoxCall が揺れる

---

## 📊 メソッド降下の決定木 (Decision Tree)

```
Method Call: receiver.method(args)
│
├─ [Phase 1: handle_standard_method_call - EARLY ROUTING]
│  │
│  ├─ Q: method == "birth"?
│  │  └─ YES → emit_legacy_call (ModuleFunction) ✅ 終了
│  │
│  ├─ Q: origin_cls 存在？
│  │  ├─ YES (Known Origin)
│  │  │  ├─ try_lower_via_table(origin_cls, method, arity)
│  │  │  │  ├─ lowering::lower_builtin_method() がマッチ？
│  │  │  │  │  ├─ YES → Extern("nyrt.string.substring") ✅ 終了
│  │  │  │  │  └─ NO → 次へ
│  │  │  │
│  │  │  └─ Q: cls in {ArrayBox, MapBox, StringBox}?
│  │  │     └─ YES → resolve_builtin_method_id(cls, method)?
│  │  │        ├─ Some(method_id) → BoxCall ✅ 終了
│  │  │        └─ None → 次へ (Unified経路へフォールバック)
│  │  │
│  │  └─ NO (Unknown Origin)
│  │     ├─ Q: value_types[receiver] == MirType::String?
│  │     │  └─ YES → try_lower_via_table("StringBox", method, arity)
│  │     │     └─ マッチ → Extern("nyrt.string.substring") ✅ 終了
│  │     │
│  │     └─ Q: method in {length, len} && arity==0?
│  │        └─ YES → try_lower_via_table("StringBox", method, arity)
│  │           └─ マッチ → Extern("nyrt.string.length") ✅ 終了
│  │
│  └─ [上記すべて失敗] → emit_unified_call へフォールバック
│
├─ [Phase 2: emit_unified_call - UNIFIED ROUTING]
│  │
│  ├─ [2.1: Receiver Inference]
│  │  └─ infer_receiver(box_type, method, receiver, origin_lookup, value_types)
│  │     → (class_name: String, certainty: TypeCertainty)
│  │
│  ├─ [2.2: Early Rewrite Attempts]
│  │  ├─ try_early_str_like_to_dst() - 特殊 String-like 処理
│  │  ├─ try_special_equals_to_dst() - equals/1 特殊化
│  │  └─ try_known_or_unique_to_dst() - Known/Unique rewrite
│  │     └─ should_rewrite(builder, cls, method, arity)?
│  │        └─ if cls=="StringBox" && method in {substring, indexOf, ...}
│  │           → return false ❌ (rewrite しない)
│  │
│  ├─ [2.3: Callee Conversion]
│  │  └─ convert_target_to_callee(target, origin_lookup, value_types)
│  │     → Callee::Method { box_name, method, receiver, certainty }
│  │
│  ├─ [2.4: RouterPolicy Guard]
│  │  └─ choose_route(box_name, method, certainty, arity)
│  │     ├─ if box_name == "UnknownBox" → Route::BoxCall
│  │     ├─ if box_name == "StringBox" && method in {size,len,length} && arity==0
│  │     │  → Route::Unified (normalize で Extern 化)
│  │     ├─ if is_core_box(box_name) → Route::BoxCall (legacy 優先)
│  │     └─ else → Route::Unified
│  │
│  │  └─ if Route::BoxCall
│  │     → emit_box_or_plugin_call() → BoxCall ✅ 終了
│  │
│  ├─ [2.5: Normalization]
│  │  └─ apply_all(builder, callee, args)
│  │     ├─ normalize_string_length_call() - length/size/len → Extern
│  │     ├─ normalize_array_length_call()
│  │     ├─ normalize_map_length_call()
│  │     └─ normalize_set_call()
│  │     └─ **substring は normalize 対象外** ❌
│  │
│  └─ [2.6: Final Emission]
│     └─ emit_instruction(MirInstruction::Call { callee: Some(callee), ... })
│        ├─ Callee::Extern(name) → Extern call ✅
│        ├─ Callee::Method { ... } → Method call (VM で解決) ✅
│        └─ Callee::ModuleFunction(name) → ModuleFunction call ✅
│
└─ 終了
```

---

## 🔍 substring 専用の処理箇所

### 1. `lowering/mod.rs:26` - Early Extern テーブル

```rust
"StringBox" => match (method, arity) {
    ("substring", 2) => Some(LoweredExternSpec {
        extern_name: "nyrt.string.substring",
        prepend_recv: true
    }),
    // ...
}
```

**条件**: `recv_cls == Some("StringBox")` && `method == "substring"` && `arity == 2`
**結果**: 即座に `Extern("nyrt.string.substring", [recv, arg0, arg1])` を emit

### 2. `rewrite/gate.rs:15` - Rewrite 除外

```rust
if cls == "StringBox" && matches!(method, "length" | "len" | "substring" | "indexOf" | "lastIndexOf") {
    return false; // rewrite しない
}
```

**理由**: StringBox の substring は Extern 経路専用。Known rewrite (ModuleFunction 化) を防ぐ。

### 3. `method_call_handlers.rs:169` - Early Table 適用

```rust
if let Some(cls) = origin_cls.as_deref() {
    if let Some(dst) = try_lower_via_table(self, Some(cls), &method, object_value, &mut arg_values) {
        return Ok(dst); // ✅ 早期 return
    }
}
```

**条件**: `origin_get(receiver)` が StringBox を返す
**結果**: Phase 1 で即座に Extern 化。Unified 経路を通らない。

---

## 🚨 不安定降下の根本原因

### 問題: `s.substring(j, j+1)` が BoxCall になるケース

**シナリオ**:
1. `s` の起源が**不明** (`origin_get(s) == None`)
2. `value_types[s]` も設定されていない or `MirType::Box("StringBox")` でない
3. Phase 1 の Early Routing をすべてスキップ
4. `emit_unified_call` に到達
5. `infer_receiver()` が "UnknownBox" を返す
6. `choose_route("UnknownBox", "substring", ...)` → `Route::BoxCall`
7. BoxCall を emit → **VM で実行時エラー** (substring の method_id が存在しない)

### 起源が不明になる原因

#### A. パラメータ由来の値
```rust
box Parser {
    parse(text: StringBox) {  // text は引数 → origin 設定されない
        local s
        s = text
        local char
        char = s.substring(i, i+1)  // ❌ s の origin 不明 → BoxCall
    }
}
```

**原因**: パラメータ `text` は `NewBox` で生成されていないため、`origin_register()` が呼ばれない。

#### B. 複雑な式の結果
```rust
local s
s = path.substring(0, lastSlash)  // path.substring の結果
local ext
ext = s.substring(dotIdx + 1, s.length())  // ❌ s の origin 不明
```

**原因**: `path.substring()` の戻り値に origin が伝播していない。

#### C. メソッド呼び出し結果
```rust
local text
text = file.read()  // file.read() の戻り値
local first
first = text.substring(0, 10)  // ❌ text の origin 不明
```

---

## ✅ BoxCall になる条件 vs Extern になる条件

| 条件 | 経路 | 結果 | 備考 |
|------|------|------|------|
| `origin_get(recv) == Some("StringBox")` | Early Table | `Extern("nyrt.string.substring")` | ✅ 最速・安定 |
| `value_types[recv] == MirType::String` | Early Table (inferred) | `Extern("nyrt.string.substring")` | ✅ 安定 |
| `value_types[recv] == MirType::Box("StringBox")` | Early Table (inferred) | `Extern("nyrt.string.substring")` | ✅ 安定 |
| 上記すべて失敗 + `infer_receiver() == "StringBox"` | Unified → Normalize | `Callee::Method` (normalize 対象外) | ⚠️ Method のまま (VM 依存) |
| 上記すべて失敗 + `infer_receiver() == "UnknownBox"` | Unified → RouterPolicy | `BoxCall` | ❌ **実行時エラー** |

---

## 📝 コード位置まとめ

### Entry Points

| ファイル | 行番号 | 関数 | 役割 |
|---------|--------|------|------|
| `method_call_handlers.rs` | 106-229 | `handle_standard_method_call` | メソッド呼び出しの最初の振り分け |
| `builder_calls/emit.rs` | 10-402 | `emit_unified_call` | 統一呼び出し経路 (Unified Routing) |

### Key Decision Points

| ファイル | 行番号 | 関数 | 決定内容 |
|---------|--------|------|----------|
| `lowering/mod.rs` | 17-47 | `lower_builtin_method` | Early Extern テーブル (substring/2 → nyrt.string.substring) |
| `method_call_handlers.rs` | 127-163 | `try_lower_via_table` | Early Extern 適用 (Phase 1) |
| `method_call_handlers.rs` | 169 | (inline) | origin_cls マッチ → Early return |
| `method_call_handlers.rs` | 172-185 | (inline) | resolve_builtin_method_id → BoxCall |
| `method_call_handlers.rs` | 188-191 | (inline) | inferred_string → Early Table 適用 |
| `method_call_handlers.rs` | 194-202 | (inline) | length/len fallback → Early Table |
| `infer/receiver.rs` | 9-61 | `infer_receiver` | Receiver クラス推論 (Unknown/String/Box) |
| `router/policy.rs` | 16-80 | `choose_route` | Route::Unified vs Route::BoxCall |
| `rewrite/gate.rs` | 8-26 | `should_rewrite` | Known rewrite 除外ルール |
| `normalize/mod.rs` | 65-78 | `apply_all` | 正規化適用 (substring は対象外) |

### Extern Lowering Table

| ファイル | 行番号 | テーブル | エントリ |
|---------|--------|----------|----------|
| `lowering/mod.rs` | 24-33 | StringBox methods | substring/2, indexOf/1-2, length/0, charAt/1, etc. |
| `lowering/mod.rs` | 39-44 | MapBox methods | size/0, keys/0, values/0 |
| `lowering/mod.rs` | 35-37 | ArrayBox methods | (現在は None、将来用) |

---

## 🔧 問題解決の方針 (他のタスクで検討)

### Option 1: Origin Propagation (最も根本的)
- `annotate_call_result_from_func_name()` を拡張
- substring/indexOf の戻り値に origin="StringBox" を設定
- **利点**: 全メソッドで一貫性、追加ロジック不要
- **欠点**: 大規模変更、他のメソッドにも影響

### Option 2: value_types Propagation (中間)
- substring/indexOf の戻り値に `MirType::String` を設定
- inferred_string パスで Early Table が作動
- **利点**: 比較的小規模、既存の inferred_string パスを活用
- **欠点**: value_types の管理が複雑化

### Option 3: Normalize 拡張 (最も局所的)
- `normalize/string_methods.rs` を追加
- substring/indexOf を Extern 化する normalizer を追加
- **利点**: 局所的変更、既存コードに影響なし
- **欠点**: Phase 1 の Early Routing を逃したケースのみ対応

### Option 4: Fallback Heuristic (最小変更)
- `infer_receiver()` の heuristic を拡張
- substring/indexOf/lastIndexOf → "StringBox" を返す
- **利点**: 1箇所の変更、即効性あり
- **欠点**: 根本解決でない、他のメソッドには効かない

---

## 📚 関連ドキュメント

- **Router Policy**: `src/mir/builder/router/policy.rs`
- **Lowering Table**: `src/mir/builder/lowering/mod.rs`
- **Rewrite Gate**: `src/mir/builder/rewrite/gate.rs`
- **Receiver Inference**: `src/mir/builder/infer/receiver.rs`
- **Normalization**: `src/mir/builder/normalize/mod.rs`

---

## 🎓 学び・Takeaways

1. **3層の降下経路**: Early Table (最速) → RouterPolicy (中間) → Normalize (最終)
2. **origin の重要性**: origin が設定されていれば Early Table で即座に安定化
3. **substring の特殊性**: method_id が存在しない → BoxCall 経路を通らない → Unified に依存
4. **推論の脆弱性**: UnknownBox 推論 → BoxCall → 実行時エラー
5. **解決策のトレードオフ**: 根本修正 (origin propagation) vs 局所修正 (normalize 拡張)

---

**調査完了**: 2025-10-17
**次のステップ**: Task 2-4 の調査結果と統合し、最適な修正方針を決定
