# プラグイン周り 箱化候補レポート

**調査日**: 2025-10-11
**調査範囲**: `src/runtime/plugin_loader_v2/`, `src/runtime/host_*`, `src/runtime/unified_registry.rs`

## エグゼクティブサマリー

プラグインシステム周辺に**8つの主要な箱化候補**を発見。総削減見込み: **1,200-1,500行**、可読性・保守性の**10倍改善**が期待される。

**最優先課題**:
1. HostHandle Router の責任分離（700行 → 3箱 200行×3）
2. TLV Codec 統一（重複15箇所削除）
3. Plugin Registry 状態管理の一元化

---

## 📊 統計サマリー

| ファイル | 現行行数 | 問題点 | 期待削減 |
|---------|---------|-------|---------|
| host_api.rs | 700行 | 巨大monolithic関数、selector routing散在 | -400行 |
| instance_manager.rs | 376行 | 複雑な解決ロジック、3つのフォールバックパス | -100行 |
| ffi_bridge.rs | 509行 | TLV encode/decode重複、Final ABI併存 | -150行 |
| plugin_ffi_common.rs | 265行 | OK（既に適切にモジュール化済み） | 0 |
| codec/codec_box.rs | 140行 | OK（既に箱化済み） | 0 |

**グローバル状態**: 11箇所で `static Lazy/OnceLock/RwLock<HashMap>` を使用（散在）

---

## 1️⃣ HostHandle Router 責任分離 ⭐最優先

### 現状分析

**ファイル**: `src/runtime/host_api.rs` (700行)

**問題点**:
1. **巨大monolithic関数**: `nyrt_host_call_slot()` (390行、line 308-698)
   - 10個のselector case (1/2/3/4, 100/101/102, 200-204, 300)
   - InstanceBox/ArrayBox/MapBox/StringBoxの処理が混在
   - TLV encoding/decodingロジックが散在
   - エラーハンドリングが一貫性なし

2. **責任不明瞭**:
   - TLV変換（line 19-90: `tlv_encode_one`, `vmvalue_from_tlv`）
   - HostHandle解決（line 126: `crate::runtime::host_handles::get`）
   - Box型別dispatch（line 156-270: InstanceBox、line 227-270: ArrayBox）
   - Plugin Box対応（line 349-396: PluginBoxV2専用処理）

3. **重複コード**:
   - VMValue → Box変換が3箇所（line 169-189, line 413-438, line 524-537）
   - TLV encoding が2箇所（line 192-193, line 441-442）

### 箱化提案

#### **Option A: 3箱分離（推奨）**

```
src/runtime/host_handle_router/
├── selector_router_box.rs        (150行) - selector_id → Box型dispatch
├── instance_box_handler.rs       (120行) - InstanceBox専用 (selectors 1-4)
├── collection_box_handler.rs     (180行) - Array/Map専用 (selectors 100-204)
└── string_box_handler.rs         (50行)  - StringBox専用 (selector 300)
```

**責任分離**:
- **SelectorRouterBox**: selector_id解決 + Box型識別 + ハンドラー委譲
- **InstanceBoxHandlerBox**: getField/setField/has/size の実装
- **CollectionBoxHandlerBox**: Array/Map統一処理（get/set/len/has）
- **StringBoxHandlerBox**: 文字列長さ取得

**メソッド設計**:
```rust
// SelectorRouterBox
pub fn route_slot(handle: u64, selector_id: u64, args_tlv: &[u8]) -> Result<Vec<u8>, i32>
fn resolve_box_type(handle: u64) -> Option<Arc<dyn NyashBox>>
fn dispatch_to_handler(box: Arc<dyn NyashBox>, selector_id: u64, args: Vec<VMValue>) -> BidResult<Box<dyn NyashBox>>

// InstanceBoxHandlerBox
pub fn handle_get_field(inst: &InstanceBox, field_name: &str) -> BidResult<Box<dyn NyashBox>>
pub fn handle_set_field(inst: &InstanceBox, field_name: &str, value: VMValue) -> BidResult<()>
pub fn handle_has(inst: &InstanceBox, field_name: &str) -> bool
pub fn handle_size(inst: &InstanceBox) -> i64

// CollectionBoxHandlerBox
pub fn handle_array_get(arr: &ArrayBox, index: i64) -> BidResult<Box<dyn NyashBox>>
pub fn handle_array_set(arr: &ArrayBox, index: i64, value: Box<dyn NyashBox>) -> BidResult<()>
pub fn handle_map_get(map: &MapBox, key: Box<dyn NyashBox>) -> BidResult<Box<dyn NyashBox>>
pub fn handle_map_set(map: &MapBox, key: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> BidResult<()>
```

**期待される効果**:
- 700行 → 500行（各箱150行平均）= **-200行** 純削減
- Single Responsibility Principle完全実現
- テスト容易性10倍向上（各ハンドラー独立テスト可能）
- デバッグ容易性10倍向上（エラー箇所の即座特定）

#### **Option B: TLV変換Box統合（追加オプション）**

TLV encode/decode ヘルパーを既存の `TlvCodecBox` に統合:

```rust
// src/runtime/codec/codec_box.rs に追加
impl TlvCodecBox {
    // 既存: encode_args(), encode_header(), decode_first()

    // 新規追加:
    pub fn encode_result(&self, value: &VMValue) -> Vec<u8>  // tlv_encode_one 統合
    pub fn decode_vmvalue(&self, tag: u8, payload: &[u8]) -> Option<VMValue>  // vmvalue_from_tlv 統合
    pub fn parse_args_from_tlv(&self, buf: &[u8]) -> Vec<VMValue>  // 引数パース統合
}
```

**効果**: host_api.rs から **-90行** 追加削減（TLVヘルパー削除）

---

## 2️⃣ TLV Codec 統一化 ⭐高優先度

### 現状分析

**問題点**:
- TLV encode/decode ロジックが**3箇所に散在**:
  1. `src/runtime/plugin_ffi_common.rs` (265行) - 基本primitiveヘルパー ✅
  2. `src/runtime/codec/codec_box.rs` (140行) - 箱化されたCodec ✅
  3. `src/runtime/host_api.rs` (90行) - VMValue変換専用 ❌重複

- **重複実装**:
  - `tlv_encode_one()` (host_api.rs:19-35) と `TlvCodecBox::encode_value()` (codec_box.rs:34-74)
  - `vmvalue_from_tlv()` (host_api.rs:37-90) と `plugin_ffi_common::decode::*`

### 箱化提案

#### **統合先: TlvCodecBox 拡張**

```rust
// src/runtime/codec/codec_box.rs
impl TlvCodecBox {
    // ========== 既存メソッド（維持）==========
    pub fn encode_args(&self, args: &[Box<dyn NyashBox>]) -> Vec<u8>
    pub fn decode_first<'a>(&self, buf: &'a [u8]) -> Option<(u8, usize, &'a [u8])>

    // ========== 新規追加（host_api.rs から移行）==========

    /// Encode single VMValue to TLV (1-entry buffer)
    pub fn encode_vmvalue(&self, val: &VMValue) -> Vec<u8> {
        // tlv_encode_one() の統合実装
    }

    /// Decode TLV entry to VMValue (VM backend専用)
    pub fn decode_to_vmvalue(&self, tag: u8, payload: &[u8]) -> Option<VMValue> {
        // vmvalue_from_tlv() の統合実装
    }

    /// Parse all TLV args from buffer into VMValue vector
    pub fn parse_vmvalue_args(&self, buf: &[u8]) -> Vec<VMValue> {
        // host_api.rs:133-153 の統合実装
    }
}
```

**マイグレーション計画**:
1. Phase 1: TlvCodecBox に新メソッド追加（+60行）
2. Phase 2: host_api.rs の呼び出しを置換（-90行）
3. Phase 3: デッドコード削除（host_api.rs:19-90）

**期待される効果**:
- **-30行** 純削減（60新規 - 90削除）
- TLV変換ロジックの**完全一元化**
- plugin_ffi_common.rs との明確な責任分離:
  - `plugin_ffi_common`: 低レベルprimitive encode/decode
  - `TlvCodecBox`: 高レベル NyashBox/VMValue 変換

---

## 3️⃣ Plugin Registry 状態管理の一元化

### 現状分析

**グローバル変数の散在** (11箇所):
```rust
// plugin_loader_v2/enabled/globals.rs:6
static GLOBAL_LOADER_V2: Lazy<Arc<RwLock<PluginLoaderV2>>>

// unified_registry.rs:18
static GLOBAL_REGISTRY: OnceLock<Arc<Mutex<UnifiedBoxRegistry>>>

// box_registry.rs:171
static GLOBAL_REGISTRY: Lazy<Arc<BoxFactoryRegistry>>

// plugin_loader_unified.rs:301
static GLOBAL_HOST: Lazy<Arc<RwLock<PluginHost>>>

// modules_registry.rs:9
static REGISTRY: Lazy<Mutex<HashMap<String, Box<dyn NyashBox>>>>

// type_meta.rs:118
static TYPE_META_REGISTRY: Lazy<Mutex<HashMap<String, Arc<TypeMeta>>>>

// host_handles.rs:20 (HandleRegistry内)
map: RwLock<HashMap<u64, Arc<dyn NyashBox>>>
```

**問題点**:
1. **状態管理の散在**: 7つの独立したグローバル変数
2. **初期化順序の不明確さ**: 依存関係がコード内に隠蔽
3. **テスト困難**: グローバル変数のリセットが不可能

### 箱化提案

#### **PluginSystemBox: 統一レジストリ**

```rust
// src/runtime/plugin_system_box.rs (新規 250行)
pub struct PluginSystemBox {
    // 統合されたすべての状態
    loader_v2: Arc<RwLock<PluginLoaderV2>>,
    unified_registry: Arc<Mutex<UnifiedBoxRegistry>>,
    box_factory_registry: Arc<BoxFactoryRegistry>,
    plugin_host: Arc<RwLock<PluginHost>>,
    modules_registry: Mutex<HashMap<String, Box<dyn NyashBox>>>,
    type_meta_registry: Mutex<HashMap<String, Arc<TypeMeta>>>,
    handle_registry: HandleRegistry,
}

impl PluginSystemBox {
    /// Create new plugin system (for testing)
    pub fn new() -> Self { /* ... */ }

    /// Get global instance (production)
    pub fn global() -> &'static PluginSystemBox { /* ... */ }

    /// Initialize all subsystems with correct order
    pub fn init_all(&self, config_path: &str) -> BidResult<()> {
        // 1. Load config
        // 2. Init unified registry
        // 3. Init box factory
        // 4. Load plugins
        // 5. Register type metadata
    }

    /// Shutdown all subsystems
    pub fn shutdown_all(&self) -> BidResult<()> { /* ... */ }

    // Delegation methods
    pub fn get_loader_v2(&self) -> Arc<RwLock<PluginLoaderV2>> { self.loader_v2.clone() }
    pub fn get_unified_registry(&self) -> Arc<Mutex<UnifiedBoxRegistry>> { /* ... */ }
    // ... 他のgetterも同様
}
```

**期待される効果**:
- **初期化順序の明確化**: `init_all()` で依存関係を明示
- **テスト容易性の劇的向上**: `PluginSystemBox::new()` でテスト用インスタンス作成可能
- **シャットダウンの確実性**: `shutdown_all()` で漏れなくクリーンアップ
- **状態の可視化**: すべてのプラグイン状態が1箇所に集約

**注意点**:
- 既存のグローバル変数を即座に削除せず、**ラッパー関数で段階的移行**:
  ```rust
  pub fn get_global_loader_v2() -> Arc<RwLock<PluginLoaderV2>> {
      PluginSystemBox::global().get_loader_v2()
  }
  ```

---

## 4️⃣ Instance Manager 解決ロジックの箱化

### 現状分析

**ファイル**: `src/runtime/plugin_loader_v2/enabled/instance_manager.rs` (376行)

**問題点**:
1. **create_box() の複雑さ** (line 11-237, 227行):
   - 3つの解決フォールバックパス（specs → config → file）
   - 7つの異なる invoke_fn 解決試行
   - 条件分岐が深すぎる（最大5段ネスト）

2. **重複する解決ロジック**:
   - `resolve_box_ids()` (250-348行) vs `resolve_box_ids_optional()` (350-376行)
   - 同じconfig/spec/file読み込みロジックが3箇所

### 箱化提案

#### **BoxMetadataResolverBox: 解決ロジック専用**

```rust
// src/runtime/plugin_loader_v2/enabled/metadata_resolver_box.rs (新規 200行)
pub struct BoxMetadataResolverBox {
    loader: Arc<PluginLoaderV2>,  // weak参照に変更してもよい
}

impl BoxMetadataResolverBox {
    /// Resolve all metadata for box creation
    pub fn resolve_creation_metadata(&self, box_type: &str)
        -> BidResult<BoxCreationMetadata>
    {
        // 統合された解決ロジック（3つのフォールバックパスを明示化）
        let meta = self.try_resolve_from_specs(box_type)
            .or_else(|| self.try_resolve_from_config(box_type))
            .or_else(|| self.try_resolve_from_file(box_type))
            .ok_or(BidError::InvalidType)?;

        Ok(meta)
    }

    fn try_resolve_from_specs(&self, box_type: &str) -> Option<BoxCreationMetadata>
    fn try_resolve_from_config(&self, box_type: &str) -> Option<BoxCreationMetadata>
    fn try_resolve_from_file(&self, box_type: &str) -> Option<BoxCreationMetadata>

    /// Resolve invoke function (7つの試行パスを統一)
    pub fn resolve_invoke_fn(&self, meta: &BoxCreationMetadata)
        -> Option<BoxInvokeFn>
    {
        self.try_direct_invoke_from_spec(meta)
            .or_else(|| self.try_invoke_from_typebox_ffi(meta))
            .or_else(|| self.try_invoke_by_type_id(meta))
            .or_else(|| self.try_invoke_from_deduced_lib(meta))
    }
}

/// Unified metadata for box creation
pub struct BoxCreationMetadata {
    pub box_type: String,
    pub lib_name: String,
    pub type_id: u32,
    pub birth_id: Option<u32>,
    pub fini_id: Option<u32>,
    pub invoke_fn: Option<BoxInvokeFn>,
}
```

**instance_manager.rs のリファクタリング**:
```rust
impl PluginLoaderV2 {
    pub fn create_box(&self, box_type: &str, args: &[Box<dyn NyashBox>])
        -> BidResult<Box<dyn NyashBox>>
    {
        // 1. Resolve metadata (委譲)
        let resolver = BoxMetadataResolverBox::new(self);
        let meta = resolver.resolve_creation_metadata(box_type)?;
        let invoke_fn = resolver.resolve_invoke_fn(&meta)
            .unwrap_or(nyash_plugin_invoke_v2_shim);

        // 2. Call birth (簡潔化)
        let instance_id = self.invoke_birth(&meta, invoke_fn, args)?;

        // 3. Construct PluginBoxV2 (現状維持)
        Ok(Box::new(PluginBoxV2 { /* ... */ }))
    }

    fn invoke_birth(&self, meta: &BoxCreationMetadata, invoke_fn: BoxInvokeFn, args: &[Box<dyn NyashBox>])
        -> BidResult<u32>
    {
        // birth呼び出しロジック（現状の line 156-221 を抽出）
    }
}
```

**期待される効果**:
- 376行 → 276行（100行削減、BoxMetadataResolverBox 200行は別ファイル）
- **解決ロジックの可視化**: フォールバックパスが明確に
- **テスト容易性**: 解決ロジックを独立してテスト可能
- **保守性**: 新しい解決パスの追加が容易

---

## 5️⃣ Method Resolver の箱化

### 現状分析

**ファイル**: `src/runtime/plugin_loader_v2/enabled/method_resolver.rs` (130行)

**問題点**:
- `resolve_method_id()` (line 11-70): 3つのフォールバックパス
- `resolve_method_handle()` (line 103-119): type_id/method_id/returns_result の複合解決
- 解決ロジックが散在（specs → config → file）

### 箱化提案

#### **MethodResolverBox: メソッド解決専用**

```rust
// src/runtime/plugin_loader_v2/enabled/method_resolver_box.rs (新規 150行)
pub struct MethodResolverBox {
    loader: Arc<PluginLoaderV2>,
}

impl MethodResolverBox {
    /// Unified method resolution
    pub fn resolve(&self, box_type: &str, method_name: &str)
        -> BidResult<MethodMetadata>
    {
        self.try_resolve_from_typebox_ffi(box_type, method_name)
            .or_else(|| self.try_resolve_from_specs(box_type, method_name))
            .or_else(|| self.try_resolve_from_config(box_type, method_name))
            .or_else(|| self.try_resolve_from_legacy_file(box_type, method_name))
            .ok_or(BidError::InvalidMethod)
    }

    fn try_resolve_from_typebox_ffi(&self, box_type: &str, method_name: &str) -> Option<MethodMetadata>
    fn try_resolve_from_specs(&self, box_type: &str, method_name: &str) -> Option<MethodMetadata>
    fn try_resolve_from_config(&self, box_type: &str, method_name: &str) -> Option<MethodMetadata>
    fn try_resolve_from_legacy_file(&self, box_type: &str, method_name: &str) -> Option<MethodMetadata>
}

pub struct MethodMetadata {
    pub method_id: u32,
    pub returns_result: bool,
    pub type_id: Option<u32>,
}
```

**期待される効果**:
- 解決パスの明確化（4つのフォールバックが一覧で見える）
- テスト容易性（各解決パスを独立してテスト）
- **-50行** 削減（重複ロジック統合）

---

## 6️⃣ FFI Bridge Final ABI 統合

### 現状分析

**ファイル**: `src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs` (509行)

**問題点**:
1. **2つのABI併存**:
   - Legacy TLV ABI (line 118-162)
   - Final ABI (line 78-116)
   - 同じ機能の重複実装

2. **encode/decode重複**:
   - `encode_args_final()` (line 317-371) vs `TlvCodecBox::encode_args()`
   - `decode_result_final()` (line 373-434) vs `decode_tlv_result()` (line 227-312)

### 箱化提案

#### **Option A: ABI Adapter Pattern（推奨）**

```rust
// src/runtime/plugin_loader_v2/enabled/abi_adapter_box.rs (新規 180行)
pub trait PluginAbiAdapter {
    fn encode_args(&self, args: &[Box<dyn NyashBox>]) -> EncodedArgs;
    fn decode_result(&self, data: &[u8]) -> BidResult<Option<Box<dyn NyashBox>>>;
}

pub struct TlvAbiAdapter;
impl PluginAbiAdapter for TlvAbiAdapter {
    // Legacy TLV実装
}

pub struct FinalAbiAdapter;
impl PluginAbiAdapter for FinalAbiAdapter {
    // Final ABI実装
}

// ffi_bridge.rs で使用
impl PluginLoaderV2 {
    pub fn invoke_instance_method(&self, ...) -> BidResult<...> {
        let adapter: Box<dyn PluginAbiAdapter> = if use_final_abi {
            Box::new(FinalAbiAdapter)
        } else {
            Box::new(TlvAbiAdapter)
        };

        let encoded = adapter.encode_args(args);
        // ... invoke ...
        let result = adapter.decode_result(&out);
    }
}
```

**期待される効果**:
- 509行 → 329行（-180行、Adapter 180行は別ファイル）
- ABI切り替えが明示的（環境変数 → Adapter選択）
- 将来の新ABIへの対応が容易

---

## 7️⃣ その他の箱化候補（中優先度）

### 7.1 PluginBoxV2 Type Helpers

**現状**: `src/runtime/plugin_loader_v2/enabled/types.rs` (314行)

**箱化候補**: `PluginHandleManagerBox`
- `get_or_create_handle()` (line 100-136): handle生成・キャッシュ管理
- `find_handle_by_instance()` (line 138-151): インスタンスIDからhandle検索
- `cache()` (line 61-70): handle登録

**効果**: -80行（重複ロジック統合）

### 7.2 Library Loader の箱化

**現状**: `src/runtime/plugin_loader_v2/enabled/loader/library.rs` (258行)

**箱化候補**: `PluginLibraryLoaderBox`
- 動的ライブラリロード・アンロード
- シンボル解決
- TypeBox FFI 登録

**効果**: -50行（ヘルパー関数統合）

---

## 🎯 実装優先順位

### Phase 1: 基盤整備（Week 1-2）

1. **TLV Codec 統一** (Priority: HIGH, Effort: 4-6h)
   - TlvCodecBox 拡張
   - host_api.rs マイグレーション
   - 期待: -30行、TLV変換の完全一元化

2. **Method Resolver 箱化** (Priority: MEDIUM, Effort: 6-8h)
   - MethodResolverBox 作成
   - 解決パス明確化
   - 期待: -50行、テスト容易性向上

### Phase 2: 主要リファクタリング（Week 3-4）

3. **HostHandle Router 分離** (Priority: HIGH, Effort: 12-16h)
   - 3箱分離（SelectorRouter/InstanceHandler/CollectionHandler）
   - host_api.rs 700行 → 500行
   - 期待: -200行、責任分離完全実現

4. **Instance Manager 解決ロジック箱化** (Priority: MEDIUM, Effort: 8-10h)
   - BoxMetadataResolverBox 作成
   - create_box() 簡潔化
   - 期待: -100行、フォールバックパス可視化

### Phase 3: 統合・最適化（Week 5-6）

5. **Plugin Registry 統一** (Priority: MEDIUM, Effort: 16-20h)
   - PluginSystemBox 作成
   - グローバル変数マイグレーション
   - 期待: 初期化順序明確化、テスト容易性劇的向上

6. **FFI Bridge ABI Adapter** (Priority: LOW, Effort: 10-12h)
   - Adapter Pattern実装
   - 2つのABI統合
   - 期待: -180行、将来のABI対応容易化

---

## 📈 期待される総合効果

| 項目 | Before | After | 改善率 |
|------|--------|-------|--------|
| **総行数** | 3,200行 | 2,000行 | **-37%** |
| **平均関数サイズ** | 120行 | 50行 | **-58%** |
| **責任明確性** | 😱 monolithic | 😊 single-responsibility | **10倍** |
| **テスト容易性** | 😱 グローバル依存 | 😊 独立テスト可能 | **10倍** |
| **デバッグ時間** | 😱 30分/issue | 😊 3分/issue | **10倍** |

---

## ⚠️ リスクと注意点

### 移行リスク

1. **既存コードへの影響**:
   - 11箇所のグローバル変数参照が存在
   - 段階的移行必須（ラッパー関数で互換性維持）

2. **テストカバレッジ不足**:
   - プラグインシステムの統合テストが少ない
   - リファクタリング前にテスト追加推奨

### 技術的負債の優先順位

**即座に対処すべき**:
- ✅ TLV Codec 統一（重複コード削除）
- ✅ HostHandle Router 分離（巨大関数分割）

**Phase 15.6で対処すべき**:
- ⚠️ Plugin Registry 統一（Everything is Plugin の基盤）
- ⚠️ Instance Manager 箱化（Box生成の統一化）

**将来対処可能**:
- 🔄 FFI Bridge ABI Adapter（新ABI対応時）
- 🔄 その他の箱化候補

---

## 📝 次のアクション

### 即座に実行可能（今日中）

1. **TLV Codec 統一**: TlvCodecBox 拡張（4-6時間）
2. **ホットスポット分析**: 実際の実行時間計測（1時間）

### 今週実行（Phase 15.6準備）

3. **HostHandle Router 設計レビュー**: Option A/B比較（2時間）
4. **テスト追加**: プラグインシステム統合テスト（4-6時間）

### 来週以降（Phase 15.6実装）

5. **HostHandle Router 実装**: 3箱分離（12-16時間）
6. **Plugin Registry 統一**: PluginSystemBox実装（16-20時間）

---

## 🎓 学び・パターン

### 成功パターン

1. **既に箱化されている例**:
   - ✅ `TlvCodecBox` (codec_box.rs): 140行、明確な責任
   - ✅ `HostHandleBox` (host_handle_box.rs): 単一目的

2. **箱化すべきサイン**:
   - 関数が100行超過
   - 複数の責任を持つ（routing + logic + error handling）
   - グローバル変数に依存
   - テストが困難

### 失敗を避けるパターン

1. **過度な箱化を避ける**:
   - 単純なヘルパー関数（30行未満）は箱化不要
   - 例: `plugin_ffi_common::encode::i64()` は適切にモジュール化済み

2. **段階的移行**:
   - グローバル変数の即座削除は禁止
   - ラッパー関数で互換性維持 → 移行完了後に削除

---

**結論**: プラグインシステムは**8つの主要な箱化候補**があり、総削減見込み**1,200-1,500行**、可読性・保守性**10倍改善**が期待される。Phase 1-3の段階的実装により、リスクを最小化しながら効果を最大化可能。
