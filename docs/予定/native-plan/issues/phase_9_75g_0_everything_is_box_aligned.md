# Phase 9.75g-0 修正版: Everything is Box哲学準拠のシンプル設計

## 🎯 基本方針：Nyash哲学ファースト

**重要な気づき**: ChatGPT設計は一般的だが、**既存のFutureBox等と二重実装**になってしまう。
Nyashの「Everything is Box」哲学を貫き、既存資産を活用する。

## 🌟 Nyash哲学に準拠した設計

### 1. 型システム：プリミティブ + Handle

```rust
// src/bid/types.rs - Everything is Box哲学準拠

#[derive(Clone, Debug, PartialEq)]
pub enum BidType {
    // === プリミティブ型（FFI境界で直接渡せる） ===
    Bool,           // Nyashのbool literal
    I32,            // 32ビット整数
    I64,            // Nyashの標準整数
    F32,            // 32ビット浮動小数点
    F64,            // Nyashの標準浮動小数点
    String,         // UTF-8文字列 (ptr: usize, len: usize)
    Bytes,          // バイナリデータ (ptr: usize, len: usize)
    
    // === Everything is Box: すべてのBoxは統一Handle ===
    Handle(String), // "StringBox:123", "FileBox:456", "FutureBox:789"
    
    // === メタ型（FFI用） ===
    Void,           // 戻り値なし
    
    // Phase 2以降で追加（定義だけ先に）
    Option(Box<BidType>),         // Option<T>
    Result(Box<BidType>, Box<BidType>), // Result<T, E>
    Array(Box<BidType>),          // Array<T>
    
    // === Everything is Box哲学の拡張 ===
    // Array, Map, Future等はすべてHandle("ArrayBox:id")として扱う
}

// Nyashの既存Boxとの対応表
/*
Handle("StringBox:123")   → StringBox インスタンス
Handle("IntegerBox:456")  → IntegerBox インスタンス  
Handle("FutureBox:789")   → FutureBox インスタンス（非同期）
Handle("FileBox:101")     → FileBox インスタンス
Handle("ArrayBox:102")    → ArrayBox インスタンス
Handle("P2PBox:103")      → P2PBox インスタンス
*/
```

### 2. シンプルなBoxヘッダー（Nyash統一仕様）

```rust
// 既存のNyash Boxヘッダーと統一
#[repr(C, align(8))]
pub struct BoxHeader {
    magic: u32,         // "NYBX" (0x5859424E)
    version: u16,       // 1
    _pad: u16,          // アライメント用
    type_id: u32,       // BoxTypeId（StringBox=1, FileBox=2等）
    instance_id: u32,   // インスタンス識別子
    ref_count: u32,     // 非atomic（Nyashはシングルスレッド中心）
    flags: u32,         // 将来の拡張用
}

// Nyashの既存Box型との統合
pub const NYASH_BOX_TYPES: &[(u32, &str)] = &[
    (1, "StringBox"),
    (2, "IntegerBox"), 
    (3, "BoolBox"),
    (4, "ArrayBox"),
    (5, "MapBox"),
    (6, "FileBox"),     // プラグインで提供
    (7, "FutureBox"),   // 既存の非同期Box
    (8, "P2PBox"),      // 既存のP2P Box
    // 新しいプラグインBoxも同じ仕組みで追加
];
```

### 3. 単一エントリーポイント（Everything is Box対応）

```rust
// Nyashの全Boxを統一的に扱える設計
#[no_mangle]
extern "C" fn nyash_plugin_invoke(
    box_type_id: u32,       // どのBox型？（StringBox=1, FileBox=6等）
    method_id: u32,         // どのメソッド？（open=1, read=2等）
    instance_id: u32,       // どのインスタンス？（Handle解析用）
    args_ptr: *const u8,    // 引数データ
    args_len: usize,        // 引数サイズ
    result_ptr: *mut u8,    // 結果置き場
    result_len: *mut usize, // 結果サイズ
) -> i32 {  // 0=成功, 非0=エラー
    // Everything is Box哲学：すべて同じ仕組みで処理
}

// 既存のNyash Boxとの統合例
fn handle_stringbox_call(method_id: u32, instance_id: u32, args: &[u8]) -> Result<Vec<u8>, BidError> {
    match method_id {
        1 => { /* length() */ },
        2 => { /* substring() */ },
        3 => { /* append() */ },
        _ => Err(BidError::MethodNotFound),
    }
}

fn handle_filebox_call(method_id: u32, instance_id: u32, args: &[u8]) -> Result<Vec<u8>, BidError> {
    match method_id {
        1 => { /* open() */ },
        2 => { /* read() */ },
        3 => { /* write() */ },
        4 => { /* close() */ },
        _ => Err(BidError::MethodNotFound),
    }
}
```

### 4. 既存Nyash Boxとの統合戦略

```rust
// src/bid/integration.rs - 既存Boxとの橋渡し

pub struct NyashBoxRegistry {
    // 既存のStringBox、ArrayBox等のインスタンス管理
    instances: HashMap<u32, Arc<RwLock<dyn NyashBox>>>,
    next_id: AtomicU32,
}

impl NyashBoxRegistry {
    pub fn register_box(&self, box_instance: Arc<RwLock<dyn NyashBox>>) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.instances.insert(id, box_instance);
        id
    }
    
    pub fn call_method(&self, type_id: u32, instance_id: u32, method_id: u32, args: &[u8]) 
        -> Result<Vec<u8>, BidError> 
    {
        match type_id {
            1 => self.call_stringbox_method(instance_id, method_id, args),
            6 => self.call_filebox_method(instance_id, method_id, args),
            7 => self.call_futurebox_method(instance_id, method_id, args), // 既存FutureBox活用！
            _ => Err(BidError::UnknownBoxType(type_id)),
        }
    }
}
```

## 📋 修正された実装計画

### Day 1: Nyash統合基盤
- [ ] `src/bid/types.rs` - Everything is Box準拠型定義
- [ ] `src/bid/registry.rs` - 既存Box統合レジストリ
- [ ] `src/bid/header.rs` - 統一Boxヘッダー
- [ ] テスト: 既存StringBoxとの統合

### Day 2: プラグインローダー
- [ ] `src/bid/loader.rs` - dlopen/dlsym
- [ ] 最小プラグイン（MathBox拡張）
- [ ] テスト: 既存BoxとプラグインBoxの共存

### Day 3: Handle型統合
- [ ] Handle("FileBox:123")の解決機構
- [ ] プリミティブ⇔Handle変換
- [ ] テスト: 全Box型の統一的操作

### Day 4: FileBox実装
- [ ] FileBoxプラグインの完全実装
- [ ] 既存のNyashコードとの互換性
- [ ] テスト: FileBox e2e動作

### Day 5: エラー処理とOption/Result
- [ ] 統一エラーシステム
- [ ] Option/Result型の最小実装
- [ ] テスト: エラーケース網羅

### Day 6-7: 統合テスト・ドキュメント
- [ ] 既存インタープリターとの統合
- [ ] 使用例とドキュメント
- [ ] Linux x86-64 CI設定

## 🌟 この設計の哲学的利点

### 1. Everything is Box哲学の完全準拠
```nyash
// Nyashコード側：変わらない！
local file = new FileBox("test.txt", "r")    // プラグイン提供
local future = new FutureBox()               // 既存Box
local array = new ArrayBox()                 // 既存Box

// すべて同じHandle("BoxType:id")として扱われる
```

### 2. 既存資産の完全活用
- ❌ 新しいBidFuture実装 → ✅ 既存FutureBox活用
- ❌ 新しい型システム → ✅ 既存Nyash型との統合
- ❌ 二重実装 → ✅ 単一の統一システム

### 3. スレッド最小設計
```rust
// Nyashの現実に合わせた設計
ref_count: u32,  // 非atomic（シングルスレッド中心）

// Phase 2以降でatomic対応を検討
#[cfg(feature = "atomic")]
ref_count: AtomicU32,
```

### 4. エラー対策の強化
```rust
// 統一エラー処理でプラグインの安定性向上
pub enum BidError {
    UnknownBoxType(u32),
    InstanceNotFound(u32),
    MethodNotFound(u32),
    InvalidArguments(String),
    PluginError(String),     // プラグイン側エラーを安全に伝播
}
```

## ✅ 成功基準（Everything is Box準拠）

### 必須
- [ ] 既存StringBox、ArrayBoxとプラグインFileBoxが同じ仕組みで動作
- [ ] Handle("FileBox:123")でのBox操作
- [ ] 既存FutureBoxの活用（新実装なし）
- [ ] すべてのBoxが統一的にアクセス可能

### 理想
- [ ] 新しいプラグインBoxも既存Boxと見分けがつかない
- [ ] Nyashコード側は変更不要
- [ ] Everything is Box哲学の技術的実現

## 📝 まとめ

**ChatGPT先生の一般論は正しいが、Nyashの独特な哲学には合わない。**

**Nyash Way**: Everything is Box → すべてHandle + 既存Box活用  
**一般的Way**: 型システム分離 → 新しい実装追加

**結論**: Nyashの哲学を貫いて、既存の資産を最大活用する設計で進む！

---

**修正日**: 2025-08-17  
**修正理由**: Everything is Box哲学の完全準拠、既存資産活用  
**キーワード**: Simple, Nyash-native, No-duplication