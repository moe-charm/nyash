# Collections API Phase 1 統一計画とHakorune VM実装の関連調査

**調査日**: 2025-10-10
**調査者**: Claude (Anthropic)
**目的**: Collections API Phase 1 (size/isEmpty統一)がHakorune VMのハードコーディング削減とどう関連するか調査

---

## 📋 Executive Summary

### 主要発見

✅ **Collections API Phase 1 は Rust VM 側の統一計画**:
- **対象**: Rust VM の ArrayBox/MapBox/StringBox の API統一
- **目的**: `size()/isEmpty()` の統一、`MapBox.get(missing) → null` の修正
- **範囲**: Rust VM のみ（Hakorune VM は対象外）

✅ **Hakorune VM は別の課題を持つ**:
- **主要課題**: MirCall Phase 2 (ModuleFunction/Method) 未実装
- **副次課題**: ArrayBox/MapBox 参照保持問題（調査中）
- **Collections API**: StringBox のみ完全動作、ArrayBox/MapBox は問題あり

⚠️ **両者の関連性**:
- **直接的関連**: なし（Rust VM と Hakorune VM は別レイヤー）
- **間接的関連**: 両方とも「Core Box判定」の統一が必要
- **共通基盤**: `hako_core_*` crates の意味論を共有すべき

---

## 1. Collections API Phase 1 計画

### 1.1 元の計画（CURRENT_TASK.md）

**Step A — 構造的解決（最優先・小差分）**:
```rust
// type_registry.rs
pub static CORE_BOXES: Lazy<HashMap<&'static str, CoreBoxEntry>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("MapBox",    CoreBoxEntry { type_id: 11, factory: create_map_box });
    m.insert("ArrayBox",  CoreBoxEntry { type_id: 12, factory: create_array_box });
    m.insert("StringBox", CoreBoxEntry { type_id: 13, factory: create_string_box });
    m
});

pub fn is_core_box(type_name: &str) -> bool {
    CORE_BOXES.contains_key(type_name)
}
```

**置き換え対象**:
```rust
// Before (hardcoding)
if matches!(type_name, "ArrayBox" | "MapBox" | "StringBox") { ... }

// After (SSOT)
if is_core_box(type_name) { ... }
```

**Step B — ドキュメント**: `docs/architecture/single-route-single-face.md` に責務を集約

**Step C — スモーク**: plugin-on/strict で動作検証

**Step D — コード**: 意味論を `hako_core_*` crates に移譲

---

### 1.2 ChatGPT 評価結果（40/100点の理由）

**評価**: 40/100点（**構造優先主義の欠如**）

**問題点**:
1. ❌ **Step A-D の順序が逆**: コード変更（Step D）を先にやると構造が見えない
2. ❌ **責務の文書化なし**: 各 Box の意味論が分散（ドキュメント不足）
3. ⚠️ **スモークの不足**: Map.keys() の順序、Array.slice の境界など

**改善案**: Step A（構造）→ Step B（ドキュメント）→ Step C（スモーク）→ Step D（コード）

**自動登録の可能性**: Plugin near-spec を利用して type_id を自動記録（既に実装済み）

---

### 1.3 現在の実装状況（Rust VM）

#### is_core_box() の実装

**ファイル**: `src/runtime/type_registry.rs:431`

```rust
pub static CORE_BOXES: Lazy<HashMap<&'static str, CoreBoxEntry>> = Lazy::new(|| {
    let mut m = HashMap::new();
    let ids = (|| {
        for cfg in ["hako.toml", "nyash.toml", "hakorune.toml"].iter() {
            if let Ok(conf) = crate::config::nyash_toml_v2::NyashConfigV2::from_file(cfg) {
                if !conf.box_types.is_empty() { return conf.box_types; }
            }
        }
        HashMap::new()
    })();
    let tid = |name: &str, d: u32| ids.get(name).copied().unwrap_or(d);
    m.insert("MapBox",    CoreBoxEntry { type_id: tid("MapBox", 11), factory: create_map_box });
    m.insert("ArrayBox",  CoreBoxEntry { type_id: tid("ArrayBox", 12), factory: create_array_box });
    m.insert("StringBox", CoreBoxEntry { type_id: tid("StringBox", 13), factory: create_string_box });
    m
});

#[inline]
pub fn is_core_box(type_name: &str) -> bool {
    CORE_BOXES.contains_key(type_name)
}
```

**特徴**:
- ✅ SSOT (Single Source of Truth): CORE_BOXES が唯一の真実
- ✅ 設定ファイル対応: hako.toml/nyash.toml/hakorune.toml から type_id 読み込み
- ✅ Fallback: 設定なしの場合はデフォルト値（11/12/13）

---

#### 使用箇所（9箇所）

**ファイル**: `src/` 配下

```rust
// 1. MIR Builder Policy
src/mir/builder/router/policy.rs:51
    if crate::runtime::type_registry::is_core_box(box_name) { ... }

// 2. Codec
src/runtime/codec/mod.rs:39
    if crate::runtime::type_registry::is_core_box(a.type_name()) { ... }

// 3. Method Router
src/runtime/method_router_box/mod.rs:73
    if crate::runtime::type_registry::is_core_box(bx.type_name()) { ... }

src/runtime/method_router_box/mod.rs:83
    let is_core = crate::runtime::type_registry::is_core_box(&p.box_type);

// 4. Provider Box
src/runtime/provider_box/mod.rs:63
    let is_core = crate::runtime::type_registry::is_core_box(box_type);

// 5. VM Handlers
src/backend/mir_interpreter/handlers/calls/legacy/method_handler.rs:31
    if crate::runtime::type_registry::is_core_box(tn.as_str()) { type_name = Some(tn); }

src/backend/mir_interpreter/handlers/boxes/legacy/plugin_bridge.rs:260
    if crate::runtime::type_registry::is_core_box(recv_box.type_name()) { ... }

src/backend/mir_interpreter/handlers/boxes/legacy/mod.rs:187
    VMValue::BoxRef(bx) => crate::runtime::type_registry::is_core_box(bx.type_name())
```

**統計**:
- ✅ **9箇所で統一使用**（ハードコーディングなし）
- ✅ **SSOT パターン確立**（`is_core_box()` 経由のみ）

---

### 1.4 ハードコーディング分類（Rust VM）

#### 削除済み箇所

**調査結果**: `matches!(..., "ArrayBox"|"MapBox"|"StringBox")` パターンは発見されず

**推測**: 既に `is_core_box()` に置き換え済み

---

#### 必要な箇所（LLVM最適化）

**対象外**: Rust VM の Collections API Phase 1 は LLVM 最適化に関係なし

**理由**: LLVM バックエンドは別の最適化経路を持つ

---

### 1.5 hako_core_* crates の役割

#### 意味論の集約

**crate 一覧**:
- `hako_core_map`: Map の意味論（keys/values の順序、size/isEmpty）
- `hako_core_array`: Array の意味論（slice 境界、get/set のインデックス処理）
- `hako_core_string`: String の意味論（length/substring/indexOf）

**実装例**:

```rust
// hako_core_map/src/lib.rs
pub fn size(len: usize) -> i64 { len as i64 }

pub fn keys_sorted_from_map_str<V>(m: &HashMap<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();  // Dictionary order (lexicographic)
    keys
}

// hako_core_array/src/lib.rs
pub fn slice_bounds(len: usize, start: i64, end: i64) -> (usize, usize) {
    let l = len as i64;
    let mut i0 = start.max(0).min(l) as usize;
    let mut i1 = if end < 0 { len } else { end.max(0).min(l) as usize };
    if i0 > i1 { i0 = i1; }
    (i0, i1)
}

pub fn safe_get_index(len: usize, idx: i64) -> Option<usize> {
    if idx < 0 { return None; }
    let u = idx as usize;
    if u < len { Some(u) } else { None }
}
```

**使用箇所** (30箇所):
- `src/boxes/array/mod.rs`: ArrayBox の実装で使用
- `src/boxes/map_box.rs`: MapBox の実装で使用
- `src/runtime/method_router_box/mod.rs`: StringBox ルーティングで使用
- `src/backend/mir_interpreter/extern_adapter.rs`: Extern 呼び出しで使用

**統計**:
- ✅ **30箇所で使用**（重複実装なし）
- ✅ **SSOT パターン確立**（`hako_core_*` 経由のみ）

---

## 2. Rust VM 統一状況

### 2.1 Core Box 判定

#### CORE_BOXES の定義

**ファイル**: `src/runtime/type_registry.rs:411-427`

**構造**:
```rust
pub struct CoreBoxEntry {
    pub type_id: u32,
    pub factory: fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError>,
}

pub static CORE_BOXES: Lazy<HashMap<&'static str, CoreBoxEntry>> = Lazy::new(|| {
    // 設定ファイルから type_id 読み込み (hako.toml > nyash.toml > hakorune.toml)
    // Fallback: MapBox=11, ArrayBox=12, StringBox=13
});
```

**特徴**:
- ✅ **type_id と factory を統一管理**
- ✅ **設定ファイル対応**（dynamic type_id assignment）
- ✅ **Fallback 機能**（設定なしでもデフォルト値）

---

#### is_core_box() の使われ方

**パターン1**: Type 判定

```rust
// src/runtime/provider_box/mod.rs:63
let is_core = crate::runtime::type_registry::is_core_box(box_type);
if is_core && plugin_on_strict {
    // Fail-Fast: core box はプラグイン必須
    return Err(RuntimeError::PluginRequired { box_type });
}
```

**パターン2**: Routing 判定

```rust
// src/runtime/method_router_box/mod.rs:73
if crate::runtime::type_registry::is_core_box(bx.type_name()) {
    // Core box は builtin invoker へ
    return invoke_builtin_method(bx, method, args);
}
```

**パターン3**: Policy 判定

```rust
// src/mir/builder/router/policy.rs:51
if crate::runtime::type_registry::is_core_box(box_name) {
    // Core box は plugin-on policy 適用
    apply_plugin_on_policy(box_name);
}
```

---

### 2.2 ハードコーディング箇所

#### 削除可能な箇所

**調査結果**: 0件（既に `is_core_box()` に置き換え済み）

**検証**:
```bash
grep -r 'matches!.*ArrayBox.*MapBox.*StringBox' src/
# → 0件（.bak ファイルのみ）
```

---

#### 必要な箇所

**該当なし**: Collections API Phase 1 は LLVM 最適化とは無関係

**理由**:
- LLVM バックエンドは別の最適化経路（LLVM IR レベル）
- Core Box 判定は Runtime レベルの話

---

## 3. Hakorune VM 実装への示唆

### 3.1 Core Box vs User Box

#### Hakorune VM の課題

**現状**:
- ✅ StringBox: 完全動作（4/9 テスト PASS）
- ⚠️ ArrayBox: 参照保持問題（5/9 テスト失敗）
- ⚠️ MapBox: 参照保持問題（5/9 テスト失敗）

**問題**:
- **ArrayBox.push() 後に size() が 0 を返す**
- **原因**: Selfhost VM（Hakoruneスクリプト）⇔ Rust VM 連携の問題（調査中）

---

#### Core Box 判定の必要性

**Hakorune VM での実装案**:

**Option A**: Rust VM の `is_core_box()` を呼び出す（推奨）

```hako
// 疑似コード (Hakorune VM)
static box CoreBoxRegistry {
    core_types: MapBox  // {"ArrayBox": 1, "MapBox": 1, "StringBox": 1}

    birth() {
        me.core_types = new MapBox()
        me.core_types.set("ArrayBox", 1)
        me.core_types.set("MapBox", 1)
        me.core_types.set("StringBox", 1)
    }

    is_core_box(type_name: StringBox) {
        // Rust VM に委譲（推奨）
        return RustVmBridge.is_core_box(type_name)
    }
}
```

**Option B**: Hakorune VM 内で独自実装

```hako
static box CoreBoxRegistry {
    is_core_box(type_name: StringBox) {
        if type_name == "ArrayBox" { return 1 }
        if type_name == "MapBox" { return 1 }
        if type_name == "StringBox" { return 1 }
        return 0
    }
}
```

**推奨**: **Option A（Rust VM 委譲）**
- **理由**: SSOT（Single Source of Truth）を維持
- **利点**: Rust VM と Hakorune VM の判定が一致
- **欠点**: Rust VM への依存

---

### 3.2 共通インターフェース

#### size()/isEmpty() の実装

**Rust VM の実装**:

```rust
// ArrayBox
impl NyashBox for ArrayBox {
    fn invoke(&mut self, method: &str, args: &[Box<dyn NyashBox>]) -> Box<dyn NyashBox> {
        match method {
            "size" | "length" | "len" => {
                Box::new(IntegerBox::new(hako_core_array::length(self.items.len())))
            }
            "isEmpty" => {
                Box::new(BoolBox::new(self.items.is_empty()))
            }
            // ...
        }
    }
}

// MapBox
impl NyashBox for MapBox {
    fn invoke(&mut self, method: &str, args: &[Box<dyn NyashBox>]) -> Box<dyn NyashBox> {
        match method {
            "size" => {
                let guard = self.inner.read().unwrap();
                Box::new(IntegerBox::new(hako_core_map::size(guard.len())))
            }
            "isEmpty" => {
                let guard = self.inner.read().unwrap();
                Box::new(BoolBox::new(guard.is_empty()))
            }
            // ...
        }
    }
}
```

---

#### Hakorune VM での実装

**現状**: BoxCall 経由で Rust VM の ArrayBox/MapBox を呼び出し

**問題**: 参照保持問題により size() が正しく動作しない

**解決策**:

**Option A**: Rust VM への完全委譲（推奨）

```hako
// MirCall Handler (Hakorune VM)
static box MirCallHandlerBox {
    handle_boxcall(recv: IntegerBox, method: StringBox, args: ArrayBox) {
        // Rust VM に委譲
        local result = RustVmBridge.invoke_boxcall(recv, method, args)
        return result
    }
}
```

**Option B**: Selfhost VM 内で size/isEmpty を実装

```hako
// BoxCall Handler (Hakorune VM)
static box BoxCallHandlerBox {
    handle(recv: IntegerBox, method: StringBox, args: ArrayBox) {
        // Core Box 判定
        local type_name = recv.type_name()
        if CoreBoxRegistry.is_core_box(type_name) {
            // Common interface
            if method == "size" { return me.handle_size(recv) }
            if method == "isEmpty" { return me.handle_isEmpty(recv) }
        }
        // User Box はそのまま
        return RustVmBridge.invoke_boxcall(recv, method, args)
    }

    handle_size(recv: IntegerBox) {
        // ArrayBox/MapBox/StringBox 共通
        // （実装は Rust VM に委譲）
        return RustVmBridge.get_size(recv)
    }
}
```

**推奨**: **Option A（完全委譲）**
- **理由**: Hakorune VM は「MIR 生成器」として機能すれば十分
- **利点**: 実装量が少ない（~50行）、Rust VM と動作が一致
- **欠点**: Rust VM への依存

---

### 3.3 ハードコーディング削減戦略

#### Hakorune VM の現状

**ハードコーディング箇所**:
- ❌ **BoxCall Handler**: ArrayBox/MapBox/StringBox の分岐（推測）
- ❌ **MirCall Handler**: Global/Extern の分岐（実装済み）
- ⏳ **未実装**: Method/ModuleFunction（MirCall Phase 2）

**削減戦略**:

**Strategy 1**: Core Box Registry 導入

```hako
// Before (hardcoding)
if type_name == "ArrayBox" {
    // ArrayBox 専用処理
} else if type_name == "MapBox" {
    // MapBox 専用処理
} else if type_name == "StringBox" {
    // StringBox 専用処理
}

// After (SSOT)
if CoreBoxRegistry.is_core_box(type_name) {
    // Core Box 共通処理
    return me.handle_core_box(type_name, method, args)
}
```

**Strategy 2**: Rust VM 委譲パターン

```hako
// Before (hardcoding in Hakorune VM)
static box BoxCallHandlerBox {
    handle(recv, method, args) {
        local type_name = recv.type_name()
        if type_name == "ArrayBox" { /* ArrayBox 処理 */ }
        if type_name == "MapBox" { /* MapBox 処理 */ }
        if type_name == "StringBox" { /* StringBox 処理 */ }
    }
}

// After (delegation to Rust VM)
static box BoxCallHandlerBox {
    handle(recv, method, args) {
        // すべて Rust VM に委譲
        return RustVmBridge.invoke_boxcall(recv, method, args)
    }
}
```

**推奨**: **Strategy 2（Rust VM 委譲）**
- **理由**: Hakorune VM の責務を「MIR 生成」に限定
- **利点**: 実装量が少ない、Rust VM と動作が一致
- **欠点**: Rust VM への依存（ただし、これは設計方針）

---

## 4. 統合プラン

### 4.1 Rust VM 側

#### Step A: 構造的解決（完了）

✅ **is_core_box() 実装済み**:
- `CORE_BOXES` による SSOT 確立
- 9箇所で統一使用
- ハードコーディングなし

✅ **hako_core_* crates 実装済み**:
- 30箇所で使用
- 意味論の集約完了

---

#### Step B: ドキュメント（一部完了）

✅ **single-route-single-face.md 作成済み**:
- Core semantics 責務を明記
- size/isEmpty 統一
- Map.get(missing) → null 仕様

⏳ **未完了**:
- Array.slice 境界の詳細仕様（実装はあるがドキュメント不足）
- String.substring 境界の詳細仕様（実装はあるがドキュメント不足）

---

#### Step C: スモーク（一部完了）

✅ **plugin-on profile 実装済み**:
- `NYASH_PLUGIN_ON_STRICT=1` で Fail-Fast 動作

⏳ **未完了**:
- Map.keys() 順序のテスト（lexicographic order）
- Array.slice 負数 end のテスト（clamp-to-len）

---

#### Step D: コード（完了）

✅ **意味論の移譲完了**:
- ArrayBox → `hako_core_array::*`
- MapBox → `hako_core_map::*`
- StringBox → `hako_core_string::*`（Router 経由）

---

### 4.2 Hakorune VM 側

#### Phase 1: 必須実装（最優先）⭐

**Step 1: ArrayBox/MapBox 問題修正**（見積もり: 0.5-1人日）

**内容**:
- Task Teacher で根本原因特定
- Rust VM 修正（必要に応じて）
- Collection API 全テスト再実行

**成果**:
- ArrayBox/MapBox BoxCall テスト 9/9 PASS
- Collection API 完全動作

---

**Step 2: Core Box Registry 導入**（見積もり: 0.5人日）

**内容**:
- `CoreBoxRegistry.is_core_box()` 実装
- Rust VM の `is_core_box()` に委譲

**成果**:
- Core Box 判定の統一
- ハードコーディング削減

---

#### Phase 2: MirCall Phase 2 実装（次の優先）

**Step 3: ModuleFunction 実装**（見積もり: 2-3人日）

**内容**:
- Rust VM 委譲ブリッジの設計
- 最小プロトタイプ実装

**成果**:
- ModuleFunction 呼び出し動作
- Selfhost compiler の基本機能が動作

---

**Step 4: Method 実装**（見積もり: 1-2人日）

**内容**:
- ModuleFunction ブリッジを Method に拡張

**成果**:
- Method 呼び出し動作
- MirCall Phase 2 完全実装

---

### 4.3 両者の連携

#### 共通基盤の確立

**原則**:
- **SSOT**: `hako_core_*` crates が唯一の真実
- **委譲**: Hakorune VM は Rust VM に委譲
- **検証**: 両者のテストで動作を検証

**実装パターン**:

```
                   ┌─────────────────┐
                   │ hako_core_*     │
                   │ crates          │
                   │ (意味論)        │
                   └────────┬────────┘
                            │
              ┌─────────────┴─────────────┐
              ↓                           ↓
    ┌─────────────────┐         ┌─────────────────┐
    │ Rust VM         │         │ Hakorune VM     │
    │ (直接使用)      │←───────│ (Rust VM委譲)  │
    │                 │ Bridge  │                 │
    └─────────────────┘         └─────────────────┘
```

**利点**:
- ✅ **SSOT 維持**: 意味論が `hako_core_*` に集約
- ✅ **動作一致**: Rust VM と Hakorune VM が同じ動作
- ✅ **保守性**: 変更箇所が 1 箇所（`hako_core_*`）

---

## 5. 工数見積もり

### 5.1 Rust VM 側（Collections API Phase 1）

| Step | 内容 | 見積もり | 状態 |
|------|------|---------|------|
| **Step A** | 構造的解決 (is_core_box) | - | ✅ 完了 |
| **Step B** | ドキュメント (single-route-single-face.md) | 0.5人日 | ⏳ 一部完了 |
| **Step C** | スモーク (plugin-on/strict) | 1人日 | ⏳ 一部完了 |
| **Step D** | コード (hako_core_* 委譲) | - | ✅ 完了 |
| **合計** | | **1.5人日** | |

---

### 5.2 Hakorune VM 側

| Step | 内容 | 見積もり | 優先度 |
|------|------|---------|--------|
| **Step 1** | ArrayBox/MapBox 修正 | 0.5-1人日 | ⭐ 最優先 |
| **Step 2** | Core Box Registry 導入 | 0.5人日 | 高 |
| **Step 3** | ModuleFunction 実装 | 2-3人日 | ⭐ 最優先 |
| **Step 4** | Method 実装 | 1-2人日 | ⭐ 最優先 |
| **合計** | | **4.5-6.5人日** | |

---

### 5.3 Critical Path

**最優先パス**: Hakorune VM の ArrayBox/MapBox 修正 → MirCall Phase 2 実装

**理由**:
- ArrayBox/MapBox 問題を先に解決（BoxCall の基盤）
- MirCall Phase 2 が Selfhost compiler 完全動作に必須

**最短期間**: **4.5人日**（楽観的）
**現実的期間**: **6.5人日**（保守的）

---

## 6. 推奨アクション

### 6.1 Rust VM 側（Collections API Phase 1）

**アクション1: ドキュメント完成**（0.5人日）

**内容**:
- Array.slice 境界の詳細仕様追記
- String.substring 境界の詳細仕様追記

**成果**:
- `single-route-single-face.md` 完全版

---

**アクション2: スモーク追加**（1人日）

**内容**:
- Map.keys() 順序テスト（lexicographic order）
- Array.slice 負数 end テスト（clamp-to-len）

**成果**:
- plugin-on/strict profile 完全版

---

### 6.2 Hakorune VM 側

**アクション1: ArrayBox/MapBox 修正**（0.5-1人日）⭐最優先

**内容**:
- Task Teacher で根本原因特定
- Rust VM 修正（必要に応じて）

**成果**:
- BoxCall 完全動作

---

**アクション2: Core Box Registry 導入**（0.5人日）

**内容**:
- `CoreBoxRegistry.is_core_box()` 実装
- Rust VM 委譲ブリッジ

**成果**:
- Core Box 判定の統一

---

**アクション3: MirCall Phase 2 実装**（3-5人日）⭐最優先

**内容**:
- ModuleFunction 実装（2-3人日）
- Method 実装（1-2人日）

**成果**:
- Selfhost compiler 完全動作

---

## 7. まとめ

### 7.1 主要発見

✅ **Collections API Phase 1 は Rust VM 側の計画**:
- is_core_box() 実装済み（SSOT 確立）
- hako_core_* crates 実装済み（意味論の集約）
- ハードコーディングなし（9箇所で統一使用）

✅ **Hakorune VM は別の課題を持つ**:
- ArrayBox/MapBox 参照保持問題（調査中）
- MirCall Phase 2 未実装（最優先）

⚠️ **両者の関連性**:
- **直接的関連**: なし（別レイヤー）
- **間接的関連**: Core Box 判定の統一
- **共通基盤**: `hako_core_*` crates の意味論

---

### 7.2 推奨実装順序

**Rust VM 側**:
1. ドキュメント完成（0.5人日）
2. スモーク追加（1人日）

**Hakorune VM 側**:
1. ArrayBox/MapBox 修正（0.5-1人日）⭐最優先
2. Core Box Registry 導入（0.5人日）
3. ModuleFunction 実装（2-3人日）⭐最優先
4. Method 実装（1-2人日）⭐最優先

**合計**: **6-8人日**（Rust VM 1.5人日 + Hakorune VM 4.5-6.5人日）

---

### 7.3 成功の定義

**Rust VM 側**:
- ✅ single-route-single-face.md 完全版
- ✅ plugin-on/strict テスト緑

**Hakorune VM 側**:
- ✅ ArrayBox/MapBox 完全動作（9/9 テスト PASS）
- ✅ Core Box Registry 統一
- ✅ MirCall Phase 2 完全実装
- ✅ Selfhost compiler 完全動作

---

### 7.4 次のアクション

**今すぐ実施** ⚡:
- **Hakorune VM**: ArrayBox/MapBox 問題修正（0.5-1人日）

**次に実施**:
- **Hakorune VM**: Core Box Registry 導入（0.5人日）
- **Hakorune VM**: MirCall Phase 2 実装（3-5人日）

**その後**:
- **Rust VM**: ドキュメント完成（0.5人日）
- **Rust VM**: スモーク追加（1人日）

---

## 付録: 参考資料

### A. Rust VM 関連

- **single-route-single-face.md**: Collections 責務の文書化
- **type_registry.rs**: is_core_box() 実装
- **hako_core_* crates**: 意味論の実装

### B. Hakorune VM 関連

- **mini_vm_progress.md**: 開発進捗（Phase 1-4 Day 11）
- **rust_vm_vs_hakorune_vm_gap_analysis.md**: ギャップ分析

### C. 設計書

- **collection-api-INDEX.md**: Collections API 提案
- **collection-api-adr-2025-10-09.md**: ADR（決定記録）

---

**調査完了日**: 2025-10-10
**次回更新**: Hakorune VM Phase 1 完了時
