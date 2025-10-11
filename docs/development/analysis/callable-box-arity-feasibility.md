# CallableBox.arity() 実現可能性調査報告

**調査日**: 2025-10-10
**調査目的**: ChatGPT提案のCallableBox.arity()メソッドが既存のarity実装と整合するか検証

---

## 既存arity実装

### 1. maybe_arity_guard実装

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs:15-30`

```rust
#[inline]
fn maybe_arity_guard(type_name: &str, method: &str, arity: usize) -> Result<(), VMError> {
    if method == "birth" { return Ok(()); }  // birth()は常に許可

    // TypeRegistryからarity情報を取得
    if crate::runtime::type_registry::resolve_typebox_by_name(type_name).is_some() {
        // スロット解決を試みる
        if crate::runtime::type_registry::resolve_slot_by_name(type_name, method, arity).is_none() {
            // 既知のarity一覧を取得して診断
            if let Some(known) = crate::runtime::type_registry::known_arities_for(type_name, method) {
                if !known.is_empty() {
                    return Err(VMError::InvalidInstruction(format!(
                        "No matching method: {}.{}({} args). Available arities: {:?}",
                        type_name, method, arity, known
                    )));
                }
            }
        }
    }
    Ok(())
}
```

**使用箇所**:
- BoxCall実行前の静的arity検証
- StringBox/ArrayBox/MapBoxのメソッド呼び出し時（`mod.rs:39, 137, 199`）
- **呼び出し時**にarity不一致を検出（Fail-Fast設計）

### 2. TypeRegistry のarity管理

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/type_box_abi.rs:47-51`

```rust
pub struct MethodEntry {
    pub name: &'static str,
    pub arity: u8,           // 期待される引数の数
    pub slot: u16,
}
```

**TypeBox構造**:
```rust
pub struct TypeBox {
    pub type_name: &'static str,
    pub methods: &'static [MethodEntry],  // 静的メソッド一覧
}
```

**arity取得API**:
- `resolve_slot_by_name(type_name, method, arity) -> Option<u16>`: arity完全一致のスロットを返す
- `known_arities_for(type_name, method) -> Option<Vec<u8>>`: 既知のarity一覧を返す（診断用）

**実装例** (StringBox):
```rust
const STRING_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "len",     arity: 0, slot: 300 },
    MethodEntry { name: "indexOf", arity: 1, slot: 303 },  // indexOf(needle)
    MethodEntry { name: "indexOf", arity: 2, slot: 303 },  // indexOf(needle, from)
    // 同じメソッド名で複数arityをサポート（オーバーロード）
];
```

### 3. ExternRegistry のarity管理

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/extern_registry.rs:8-14`

```rust
pub struct ExternSpec {
    pub iface: &'static str,
    pub method: &'static str,
    pub min_arity: u8,    // 最小引数
    pub max_arity: u8,    // 最大引数
    pub slot: Option<u16>,
}

// 例:
ExternSpec {
    iface: "env.console",
    method: "log",
    min_arity: 1,
    max_arity: 255,  // 可変長引数
    slot: Some(10),
}
```

**arity検証API**:
```rust
pub fn check_arity(iface: &str, method: &str, argc: usize) -> Result<(), String> {
    if let Some(s) = resolve(iface, method) {
        if argc as u8 >= s.min_arity && argc as u8 <= s.max_arity {
            Ok(())
        } else {
            Err(format!("arity {} out of range {}..{}", argc, s.min_arity, s.max_arity))
        }
    } else {
        Err("unknown extern".to_string())
    }
}
```

---

## CallableBox.arity()の実現可能性

### 1. 静的arity vs 動的arity

| 種類 | 説明 | Hakoruneでの実現 |
|------|------|-----------------|
| **静的arity** | `ref Module.func/2` の `/2` 部分 | ✅ **実現可能** |
| **動的arity** | `cb.arity()` メソッド | ⚠️ **部分的に実現可能** |

**静的arityの問題点**:
- Hakoruneには現在 `ref Module.func/2` 構文が**存在しない**
- Parser groundworkは存在（`Parent::method` 参照: `docs/development/current/function_values_and_captures.md:9`）
- 実装には新しい構文拡張が必要

**動的arityの問題点**:
- `Callee::Value(vid)` 呼び出しで関数値が動的
- arity情報をどこに保存するか（FunctionBox構造体拡張が必要）

### 2. Callee別のarity取得難易度

| Callee variant | arity取得方法 | 実現難易度 | 備考 |
|---------------|-------------|----------|------|
| **Global** | 関数定義から取得 | 🟢 **低** | 関数シグネチャに含まれる |
| **Extern** | ExternRegistry から取得 | 🟢 **低** | `ExternSpec.min_arity/max_arity` 利用 |
| **ModuleFunction** | 関数定義から取得 | 🟢 **低** | シグネチャ `Class.method/N` の `/N` 部分 |
| **Method** | TypeRegistry から取得 | 🟡 **中** | 複数arityの場合は一覧を返す必要 |
| **Constructor** | Box定義から取得 | 🟡 **中** | `birth/N` のarityを取得 |
| **Closure** | Closure定義から取得 | 🟢 **低** | `NewClosure { params, .. }` に含まれる |
| **Value** | **動的dispatch** | 🔴 **高** | FunctionBox構造体拡張が必要 |

**Value variant の問題**:
```hakorune
local cb = handlers.get("unknown")  // 型不明
local arity = cb.arity()  // ❌ arityをどうやって取得？
```

現在の `FunctionBox` にはarity情報が**保存されていない**:
```rust
// src/boxes/function_box.rs:8
pub struct ClosureEnv {
    pub captures: HashMap<String, crate::value::NyashValue>,
    pub params: Vec<String>,  // パラメータ名のみ（arityは params.len()）
    pub me: Option<Arc<dyn NyashBox>>,
}
```

**解決策**:
1. `FunctionBox` に `arity: usize` フィールドを追加
2. `NewClosure` 命令時に `params.len()` を保存
3. `cb.arity()` 呼び出しで取得可能に

### 3. 部分適用のarity

ChatGPT提案:
```hakorune
local cb1 = ref Math.add/2  // arity = 2
local cb2 = cb1.partial([10])  // arity = 1（部分適用後）
```

**現状**:
- Hakoruneには `partial()` メソッドが**存在しない**
- `NewClosure` 命令で captures を保存する仕組みはある
- 部分適用は**新しい機能として実装が必要**

**実現難易度**: 🔴 **高**（Phase 20+ の大規模機能）

**理由**:
1. 新しいメソッド `CallableBox.partial(args)` の実装
2. 部分適用後の新しい Closure 生成
3. MIR レベルでの部分適用表現

---

## Fail-Fast設計との整合性

### 現在のFail-Fast箇所

| 箇所 | 検出タイミング | エラーメッセージ |
|------|-------------|----------------|
| **BoxCall** | 呼び出し直前 | `No matching method: MapBox.set(1 args). Available arities: [2]` |
| **MirCall** | Builder時 | （Builder側でarity検証なし、VM側で検出） |
| **ExternCall** | 呼び出し直前 | `arity 3 out of range 1..255` |

**問題点**:
- Builder時にarity検証を**していない**（VM実行時まで検出されない）
- CallableBox導入でarity検証を**前倒し可能**

### CallableBox導入後のFail-Fast

ChatGPT提案:
```hakorune
local cb = handlers.get("unknown")
if cb == null { /* エラー */ }

local result = cb.call([1, 2, 3])  // ❌ arity不一致ならここでFail-Fast
```

**実装方針**:
1. **登録時検証**: `handlers.set("key", cb)` 時にarity検証
2. **呼び出し時検証**: `cb.call(args)` 時にarity検証

**コード例**:
```rust
// CallableBox.call(args)
pub fn call(&self, args: &[VMValue]) -> Result<VMValue, VMError> {
    let expected_arity = self.arity();  // 期待arity
    let actual_arity = args.len();      // 実際のarity

    if actual_arity != expected_arity {
        return Err(VMError::InvalidInstruction(format!(
            "Callable arity mismatch: expected {}, got {}",
            expected_arity, actual_arity
        )));
    }

    // 実際の呼び出し
    self.invoke_internal(args)
}
```

**メリット**:
- ✅ **呼び出し時に即座にエラー検出**（既存のmaybe_arity_guardと同等）
- ✅ **null検出も同時に可能**（`handlers.get()` が null を返す場合）

---

## 既存コードとの互換性

### 既存のBoxCall

```hakorune
receiver.method(arg1, arg2)  // 既存構文
```

**内部的にCallableBoxに変換可能か？**

❌ **困難**

**理由**:
1. BoxCall は `receiver.method(args)` という**構文**
2. CallableBox は `cb.call(args)` という**値**
3. 構文を値に自動変換するには大規模な変更が必要

**互換性維持の方針**:
- 既存の BoxCall 構文は**そのまま維持**
- CallableBox は**新しい機能として追加**（破壊的変更なし）

### 既存のMirCall

```hakorune
Module.func(arg1, arg2)  // 既存構文
```

**内部的にCallableBoxに変換可能か？**

❌ **困難**

**理由**:
- MirCall は `Callee` enum で多様な呼び出しを表現
- CallableBox に統一するには全バックエンド（VM/LLVM/WASM）の書き換えが必要

**互換性維持の方針**:
- 既存の MirCall は**そのまま維持**
- CallableBox は**FunctionBox を拡張した新しいBox型として追加**

### 後方互換性

**破壊的変更**: ❌ **なし**

**理由**:
1. CallableBox は**新しいBox型**として追加
2. 既存の BoxCall/MirCall 構文は変更しない
3. FunctionBox の拡張は内部的な変更のみ（API互換性維持）

**移行コスト**: 🟡 **中**

**必要な作業**:
1. FunctionBox に `arity()` メソッド追加
2. `NewClosure` 命令で arity 情報を保存
3. CallableBox API の設計・実装（`call()`, `arity()`, `partial()`）
4. ドキュメント・サンプルコード作成

---

## 総合評価

### ChatGPT提案の整合性

**既存arity実装との整合性**: 🟢 **高**

**理由**:
1. ✅ TypeRegistry の `MethodEntry.arity` と同じ概念
2. ✅ ExternRegistry の `min_arity/max_arity` と整合
3. ✅ maybe_arity_guard と同じFail-Fast設計
4. ✅ 既存のarity検証パターンを踏襲

### 実現可能性

**総合評価**: 🟡 **実現可能だが大規模実装必要**

**段階的実装計画**:

#### Phase 1: FunctionBox.arity() 実装（2-3人日）
- ✅ FunctionBox に `arity` フィールド追加
- ✅ `NewClosure` 命令で `params.len()` を保存
- ✅ `arity()` メソッド実装
- ✅ VM Backend サポート

#### Phase 2: CallableBox.call() 実装（3-5人日）
- ✅ `CallableBox.call(args)` メソッド実装
- ✅ arity検証の組み込み
- ✅ null検証の組み込み
- ✅ 既存の MirCall/BoxCall との統合テスト

#### Phase 3: 静的arity構文 `ref Module.func/2`（5-8人日）
- ⚠️ Parser 拡張（新しい構文）
- ⚠️ MIR Builder 拡張
- ⚠️ arity検証の前倒し（Builder時）

#### Phase 4: 部分適用 `cb.partial(args)`（8-12人日）
- 🔴 新しい機能として大規模実装
- 🔴 MIR レベルでの表現方法の設計
- 🔴 全バックエンド対応

**優先度**:
1. **Phase 1**: 🟢 **高**（既存機能の自然な拡張）
2. **Phase 2**: 🟢 **高**（Fail-Fast設計の強化）
3. **Phase 3**: 🟡 **中**（新機能、破壊的変更なし）
4. **Phase 4**: 🔴 **低**（Phase 20+ の大規模機能）

---

## 推奨事項

### 1. 最小実装（Phase 1-2）を優先

**理由**:
- FunctionBox.arity() は既存の仕組みで実現可能
- Fail-Fast設計を強化できる
- 既存コードとの互換性を維持

**スコープ**:
```hakorune
// ✅ 実現可能
local cb = function(a, b) { return a + b }
print(cb.arity())  // → 2
local result = cb.call([1, 2])  // → 3

// ✅ arity不一致検出
local result2 = cb.call([1, 2, 3])  // ❌ Error: expected 2, got 3
```

### 2. 静的arity構文（Phase 3）は後回し

**理由**:
- Parser 拡張が必要（大規模）
- 動的arity（`cb.arity()`）で十分なケースが多い
- 破壊的変更なし、追加的機能として実装可能

### 3. 部分適用（Phase 4）は Phase 20+

**理由**:
- 大規模な新機能
- MIR レベルでの設計が必要
- 既存の Closure 機構と統合が必要

---

## 結論

**ChatGPT提案は既存実装と高度に整合している。Phase 1-2（FunctionBox.arity() + CallableBox.call()）の実装を推奨する。**

**期待される効果**:
1. ✅ Fail-Fast設計の強化（呼び出し時にarity検証）
2. ✅ 既存のTypeRegistry/ExternRegistryとの統一感
3. ✅ 後方互換性の維持（破壊的変更なし）
4. ✅ Phase 20+ の部分適用への布石

**実装優先度**:
- Phase 1-2: 🟢 **今すぐ実装可能**（2-3週間）
- Phase 3: 🟡 **Phase 19完了後**（1-2ヶ月）
- Phase 4: 🔴 **Phase 20+**（3-6ヶ月）
