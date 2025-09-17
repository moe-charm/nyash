# Phase 9.75g-0: BID-FFI ABI v0 統合仕様

## 🎯 概要

**目的**: BID（Box Interface Definition）をFFI ABI v0仕様に準拠させ、シンプルで実用的な実装を実現する

**背景**: 
- FFI ABI v0は既に確立された明確な仕様
- BID Phase 9.75gは野心的だが過度に複雑
- 両者の良い部分を統合し、段階的実装を可能にする

**期間**: 1週間（Phase 9.75g-1の前に実施）

## 📊 FFI ABI v0とBIDの差異分析

### 型システムの比較

| 項目 | FFI ABI v0 | BID (現在) | 統合案 |
|------|------------|------------|---------|
| 整数型 | i32, i64 | i64のみ | **i32, i64** |
| 浮動小数点 | f32, f64 | f64のみ | **f32, f64** |
| 文字列 | (ptr:i32, len:i32) | string（曖昧） | **string: (ptr:i32, len:i32)** |
| 配列 | array(T): (ptr, len) | 将来拡張 | **array(T): (ptr:i32, len:i32)** |
| 真偽値 | i32 (0/1) | bool | **bool: i32表現** |
| ハンドル | boxref: i32 | handle(String) | **handle: i32** |

### メモリモデルの統一

```rust
// FFI ABI v0準拠のメモリレイアウト
pub struct MemoryLayout {
    // ポインタサイズ: WASM MVPは32ビット
    pointer_size: 32,
    
    // アライメント: 4バイト推奨
    alignment: 4,
    
    // エンディアン: リトルエンディアン
    endianness: LittleEndian,
    
    // 文字列: UTF-8、NUL終端不要
    string_encoding: UTF8,
}
```

## 🛠️ 実装計画

### Phase 9.75g-0: FFI ABI v0準拠（新規・1週間）

#### Day 1-2: 型システム統合

```rust
// src/bid/types.rs - FFI ABI v0準拠版
#[derive(Clone, Debug, PartialEq)]
pub enum BidType {
    // === 基本型（FFI ABI v0準拠） ===
    Bool,       // i32 (0=false, 1=true)
    I32,        // 32ビット符号付き整数
    I64,        // 64ビット符号付き整数
    F32,        // IEEE 754 単精度
    F64,        // IEEE 754 倍精度
    
    // === 複合型 ===
    String,     // UTF-8 (ptr:i32, len:i32)
    Bytes,      // バイナリ (ptr:i32, len:i32)
    Array(Box<BidType>),  // 配列 (ptr:i32, len:i32)
    
    // === 特殊型 ===
    Handle(String),  // 不透明ハンドル（i32）
    Void,           // 戻り値なし
    
    // === Phase 2以降 ===
    List(Box<BidType>),
    Map(Box<BidType>, Box<BidType>),
    Optional(Box<BidType>),
    Result(Box<BidType>, Box<BidType>),
}

impl BidType {
    /// FFI ABI v0でのWASM表現を返す
    pub fn to_wasm_types(&self) -> Vec<WasmType> {
        match self {
            BidType::Bool => vec![WasmType::I32],
            BidType::I32 => vec![WasmType::I32],
            BidType::I64 => vec![WasmType::I64],
            BidType::F32 => vec![WasmType::F32],
            BidType::F64 => vec![WasmType::F64],
            BidType::String => vec![WasmType::I32, WasmType::I32], // ptr, len
            BidType::Bytes => vec![WasmType::I32, WasmType::I32],  // ptr, len
            BidType::Array(_) => vec![WasmType::I32, WasmType::I32], // ptr, len
            BidType::Handle(_) => vec![WasmType::I32],
            BidType::Void => vec![],
            _ => panic!("Phase 2型は未実装"),
        }
    }
    
    /// MirTypeとの相互変換
    pub fn from_mir_type(mir_type: &MirType) -> Self {
        match mir_type {
            MirType::Integer => BidType::I64,  // Nyashのデフォルト
            MirType::Float => BidType::F64,    // Nyashのデフォルト
            MirType::String => BidType::String,
            MirType::Bool => BidType::Bool,
            MirType::Box(name) => BidType::Handle(name.clone()),
            _ => panic!("未対応のMirType: {:?}", mir_type),
        }
    }
}
```

#### Day 3: Effect System統合

```rust
// src/bid/effects.rs - FFI ABI v0準拠
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// 純粋関数：再順序化可能、メモ化可能
    Pure,
    
    /// 可変状態：同一リソースへの操作順序を保持
    Mut,
    
    /// I/O操作：プログラム順序を厳密に保持
    Io,
    
    /// 制御フロー：CFGに影響（break, continue, return等）
    Control,
}

impl Effect {
    /// MIR EffectMaskとの変換
    pub fn to_mir_effects(&self) -> EffectMask {
        match self {
            Effect::Pure => EffectMask::empty(),
            Effect::Mut => EffectMask::MUT,
            Effect::Io => EffectMask::IO,
            Effect::Control => EffectMask::CONTROL,
        }
    }
}
```

#### Day 4: Box Layout標準化

```rust
// src/bid/layout.rs - FFI ABI v0準拠のBoxレイアウト
#[repr(C)]
pub struct BoxHeader {
    type_id: i32,      // Box型識別子
    ref_count: i32,    // 参照カウント（Arc実装用）
    field_count: i32,  // フィールド数
}

#[repr(C)]
pub struct StringBoxLayout {
    header: BoxHeader,
    data_ptr: i32,     // UTF-8データへのポインタ
    length: i32,       // 文字列長（バイト数）
}

// レイアウト検証関数
pub fn validate_alignment(ptr: *const u8) -> bool {
    (ptr as usize) % 4 == 0  // 4バイトアライメント
}
```

#### Day 5: BID YAML仕様の調整

```yaml
# FFI ABI v0準拠のBID定義
version: 0  # FFI ABI v0準拠を明示

# メモリモデル宣言（オプション、デフォルトはFFI ABI v0）
memory:
  pointer_size: 32
  alignment: 4
  endianness: little

# トランスポート層（Phase 1はシンプルに）
transport:
  type: dynamic_library
  location: ./libnyash_file.so
  # 将来の拡張を予約
  future_transports: [grpc, rest, wasm, python_bridge]

interfaces:
  - namespace: nyash
    name: file
    version: "1.0.0"
    
    # 型定義（FFI ABI v0準拠）
    types:
      - name: FileHandle
        type: handle
        
    methods:
      - name: open
        params:
          - { name: path, type: string }     # (ptr:i32, len:i32)
          - { name: mode, type: string }     # (ptr:i32, len:i32)
        returns: { type: FileHandle }        # i32
        effect: io
        
      - name: read
        params:
          - { name: handle, type: FileHandle }  # i32
          - { name: size, type: i32 }          # 32ビット整数
        returns: { type: bytes }               # (ptr:i32, len:i32)
        effect: io
        
      - name: write
        params:
          - { name: handle, type: FileHandle }  # i32
          - { name: data, type: bytes }        # (ptr:i32, len:i32)
        returns: { type: i32 }                 # 書き込みバイト数
        effect: io
        
      - name: close
        params:
          - { name: handle, type: FileHandle }  # i32
        returns: { type: void }
        effect: io
```

### Phase 9.75g-1: 調整版実装（1週間）

#### 簡素化されたUniversalConnector

```rust
// src/bid/connector.rs - シンプル版
pub trait UniversalConnector: Send + Sync {
    /// Phase 1: dynamic_libraryのみサポート
    fn connect(&self, bid: &BidDefinition) -> Result<Box<dyn Connection>, BidError>;
    
    fn supported_transport(&self) -> TransportType {
        TransportType::DynamicLibrary  // Phase 1固定
    }
}

// 将来の拡張ポイントを明示
#[derive(Debug, Clone)]
pub enum TransportType {
    DynamicLibrary,      // Phase 1
    #[allow(dead_code)]
    Grpc,               // Phase 2
    #[allow(dead_code)]  
    Rest,               // Phase 2
    #[allow(dead_code)]
    WasmComponent,      // Phase 3
    #[allow(dead_code)]
    PythonBridge,       // Phase 3
}
```

#### C ABI関数シグネチャ生成

```rust
// src/bid/codegen/c_abi.rs
impl Method {
    /// FFI ABI v0準拠のC関数シグネチャを生成
    pub fn to_c_signature(&self) -> String {
        let mut params = Vec::new();
        
        for param in &self.params {
            match &param.param_type {
                BidType::String => {
                    params.push(format!("const char* {}_ptr", param.name));
                    params.push(format!("int32_t {}_len", param.name));
                }
                BidType::Array(_) => {
                    params.push(format!("const void* {}_ptr", param.name));
                    params.push(format!("int32_t {}_len", param.name));
                }
                BidType::I32 => params.push(format!("int32_t {}", param.name)),
                BidType::I64 => params.push(format!("int64_t {}", param.name)),
                BidType::F32 => params.push(format!("float {}", param.name)),
                BidType::F64 => params.push(format!("double {}", param.name)),
                BidType::Bool => params.push(format!("int32_t {}", param.name)),
                BidType::Handle(_) => params.push(format!("int32_t {}", param.name)),
                _ => panic!("未対応の型"),
            }
        }
        
        let return_type = match &self.returns {
            Some(BidType::Void) | None => "void",
            Some(BidType::I32) => "int32_t",
            Some(BidType::I64) => "int64_t", 
            Some(BidType::F32) => "float",
            Some(BidType::F64) => "double",
            Some(BidType::Handle(_)) => "int32_t",
            _ => panic!("複合戻り値は未対応"),
        };
        
        format!("{} nyash_{}({})", return_type, self.name, params.join(", "))
    }
}
```

## 📊 成功基準

### Phase 9.75g-0完了時
- [ ] FFI ABI v0の全型をBidTypeでサポート
- [ ] メモリレイアウトの明確な定義
- [ ] Effect systemの4種類実装
- [ ] Box layout標準の確立
- [ ] C ABI関数シグネチャの自動生成

### 互換性保証
- [ ] 既存のFileBoxがFFI ABI v0準拠で動作
- [ ] WASM/VM/LLVMすべてで同じ型表現
- [ ] 文字列の(ptr, len)表現が統一

## 🔧 実装上の注意点

### 1. 後方互換性の維持
```rust
// 既存のBID定義も動作するように
if bid.version == 0 {
    // FFI ABI v0準拠モード
} else {
    // 将来の拡張モード
}
```

### 2. エラーメッセージの改善
```rust
// 型不一致時の親切なエラー
BidError::TypeMismatch {
    expected: "string (ptr:i32, len:i32)",
    actual: "string (単一値)",
    hint: "FFI ABI v0では文字列は(ptr, len)ペアで渡す必要があります",
}
```

### 3. デバッグ支援
```rust
// BID定義の検証ツール
nyash-bid validate --ffi-abi-v0 file.bid.yaml
```

## 🎯 期待される成果

1. **明確な仕様**: FFI ABI v0準拠で曖昧さを排除
2. **実装の簡素化**: 複雑なvtableを後回しにして基本に集中
3. **相互運用性**: WASM/VM/LLVM/ネイティブで統一的な型表現
4. **段階的拡張**: 基礎が固まってから高度な機能を追加

---

**作成日**: 2025-08-17  
**作成者**: Claude  
**優先度**: 🔥 最高（Phase 9.75g-1の前提条件）