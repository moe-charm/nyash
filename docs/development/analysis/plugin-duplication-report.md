# プラグイン重複コード洗い出しレポート

**調査日**: 2025-10-11
**調査範囲**: 全プラグイン（19個）、特に以下を重点調査:
- `plugins/nyash-*-plugin/src/lib.rs` (全プラグインエントリーポイント)
- `plugins/*/src/tlv*.rs` (TLV関連ヘルパー)
- 既存の共通化基盤: `crates/hako_abi_impl/src/tlv.rs` (306行)

---

## 📊 **エグゼクティブサマリー**

| 項目 | 現状 | 統合候補 | 削減見込み |
|------|------|---------|-----------|
| **TLV Builder重複** | 4プラグイン | → hako_abi_impl::tlv | **~180行** |
| **TLV Parser重複** | 3プラグイン | → hako_abi_impl::tlv | **~200行** |
| **Instance管理パターン** | 15プラグイン | マクロ or trait | **~150行** |
| **プラグイン固有TLV** | 3プラグイン | 保持（統合不可） | 0行 |
| **総削減見込み** | - | - | **~530行** |

**重要**: 既に Phase 3.1-3.3 で **-972行** 削減済み。このレポートは**残存する重複**のみを対象。

---

## 1️⃣ **TLV Builder 重複（統合優先度: 高）**

### 📍 **重複箇所**
4つのプラグインで類似の TLV builder 関数が重複:

| プラグイン | ファイル | 関数 | 行数 | 重複度 |
|-----------|---------|------|------|--------|
| **map-plugin** | `src/tlv_codec.rs` | `build_tlv_i64_string()` | 18行 | ⭐⭐⭐ |
| **map-plugin** | `src/tlv_codec.rs` | `build_tlv_i64_i64()` | 16行 | ⭐⭐⭐ |
| **map-plugin** | `src/tlv_codec.rs` | `build_tlv_i64_handle()` | 17行 | ⭐⭐⭐ |
| **map-plugin** | `src/tlv_codec.rs` | `build_tlv_i64_host_handle()` | 17行 | ⭐⭐⭐ |
| **map-plugin** | `src/tlv_codec.rs` | `write_mapval_tlv()` | 7行 | ⭐ (固有) |
| **filebox-plugin** | `src/tlv_helpers.rs` | `write_tlv_result()` | 24行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `write_tlv_void()` | 3行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `write_tlv_bytes()` | 3行 | ⭐⭐ (tag違い) |
| **filebox-plugin** | `src/tlv_helpers.rs` | `write_tlv_i32()` | 3行 | ⭐⭐ (i32固有) |
| **json-plugin** | `src/tlv_helpers.rs` | `write_tlv_result()` | 24行 | ⭐⭐⭐ |
| **json-plugin** | `src/tlv_helpers.rs` | `write_tlv_void()` | 4行 | ⭐⭐⭐ |
| **json-plugin** | `src/tlv_helpers.rs` | `write_u32()` | 14行 | ⭐ (json固有) |
| **net-plugin** | `src/tlv.rs` | `write_tlv_result()` | 18行 | ⭐⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `write_tlv_void()` | 2行 | ⭐⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `write_tlv_bytes()` | 2行 | ⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `write_tlv_i32()` | 2行 | ⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `write_u32()` | 9行 | ⭐ (net固有) |

### 📐 **重複パターン詳細**

#### Pattern A: `write_tlv_result()` - 汎用TLV構築器
```rust
// 3つのプラグインで完全に同じ実装（微細な差異のみ）
pub fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    let mut buf = Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes()); // argc
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    // ... copy to result
}
```
**出現箇所**: filebox-plugin/tlv_helpers.rs:11-35, json-plugin/tlv_helpers.rs:11-35, net-plugin/tlv.rs:44-62
**行数**: 24行 × 3箇所 = 72行
**統合先**: `hako_abi_impl::tlv::build_tlv_multi()`

#### Pattern B: `write_tlv_void()` - void返却
```rust
// 3つのプラグインで同一
pub fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(9u8, &[])], result, result_len)
}
```
**出現箇所**: filebox-plugin:34-36, json-plugin:53-56, net-plugin:64-66
**行数**: 3行 × 3箇所 = 9行
**統合先**: `hako_abi_impl::tlv::write_tlv_void()`

#### Pattern C: `build_tlv_i64_*()` - 2引数TLV構築 (map-plugin固有)
```rust
// map-plugin の Stage-2 専用（Array操作のホスト呼び出し用）
pub fn build_tlv_i64_string(idx: i64, s: &str) -> Vec<u8> { /* 18行 */ }
pub fn build_tlv_i64_i64(idx: i64, value: i64) -> Vec<u8> { /* 16行 */ }
pub fn build_tlv_i64_handle(idx: i64, type_id: u32, instance_id: u32) -> Vec<u8> { /* 17行 */ }
pub fn build_tlv_i64_host_handle(idx: i64, handle: u64) -> Vec<u8> { /* 17行 */ }
```
**出現箇所**: map-plugin/tlv_codec.rs:62-137
**行数**: 68行（4関数合計）
**評価**: **高度にパターン化**。汎用ビルダー `build_tlv_args([Arg1, Arg2])` に置き換え可能

### 📊 **統合案**

#### Option A: `hako_abi_impl::tlv` に汎用ビルダー追加
```rust
// crates/hako_abi_impl/src/tlv.rs に追加

/// Build TLV with multiple arguments (generic builder)
pub fn build_tlv_multi(args: &[TlvArg]) -> Vec<u8> {
    // header: version=1, argc=args.len()
    // for each arg: encode as appropriate tag
}

pub enum TlvArg<'a> {
    I64(i64),
    I32(i32),
    String(&'a str),
    Bytes(&'a [u8]),
    Handle(u32, u32),
    HostHandle(u64),
}

/// Build TLV with generic result payload
pub fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32;

/// Write void/empty TLV
pub fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32;
```

#### Option B: プラグイン固有ヘルパーは保持
以下は**統合しない**（プラグイン固有のロジック含む）:
- `write_mapval_tlv()` (map-plugin) - MapVal型依存
- `write_tlv_bytes()` / `write_tlv_i32()` - tag定数がプラグイン固有
- `write_u32()` (json/net) - TLV非依存のraw書き込み

### 💰 **削減見込み**

| 関数 | 重複箇所 | 1箇所あたり | 削減合計 |
|------|---------|------------|---------|
| `write_tlv_result()` | 3箇所 | 24行 | **72行** |
| `write_tlv_void()` | 3箇所 | 3行 | **9行** |
| `build_tlv_i64_*()` 系 | 4関数 | 17行平均 | **68行** (汎用化で置換) |
| `write_tlv_bytes/i32()` | 6箇所 | 3行 | 18行 (統合可能だが優先度低) |
| **合計** | - | - | **~180行** |

**注**: 既に Phase 3.3 で `write_tlv_handle/i64/string/bool` は統合済み（`hako_abi_impl::tlv` からre-export）

---

## 2️⃣ **TLV Parser 重複（統合優先度: 中）**

### 📍 **重複箇所**
3つのプラグインで類似の TLV parser 関数が重複:

| プラグイン | ファイル | 関数 | 行数 | 重複度 |
|-----------|---------|------|------|--------|
| **filebox-plugin** | `src/tlv_helpers.rs` | `tlv_parse_header()` | 11行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `tlv_parse_string_at()` | 16行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `tlv_parse_handle_at()` | 22行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `tlv_parse_bytes_at()` | 16行 | ⭐⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | `tlv_parse_two_strings()` | 9行 | ⭐⭐ |
| **filebox-plugin** | `src/tlv_helpers.rs` | その他6関数 | 60行 | ⭐ (高レベル) |
| **net-plugin** | `src/tlv.rs` | `tlv_parse_header()` | 11行 | ⭐⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `tlv_parse_entry_hdr()` | 11行 | ⭐⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `tlv_parse_string()` | 12行 | ⭐⭐ |
| **net-plugin** | `src/tlv.rs` | `tlv_parse_two_strings()` | 21行 | ⭐⭐ |
| **net-plugin** | `src/tlv.rs` | その他5関数 | 50行 | ⭐ |
| **egui-plugin** | `src/lib.rs` | `tlv_parse_header()` | 11行 | ⭐⭐⭐ |

### 📐 **重複パターン詳細**

#### Pattern A: `tlv_parse_header()` - TLVヘッダー解析
```rust
// 3つのプラグインで完全同一
pub fn tlv_parse_header(data: &[u8]) -> Result<(u16, u16, usize), ()> {
    if data.len() < 4 { return Err(()); }
    let ver = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 { return Err(()); }
    Ok((ver, argc, 4))
}
```
**出現箇所**: filebox/tlv_helpers.rs:63-73, net/tlv.rs:74-84, egui/lib.rs (同等実装)
**行数**: 11行 × 3箇所 = 33行
**統合先**: `hako_abi_impl::tlv::parse_header()`

#### Pattern B: `tlv_parse_*_at()` - 位置指定パース
```rust
// filebox と net で類似実装（net は tlv_parse_entry_hdr に統合）
pub fn tlv_parse_string_at(data: &[u8], pos: &mut usize) -> Result<String, ()> { /* 16行 */ }
pub fn tlv_parse_handle_at(data: &[u8], pos: &mut usize) -> Result<(u32, u32), ()> { /* 22行 */ }
pub fn tlv_parse_bytes_at(data: &[u8], pos: &mut usize) -> Result<Vec<u8>, ()> { /* 16行 */ }
```
**出現箇所**: filebox/tlv_helpers.rs:85-143, net/tlv.rs に類似実装
**行数**: 54行（filebox）+ 相当分（net）= ~80行
**統合先**: `hako_abi_impl::tlv::parse_arg_at()` (汎用関数)

### 📊 **統合案**

#### Option A: `hako_abi_impl::tlv` に汎用パーサー追加
```rust
// crates/hako_abi_impl/src/tlv.rs に追加

/// Parse TLV header (version, argc, next position)
pub fn parse_header(data: &[u8]) -> Result<(u16, u16, usize), ()>;

/// Parse entry header at position (tag, size, payload_start)
pub fn parse_entry_hdr(data: &[u8], pos: usize) -> Result<(u8, usize, usize), ()>;

/// Parse argument at position with expected tag
pub fn parse_arg_at<T>(data: &[u8], pos: &mut usize, expected_tag: u8) -> Result<T, ()>
where
    T: TlvParseable;

pub trait TlvParseable {
    fn parse_from_bytes(data: &[u8], tag: u8, size: usize) -> Result<Self, ()> where Self: Sized;
}

// 実装: i64, String, Vec<u8>, (u32, u32) (handle), u64 (host_handle)
```

#### Option B: 高レベルパーサーは各プラグインで維持
以下は**統合しない**（プラグイン固有の組み合わせロジック）:
- `tlv_parse_two_strings()` - 組み合わせパターン多数
- `tlv_parse_optional_string_and_bytes()` - 複雑な分岐
- `tlv_parse_string()` / `tlv_parse_handle()` - 1引数特化版（便利関数）

### 💰 **削減見込み**

| 関数 | 重複箇所 | 1箇所あたり | 削減合計 |
|------|---------|------------|---------|
| `tlv_parse_header()` | 3箇所 | 11行 | **33行** |
| `tlv_parse_*_at()` 系 | 2プラグイン | 40行平均 | **80行** (汎用化で置換) |
| `tlv_parse_entry_hdr()` | 2箇所 | 11行 | **22行** |
| 高レベルパーサー | 複数 | 可変 | 50-80行 (低優先度) |
| **合計** | - | - | **~200行** |

**注**: `read_arg_i64/string/handle` は既に `hako_abi_impl::tlv` で提供済み。`tlv_parse_*` との重複整理が必要。

---

## 3️⃣ **Instance 管理パターン重複（統合優先度: 中）**

### 📍 **重複箇所**
15個のプラグインで完全に同じパターン:

```rust
// 全プラグインで同一構造（型名とフィールドだけ違う）
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{atomic::AtomicU32, Mutex};

static INSTANCES: Lazy<Mutex<HashMap<u32, XxxInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);
```

**出現プラグイン（15個）**:
- array, console, counter, fixture, map (INSTANCES + INSTANCE_COUNTER)
- egui, encoding, integer, path, regex, string, toml (INST + NEXT_ID)
- filebox (state.rs に分離、with_instance パターン)
- net (server_impl.rs のみ)
- nobirth (INSTANCES のみ、COUNTER なし)

### 📐 **重複パターン詳細**

#### Pattern A: 標準的な Instance 管理
```rust
// 51箇所のロック取得パターン (8プラグイン × 平均6-7箇所)
let map = match INSTANCES.lock() {
    Ok(m) => m,
    Err(_) => return NYB_E_INVALID_HANDLE, // or E_PLUGIN
};

// バリエーション:
// 1. 不変参照 (get)
let map = INSTANCES.lock().map_err(|_| E_PLUGIN)?;
if let Some(inst) = map.get(&instance_id) { /* use inst */ }

// 2. 可変参照 (insert/remove)
let mut map = INSTANCES.lock().map_err(|_| E_PLUGIN)?;
map.insert(id, Instance { ... });

// 3. スコープ分離（デッドロック回避、array-plugin/map-plugin）
let data = {
    let map = INSTANCES.lock()?;
    map.get(&id)?.data.clone()
}; // ← ロック解放
let mut map = INSTANCES.lock()?; // ← 別ロック
```

**出現箇所**: 51箇所（全体）
**行数**: 平均5行 × 51箇所 = 255行
**問題点**: エラーハンドリングの不統一、デッドロックリスク

#### Pattern B: `with_instance` パターン（filebox-plugin）
```rust
// filebox-plugin/state.rs に実装
pub fn with_instance<F, R>(id: u32, f: F) -> Result<R, &'static str>
where
    F: FnOnce(&FileBoxInstance) -> R,
{
    match INSTANCES.lock() {
        Ok(map) => match map.get(&id) {
            Some(instance) => Ok(f(instance)),
            None => Err("instance not found"),
        },
        Err(_) => Err("lock error"),
    }
}

pub fn with_instance_mut<F, R>(id: u32, f: F) -> Result<R, &'static str>
where
    F: FnOnce(&mut FileBoxInstance) -> R,
{ /* 同様 */ }
```
**出現箇所**: filebox-plugin のみ
**行数**: 30行（ヘルパー関数）
**評価**: **良パターン**、他プラグインにも適用可能

### 📊 **統合案**

#### Option A: マクロ化（最小侵襲）
```rust
// crates/hako_abi_impl/src/instance_manager.rs

/// Instance storage with lock management
#[macro_export]
macro_rules! define_instance_storage {
    ($inst_type:ty) => {
        use once_cell::sync::Lazy;
        use std::collections::HashMap;
        use std::sync::{atomic::{AtomicU32, Ordering}, Mutex};

        static INSTANCES: Lazy<Mutex<HashMap<u32, $inst_type>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));
        static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);
    };
}

/// Safe instance access with automatic error handling
#[macro_export]
macro_rules! with_instance {
    ($id:expr, $instances:expr, $f:expr) => {
        match $instances.lock() {
            Ok(map) => match map.get(&$id) {
                Some(inst) => Ok($f(inst)),
                None => Err(HAKO_E_INVALID_HANDLE),
            },
            Err(_) => Err(HAKO_E_PLUGIN_ERROR),
        }
    };
}

#[macro_export]
macro_rules! with_instance_mut {
    ($id:expr, $instances:expr, $f:expr) => { /* 同様 */ };
}
```

**使用例**:
```rust
// プラグイン側
use hako_abi_impl::{define_instance_storage, with_instance};

struct ArrayInstance { data: Vec<ArrayValue> }
define_instance_storage!(ArrayInstance);

// 使用
with_instance!(instance_id, INSTANCES, |inst| {
    // inst を安全に使用
    inst.data.len()
})
```

#### Option B: Trait ベース（抽象度高）
```rust
// crates/hako_abi_impl/src/instance_manager.rs

pub trait InstanceManager: Sized {
    fn create(init_data: Self::InitData) -> u32;
    fn destroy(id: u32);
    fn with<F, R>(id: u32, f: F) -> Result<R, i32> where F: FnOnce(&Self) -> R;
    fn with_mut<F, R>(id: u32, f: F) -> Result<R, i32> where F: FnOnce(&mut Self) -> R;
}

// プラグイン側で derive 可能にする
#[derive(InstanceManager)]
struct ArrayInstance { /* ... */ }
```

**評価**: 魅力的だが実装コスト高（derive macro 必要）

### 💰 **削減見込み**

| パターン | 重複箇所 | 削減行数 |
|---------|---------|---------|
| INSTANCES 定義 | 15プラグイン × 3行 | **45行** → マクロ1行 |
| lock() エラーハンドリング | 51箇所 × 5行 | **255行** → マクロ呼び出し1-2行 |
| with_instance パターン | 1実装 | 既存30行を標準化 |
| **合計** | - | **~150行** (純削減) + 可読性大幅向上 |

**副次的効果**:
- デッドロックリスクの体系的管理（スコープ分離パターンの標準化）
- エラーハンドリングの統一（17箇所の `Err(_) => return NYB_E_*` を一元化）

---

## 4️⃣ **プラグイン固有 TLV 関数（統合不可）**

以下は**統合しない**（プラグイン固有の型・ロジック依存）:

### 📍 **統合不可な関数リスト**

| プラグイン | 関数 | 理由 | 行数 |
|-----------|------|------|------|
| **map-plugin** | `write_mapval_tlv()` | MapVal 型依存（enum専用） | 7行 |
| **map-plugin** | `escape_json()` | JSON固有（map stringify用） | 14行 |
| **map-plugin** | `v_to_string()` | MapVal debug用 | 7行 |
| **filebox-plugin** | `write_tlv_bytes()` | TLV_TAG_BYTES (5) 固有 | 3行 |
| **filebox-plugin** | `write_tlv_i32()` | TLV_TAG_I32 (2) 固有 | 3行 |
| **filebox-plugin** | 複雑パーサー 6個 | 組み合わせパターン多数 | 90行 |
| **json-plugin** | `write_u32()` | raw 4-byte (TLV非依存) | 14行 |
| **net-plugin** | `write_u32()` | raw 4-byte (TLV非依存) | 9行 |
| **net-plugin** | `tlv_parse_i32()` | i32/i64 両対応（net固有） | 20行 |
| **net-plugin** | `ensure_result_capacity()` | エラーコード固有 | 11行 |

**合計**: ~180行（保持）

**評価**: これらは**統合する必要なし**。プラグインの責務範囲内。

---

## 📈 **総削減見込みサマリー**

| カテゴリ | 削減見込み | 実装コスト | 優先度 |
|---------|-----------|-----------|--------|
| **1. TLV Builder重複** | **~180行** | 4-6時間 | ⭐⭐⭐ |
| **2. TLV Parser重複** | **~200行** | 6-8時間 | ⭐⭐ |
| **3. Instance管理パターン** | **~150行** | 2-4時間 (マクロ) | ⭐⭐⭐ |
| **プラグイン固有TLV** | 0行（統合不可） | - | - |
| **総計** | **~530行** | **12-18時間** | - |

**追加効果**:
- デッドロック防止の体系的管理
- エラーハンドリング統一（保守性向上）
- 新規プラグイン開発の簡易化

---

## 🎯 **実装推奨アプローチ**

### Phase 1: 高優先度（4-6時間）
1. **TLV Builder 統合** (Priority 1)
   - `build_tlv_multi()` / `write_tlv_result()` / `write_tlv_void()` を `hako_abi_impl::tlv` に追加
   - map-plugin の 4つの `build_tlv_i64_*()` を汎用化
   - 影響プラグイン: map, filebox, json, net (4個)
   - 削減: **~180行**

2. **Instance 管理マクロ** (Priority 1)
   - `define_instance_storage!` / `with_instance!` / `with_instance_mut!` マクロ作成
   - 影響プラグイン: 15個（全Instance管理プラグイン）
   - 削減: **~150行** + 可読性大幅向上

### Phase 2: 中優先度（6-8時間）
3. **TLV Parser 統合** (Priority 2)
   - `parse_header()` / `parse_entry_hdr()` / `parse_arg_at<T>()` を追加
   - 影響プラグイン: filebox, net, egui (3個)
   - 削減: **~200行**

### Phase 3: オプション（評価後判断）
4. **高レベルパーサーの標準化** (Priority 3)
   - `parse_two_strings()` / `parse_optional_*()` 等の統一
   - プラグイン固有性が高いため、無理に統合しない方が良い可能性

---

## ⚠️ **注意事項・リスク**

### 1. 既存の統合状況
**Phase 3.1-3.3 で既に完了**:
- ✅ `write_tlv_handle/i64/string/bool/host_handle` - hako_abi_impl::tlv に統合済み
- ✅ `read_arg_i64/string/handle/host_handle` - 同上
- ✅ TLV tag 定数 - hako_abi に統合済み

**このレポートの対象**:
- 🔄 `write_tlv_result()` / `build_tlv_*()` - **未統合** (Builder系)
- 🔄 `tlv_parse_*()` - **未統合** (Parser系)
- 🔄 Instance 管理パターン - **未統合**

### 2. 後方互換性
- 既存プラグインの動作に影響なし（追加のみ）
- 段階的移行可能（プラグイン単位で切り替え）

### 3. テスト戦略
- 各Phase後に全プラグインの統合テスト実行
- 既存のスモークテスト（251/269 PASS）で回帰検証

---

## 📚 **関連ドキュメント**

- **Phase 3.1-3.3 実装記録**: [CLAUDE.md](../../../CLAUDE.md#-phase-33完了-array-plugin-デッドロック修正--filebox-tlv共通化-2025-10-11)
- **Hakorune Shared ABI 設計**: [hakorune-shared-abi-architecture.md](../proposals/hakorune-shared-abi-architecture.md)
- **hako_abi_impl 実装**: [crates/hako_abi_impl/src/tlv.rs](../../../crates/hako_abi_impl/src/tlv.rs)
- **既存TLV実装例**:
  - map-plugin: [plugins/nyash-map-plugin/src/tlv_codec.rs](../../../plugins/nyash-map-plugin/src/tlv_codec.rs)
  - filebox-plugin: [plugins/nyash-filebox-plugin/src/tlv_helpers.rs](../../../plugins/nyash-filebox-plugin/src/tlv_helpers.rs)
  - net-plugin: [plugins/nyash-net-plugin/src/tlv.rs](../../../plugins/nyash-net-plugin/src/tlv.rs)

---

**調査完了**: 2025-10-11
**次のアクション**: Phase 1 実装判断（ユーザー確認後）
