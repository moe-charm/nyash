# Phase 9.75g-0 改訂版: 型定義ファースト戦略

## 🎯 基本方針：型は全部、実装は段階的

**ユーザーの賢い指摘**：構造体の定義を最初に全部やっておけば、ビルドは楽になる！

## 📦 Phase 1で定義する全ての型（実装は後でOK）

```rust
// src/bid/types.rs - 全ての型を最初に定義！

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
    
    // === 複合型（定義だけ、実装はPhase 2） ===
    Array(Box<BidType>),           // 配列
    List(Box<BidType>),           // 可変長リスト
    Map(Box<BidType>, Box<BidType>), // キーバリューマップ
    Tuple(Vec<BidType>),          // タプル
    Record(Vec<(String, BidType)>), // 名前付きフィールド
    Variant(Vec<(String, Option<BidType>)>), // 列挙型
    
    // === 特殊型（定義だけ、実装はPhase 2） ===
    Option(Box<BidType>),         // null許容
    Result(Box<BidType>, Box<BidType>), // エラー型
    Handle(String),               // 不透明ハンドル
    Void,                        // 戻り値なし
    
    // === 非同期型（定義だけ、実装はPhase 3） ===
    Future(Box<BidType>),         // 非同期結果
    Stream(Box<BidType>),         // ストリーム
    
    // === 拡張用（定義だけ） ===
    Opaque(String),              // 不透明型
    Extension(String, Box<dyn std::any::Any + Send + Sync>), // 拡張用
}

// Transport層も全部定義（実装は段階的）
#[derive(Clone, Debug)]
pub enum TransportType {
    // Phase 1で実装
    DynamicLibrary,
    
    // Phase 2で実装（定義だけ先に）
    Grpc,
    Rest,
    WebSocket,
    
    // Phase 3で実装（定義だけ先に）
    WasmComponent,
    PythonBridge,
    
    // Phase 4で実装（定義だけ先に）
    P2P,           // NyaMesh統合
    Quantum,       // 量子コンピュータ（夢）
}

// Effect定義も完全版
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Pure,       // 副作用なし
    Mut,        // 状態変更
    Io,         // I/O操作
    Control,    // 制御フロー
    
    // 将来の拡張（定義だけ）
    Async,      // 非同期
    Parallel,   // 並列実行可
    Network,    // ネットワーク
    Gpu,        // GPU使用
}

// エラー型も完全定義
#[derive(Debug, thiserror::Error)]
pub enum BidError {
    // Phase 1で実装
    #[error("Transport error: {0}")]
    Transport(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    // Phase 2で実装（定義だけ先に）
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    // Phase 3で実装（定義だけ先に）
    #[error("Async error: {0}")]
    Async(String),
    
    #[error("Language bridge error: {0}")]
    LanguageBridge(String),
    
    // エラーカテゴリ（完全定義）
    #[error("{category} error: {message}")]
    Categorized {
        category: ErrorCategory,
        message: String,
        retryable: bool,
        details: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Invalid,
    NotFound,
    Conflict,
    Unavailable,
    Timeout,
    Cancelled,
    Internal,
    Permission,
    Resource,
    // 将来の拡張
    Quantum,
}

// UniversalConnectorも完全インターフェース
pub trait UniversalConnector: Send + Sync {
    fn connect(&self, bid: &BidDefinition) -> Result<Box<dyn Connection>, BidError>;
    fn supported_transport(&self) -> TransportType;
    
    // Phase 2で実装（デフォルト実装で逃げる）
    fn handshake(&self) -> Result<HandshakeInfo, BidError> {
        Ok(HandshakeInfo::default())
    }
    
    fn describe(&self) -> Result<Vec<InterfaceDescription>, BidError> {
        Ok(vec![])
    }
    
    // Phase 3で実装
    fn async_connect(&self, bid: &BidDefinition) -> Result<Box<dyn AsyncConnection>, BidError> {
        unimplemented!("Async not supported yet")
    }
}

// Connection trait も完全版
pub trait Connection: Send + Sync {
    // Phase 1で実装
    fn invoke(&self, method: &str, args: &[BidValue]) -> Result<BidValue, BidError>;
    
    // Phase 2で実装（デフォルト実装）
    fn invoke_async(&self, method: &str, args: &[BidValue]) -> Result<FutureHandle, BidError> {
        unimplemented!("Async not supported")
    }
    
    fn stream(&self, method: &str, args: &[BidValue]) -> Result<StreamHandle, BidError> {
        unimplemented!("Streaming not supported")
    }
    
    // Phase 3で実装
    fn batch(&self, calls: Vec<(String, Vec<BidValue>)>) -> Result<Vec<BidValue>, BidError> {
        unimplemented!("Batch not supported")
    }
}

// 実装用のマクロ（Phase 1では基本型のみ実装）
impl BidType {
    pub fn to_wasm_types(&self) -> Vec<WasmType> {
        match self {
            // Phase 1: これらは実装
            BidType::Bool => vec![WasmType::I32],
            BidType::I32 => vec![WasmType::I32],
            BidType::I64 => vec![WasmType::I64],
            BidType::F32 => vec![WasmType::F32],
            BidType::F64 => vec![WasmType::F64],
            BidType::String => vec![WasmType::I32, WasmType::I32],
            
            // Phase 2以降: とりあえずpanic
            _ => unimplemented!("Type {:?} not implemented yet", self),
        }
    }
}
```

## 🎯 この戦略のメリット

1. **ビルドエラーなし** - 型は全部あるのでコンパイル通る
2. **API安定** - 最初から完全なAPIが見える
3. **段階的実装** - `unimplemented!()` から順次実装
4. **将来の拡張が楽** - 構造体変更不要

## 📅 実装スケジュール

### Phase 1（1週間）
```rust
// 実装するもの
- 基本型（Bool, I32, I64, F32, F64, String）
- DynamicLibraryコネクター
- 同期invoke()のみ
- Linux x86-64のみ

// 実装しないもの（unimplemented!）
- 複合型（Array, Map等）
- 非同期処理
- ネットワーク
```

### Phase 2（2週間後）
```rust
// 追加実装
- Array, List, Map型
- Option, Result型  
- エラー処理の充実
```

### Phase 3（1ヶ月後）
```rust
// 非同期対応
- Future, Stream型
- async_connect, invoke_async
- WasmComponent対応
```

### Phase 4（将来）
```rust
// 拡張機能
- P2P（NyaMesh統合）
- 量子コンピュータ（？）
```

## 📝 実装例（Phase 1）

```rust
// src/bid/connectors/dynamic_library.rs

impl UniversalConnector for DynamicLibraryConnector {
    fn connect(&self, bid: &BidDefinition) -> Result<Box<dyn Connection>, BidError> {
        // Phase 1: 実装する
        let lib = unsafe { libloading::Library::new(&bid.transport.location)? };
        Ok(Box::new(DynamicLibraryConnection { lib }))
    }
    
    fn supported_transport(&self) -> TransportType {
        TransportType::DynamicLibrary
    }
    
    // Phase 2以降: デフォルト実装のまま
}

impl Connection for DynamicLibraryConnection {
    fn invoke(&self, method: &str, args: &[BidValue]) -> Result<BidValue, BidError> {
        // Phase 1: 基本型のみ実装
        match args[0] {
            BidValue::I32(n) => { /* 実装 */ },
            BidValue::String(s) => { /* 実装 */ },
            
            // Phase 2以降
            BidValue::Array(_) => unimplemented!("Array not supported yet"),
            BidValue::Future(_) => unimplemented!("Future not supported yet"),
        }
    }
}
```

## ✨ まとめ

**構造体は最初に全部定義、実装は段階的に** - これでビルドエラーなしで、APIも安定！

ユーザーの「深く考えて」の結果：この方が絶対に楽です。将来Array型を追加するときも、構造体はもうあるので実装を書くだけ！

---

**改訂日**: 2025-08-17  
**改訂理由**: 型定義ファースト戦略の採用