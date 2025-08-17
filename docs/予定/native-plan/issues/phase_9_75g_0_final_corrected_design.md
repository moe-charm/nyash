# Phase 9.75g-0 最終修正版: ChatGPT先生の知恵を反映した型設計

## 🎯 ChatGPT先生の明確な判断

> **結論**: Future/StreamはBidType（値型）に含めないでください。非同期性は「実行モデル」であって「値の表現」ではありません。

## 🛠️ 修正された型システム設計

### 1. 値型（BidType）- 純粋な値のみ

```rust
// src/bid/types.rs - ChatGPT先生推奨の清潔な設計

#[derive(Clone, Debug, PartialEq)]
pub enum BidType {
    // === 基本型（Phase 1で実装） ===
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,     // (ptr: usize, len: usize)
    Bytes,      // (ptr: usize, len: usize)
    
    // === 複合型（Phase 2で実装） ===
    Array(Box<BidType>),           // 配列
    List(Box<BidType>),           // 可変長リスト
    Map(Box<BidType>, Box<BidType>), // キーバリューマップ
    Tuple(Vec<BidType>),          // タプル
    Record(Vec<(String, BidType)>), // 名前付きフィールド
    Variant(Vec<(String, Option<BidType>)>), // 列挙型
    
    // === 特殊型（Phase 2で実装） ===
    Option(Box<BidType>),         // null許容
    Result(Box<BidType>, Box<BidType>), // エラー型
    Handle(String),               // 不透明ハンドル（同期リソース用）
    Void,                        // 戻り値なし
    
    // === 拡張用（定義だけ） ===
    Opaque(String),              // 不透明型
    
    // ❌ 削除: Future/Streamは値型ではない！
    // Future(Box<BidType>),  // 削除
    // Stream(Box<BidType>),  // 削除
}
```

### 2. 実行モデル（MethodShape）- 新設計

```rust
// メソッドの実行形状を表現（ChatGPT推奨）
#[derive(Clone, Debug, PartialEq)]
pub enum MethodShape {
    Sync,       // 通常の同期呼び出し
    Async,      // Future<T>を返す（ハンドル経由）
    Streaming,  // Stream<T>を返す（ハンドル経由）
}

// メソッドシグネチャ（形状と値型を分離）
#[derive(Clone, Debug)]
pub struct MethodSig {
    pub name: String,
    pub shape: MethodShape,     // 実行モデル
    pub params: Vec<BidType>,   // 引数の値型
    pub returns: BidType,       // 戻り値の値型（Future抜き）
    pub effects: Vec<Effect>,
}

// BID定義でメソッド記述
#[derive(Clone, Debug)]
pub struct Method {
    pub sig: MethodSig,
    pub doc: Option<String>,
}
```

### 3. 非同期ハンドル（FFI境界用）

```rust
// ChatGPT推奨のハンドル方式
use std::ffi::c_void;

// FFI境界での非同期ハンドル（不透明ポインタ）
#[repr(transparent)]
pub struct BidFutureHandle(*mut c_void);

#[repr(transparent)]
pub struct BidStreamHandle(*mut c_void);

// Rust側の安全ラッパー
pub struct BidFuture {
    handle: BidFutureHandle,
    return_type: BidType,
}

pub struct BidStream {
    handle: BidStreamHandle,
    item_type: BidType,
}

// 将来のRust async/await統合
impl std::future::Future for BidFuture {
    type Output = Result<BidValue, BidError>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // FFI経由でpolling or callback設定
        unimplemented!("Phase 3で実装")
    }
}

impl futures_core::Stream for BidStream {
    type Item = Result<BidValue, BidError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!("Phase 3で実装")
    }
}
```

### 4. Connection trait（形状別実装）

```rust
// ChatGPT推奨の分離アプローチ
pub trait Connection: Send + Sync {
    // 同期呼び出し（Phase 1で実装）
    fn invoke(&self, sig: &MethodSig, args: &[BidValue]) -> Result<BidValue, BidError>;
    
    // 非同期呼び出し（Phase 3で実装）
    fn invoke_future(&self, sig: &MethodSig, args: &[BidValue]) -> Result<BidFuture, BidError> {
        Err(BidError::Unsupported("async not supported yet".to_string()))
    }
    
    // ストリーミング（Phase 3で実装）
    fn invoke_stream(&self, sig: &MethodSig, args: &[BidValue]) -> Result<BidStream, BidError> {
        Err(BidError::Unsupported("streaming not supported yet".to_string()))
    }
}
```

### 5. FFI境界の非同期API（Phase 3で実装）

```c
// ChatGPT推奨のC ABI設計（Phase 3で実装予定）

// Future操作
extern "C" fn bid_future_poll(
    handle: *mut c_void,
    out_value: *mut BidValue,
    out_is_ready: *mut bool
) -> BidStatus;

extern "C" fn bid_future_set_callback(
    handle: *mut c_void,
    callback: extern "C" fn(*mut c_void, BidValue, BidStatus),
    user_data: *mut c_void
) -> BidStatus;

extern "C" fn bid_future_cancel(handle: *mut c_void) -> BidStatus;
extern "C" fn bid_future_free(handle: *mut c_void);

// Stream操作
extern "C" fn bid_stream_poll_next(
    handle: *mut c_void,
    out_item: *mut BidValue,
    out_has_item: *mut bool,
    out_is_closed: *mut bool
) -> BidStatus;

extern "C" fn bid_stream_set_callback(
    handle: *mut c_void,
    callback: extern "C" fn(*mut c_void, BidValue, bool, BidStatus),
    user_data: *mut c_void
) -> BidStatus;

extern "C" fn bid_stream_close(handle: *mut c_void) -> BidStatus;
extern "C" fn bid_stream_free(handle: *mut c_void);
```

## 📋 修正された実装スケジュール

### Phase 1（1週間）- 同期のみ
```rust
// 実装するもの
- BidType基本型（Bool, I32, I64, F32, F64, String）
- MethodShape::Syncのみ
- DynamicLibraryコネクター
- Connection::invoke()のみ

// 実装しないもの
- 非同期型（Future/Stream） → 定義から削除済み
- MethodShape::Async/Streaming → unsupportedエラー
```

### Phase 2（2週間後）- 複合型
```rust
// 追加実装
- Array, List, Map, Option, Result型
- エラー処理の充実
- 複数プラグイン同時ロード
```

### Phase 3（1ヶ月後）- 非同期
```rust
// ハンドル方式で非同期追加
- BidFuture/BidStream実装
- FFI境界非同期API
- Rust async/await統合
- WasmComponent対応
```

## 🌟 ChatGPT先生の知恵のまとめ

1. **型と実行モデルの分離** - 値型は純粋に、実行形状は別定義
2. **FFI境界の現実性** - ハンドル＋API関数群で非同期表現
3. **WASM整合性** - Component Modelの流儀に準拠
4. **段階的実装** - unsupportedエラーでpanic回避
5. **将来拡張性** - Transport差異を抽象化で吸収

## ✅ この設計の利点

- **シンプル**: 型システムが明確（値型のみ）
- **拡張可能**: 実行モデルを後から追加可能
- **FFI現実的**: C ABIで実際に渡せる形
- **標準準拠**: WASM Component Modelと整合
- **実装しやすい**: 同期から始めて段階的に

---

**修正日**: 2025-08-17  
**修正理由**: ChatGPT先生のアドバイス適用  
**重要な変更**: Future/Stream削除、MethodShape導入