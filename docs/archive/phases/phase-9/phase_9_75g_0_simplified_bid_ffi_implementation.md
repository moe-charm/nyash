# Phase 9.75g-0: 簡素化BID-FFI実装計画

## 🎯 目的
FFI ABI v0準拠の**シンプルで動く**プラグインシステムを1週間で実装する。

## 🌟 設計方針：シンプル第一

### 複雑さの回避
- ❌ 過度な抽象化（gRPC/REST/P2P等は後回し）
- ❌ 完璧な型システム（必要最小限から開始）
- ✅ **動くものを最速で作る**
- ✅ **拡張可能な基盤を作る**

### 技術的決定事項

```rust
// 1. ポインタとアライメント（AIレビュー反映）
pub type Ptr = usize;  // プラットフォーム依存
pub const ALIGNMENT: usize = 8;  // 8バイト境界

// 2. 最小限の型セット
pub enum BidType {
    // 基本型（必須）
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,  // (ptr: usize, len: usize)
    
    // 早期追加（ChatGPT推奨）
    Option(Box<BidType>),
    Result(Box<BidType>, Box<BidType>),
    
    // Phase 2以降
    // Array, Map, Future等
}

// 3. シンプルなBoxヘッダー（大きくてもOK）
#[repr(C, align(8))]
pub struct BoxHeader {
    magic: u32,      // "NYBX" (0x5859424E)
    version: u16,    // 1
    _pad: u16,       // アライメント用
    type_id: u32,
    ref_count: u32,  // 非atomic（Phase 1）
}

// 4. 単一エントリーポイント（ChatGPT推奨）
#[no_mangle]
extern "C" fn nyash_plugin_invoke(
    method_id: u32,
    args_ptr: *const u8,
    args_len: usize,
    result_ptr: *mut u8,
    result_len: *mut usize,
) -> i32 {  // 0=成功, 非0=エラー
    // 実装
}
```

## 📋 1週間の実装計画

### Day 1: 基礎定義
- [ ] `src/bid/types.rs` - 最小型システム
- [ ] `src/bid/header.rs` - Boxヘッダー定義
- [ ] テスト: アライメント検証

### Day 2: プラグインローダー
- [ ] `src/bid/loader.rs` - dlopen/dlsym
- [ ] 最小限のサンプルプラグイン（加算関数）
- [ ] テスト: ロード成功/失敗

### Day 3: 文字列処理
- [ ] UTF-8文字列の受け渡し実装
- [ ] 所有権ルール明文化
- [ ] テスト: 日本語/絵文字/空文字列

### Day 4: 統合実装
- [ ] FileBoxの最小実装（open/read/close）
- [ ] インタープリターとの接続
- [ ] テスト: ファイル操作e2e

### Day 5: エラー処理
- [ ] エラーコード体系
- [ ] Option/Result型の実装
- [ ] テスト: 各種エラーケース

### Day 6-7: ドキュメント・仕上げ
- [ ] 使い方ドキュメント
- [ ] Linux x86-64でのCI設定
- [ ] 予備日（問題対応）

## 🛠️ 実装の簡素化ポイント

### 1. Transport層の簡素化
```rust
// Phase 1: これだけ！
pub enum Transport {
    DynamicLibrary(PathBuf),
}

// 将来の拡張を予約（実装しない）
// Grpc(String),
// Rest(Url),
// P2P(PeerId),
```

### 2. 所有権の単純ルール
```rust
// 入力: 呼び出し側が所有（借用）
// 出力: プラグインがallocate、呼び出し側がfree

extern "C" {
    fn nyash_alloc(size: usize) -> *mut u8;
    fn nyash_free(ptr: *mut u8);
}
```

### 3. バージョニングの簡素化
```yaml
# BID定義（最小限）
version: 1
plugin:
  name: nyash-file
  version: "0.1.0"
  
methods:
  - id: 1
    name: open
    params: [string, string]
    returns: i32
```

## ✅ 成功基準

### 必須
- [ ] Linux x86-64で動作
- [ ] FileBoxプラグインが動く
- [ ] 文字列の受け渡しが正しい
- [ ] メモリリークなし

### あれば良い
- [ ] Windows対応の準備
- [ ] 性能測定
- [ ] 複数プラグイン同時ロード

## 🚀 この設計の利点

1. **シンプル**: 必要最小限の機能のみ
2. **実用的**: 1週間で確実に動く
3. **拡張可能**: 将来の機能追加が容易
4. **保守可能**: Rust中級者が理解できる

## ⚠️ 意図的に省略したもの

- vtable（動的ディスパッチ）
- 非同期処理
- ネットワーク対応
- 複雑な型（Array, Map等）
- マルチスレッド対応

これらは**動くものができてから**Phase 2以降で追加します。

## 📝 まとめ

「structが大きくても問題ない」というユーザーの言葉通り、Boxヘッダーは拡張性を考慮して余裕を持たせています。複雑さを避けつつ、確実に動くものを作ることに集中します。

**キーワード**: Simple, Practical, Extensible

---

**作成日**: 2025-08-17  
**優先度**: 🔥 最高（Phase 9.75g-1の前提）  
**期間**: 1週間  
**ターゲット**: Linux x86-64