# Phase 9.75g: BID統合プラグインアーキテクチャ実装計画

## 🎯 概要

**目的**: ビルトインBox動的ライブラリ化とBID（Box Interface Definition）統合により、全バックエンド（インタープリター/VM/WASM/AOT）で統一的に使えるプラグインシステムを構築する。

**期間**: 2週間（段階的実装）

**優先度**: 🔥 最高（VM性能改善の基盤にもなる）

## 🌟 設計哲学（AI大会議の結論を反映）

### 二層化アーキテクチャ
```
┌─────────────────────────────────────────┐
│        Nyashコード（不変）              │
├─────────────────────────────────────────┤
│     BID層（インターフェース定義）       │
│    - 型定義、メソッドシグネチャ        │
│    - エフェクト、エラー仕様           │
├─────────────────────────────────────────┤
│    Connector層（実装・トランスポート）  │
│    - DynamicLibrary (.so/.dll)         │
│    - REST/gRPC（将来）                │
│    - Language Bridge（将来）           │
└─────────────────────────────────────────┘
```

### 設計原則
1. **段階的実装**: 完璧を求めず、動くものから始める
2. **最小型集合**: i64, f64, string, bool, handle から開始
3. **コード生成**: 手書きコードを最小化、型安全性確保
4. **粗粒度API**: tight loopを避ける設計指針

## 📋 実装フェーズ

### Phase 9.75g-1: BID基盤実装（3日）

#### 1.1 BIDパーサー実装（Day 1）
```rust
// src/bid/parser.rs
pub struct BidDefinition {
    pub version: u32,
    pub transport: Transport,
    pub interfaces: Vec<Interface>,
}

pub struct Interface {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub methods: Vec<Method>,
}

pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub returns: Option<Type>,
    pub effects: Vec<Effect>,
}

// YAMLパーサー（serde_yaml使用）
pub fn parse_bid(yaml_content: &str) -> Result<BidDefinition, BidError> {
    // 実装
}
```

#### 1.2 統一型システム（Day 1）
```rust
// src/bid/types.rs
#[derive(Clone, Debug)]
pub enum BidType {
    // 基本型（Phase 1）
    Bool,
    I32,
    I64,
    F64,
    String,
    Bytes,
    Handle(String),  // resource<T>
    
    // 将来の拡張用
    List(Box<BidType>),
    Map(Box<BidType>, Box<BidType>),
    Optional(Box<BidType>),
    Result(Box<BidType>, Box<BidType>),
}

// MirValueとの相互変換
impl BidType {
    pub fn to_mir_type(&self) -> MirType {
        match self {
            BidType::I64 => MirType::Integer,
            BidType::F64 => MirType::Float,
            BidType::String => MirType::String,
            BidType::Bool => MirType::Bool,
            BidType::Handle(name) => MirType::Box(name.clone()),
            _ => todo!("Phase 2で実装")
        }
    }
}
```

#### 1.3 UniversalConnectorトレイト（Day 2）
```rust
// src/bid/connector.rs
pub trait UniversalConnector: Send + Sync {
    /// BID定義から接続を確立
    fn connect(&self, bid: &BidDefinition) -> Result<Box<dyn Connection>, BidError>;
    
    /// サポートするトランスポートタイプ
    fn supported_transport(&self) -> TransportType;
}

pub trait Connection: Send + Sync {
    /// インターフェースのvtableを取得（高速パス用）
    fn get_vtable(&self, interface: &str) -> Option<InterfaceVTable>;
    
    /// 汎用呼び出し（リモート/ブリッジ用）
    fn invoke(&self, 
        interface: &str, 
        method: &str, 
        args: &[BidValue]
    ) -> Result<BidValue, BidError>;
}
```

#### 1.4 統一エラーモデル（Day 2）
```rust
// src/bid/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BidError {
    #[error("Transport error: {0}")]
    Transport(String),
    
    #[error("Interface error in {interface}: {message}")]
    Interface { interface: String, message: String },
    
    #[error("Method not found: {interface}.{method}")]
    MethodNotFound { interface: String, method: String },
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Remote execution error: {0}")]
    Remote(String),
    
    // カテゴリー別エラー（ChatGPT提案）
    #[error("{category} error: {message}")]
    Categorized { 
        category: ErrorCategory,
        message: String,
        retryable: bool,
    },
}

#[derive(Debug)]
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
}
```

### Phase 9.75g-2: C ABI動的ライブラリConnector（3日）

#### 2.1 DynamicLibraryConnector実装（Day 3）
```rust
// src/bid/connectors/dynamic_library.rs
pub struct DynamicLibraryConnector {
    library_cache: Mutex<HashMap<String, Arc<Library>>>,
}

impl UniversalConnector for DynamicLibraryConnector {
    fn connect(&self, bid: &BidDefinition) -> Result<Box<dyn Connection>, BidError> {
        let path = &bid.transport.location;
        
        // ライブラリをロード
        let library = unsafe { 
            Library::new(path)
                .map_err(|e| BidError::Transport(format!("Failed to load {}: {}", path, e)))?
        };
        
        // バージョンチェック
        let version_fn: Symbol<unsafe extern "C" fn() -> u32> = unsafe {
            library.get(b"nyash_bid_version\0")?
        };
        
        let version = unsafe { version_fn() };
        if version != bid.version {
            return Err(BidError::Transport(format!(
                "Version mismatch: expected {}, got {}", 
                bid.version, version
            )));
        }
        
        Ok(Box::new(DynamicLibraryConnection {
            library: Arc::new(library),
            bid: bid.clone(),
        }))
    }
}
```

#### 2.2 高速vtableパス（Day 4）
```rust
// src/bid/vtable.rs
#[repr(C)]
pub struct InterfaceVTable {
    pub version: u32,
    pub interface_id: [u8; 16],  // UUID
    pub method_count: u32,
    pub methods: *const MethodEntry,
}

#[repr(C)]
pub struct MethodEntry {
    pub name: *const c_char,
    pub function: *const c_void,
    pub param_count: u32,
    pub param_types: *const BidTypeId,
    pub return_type: BidTypeId,
}

// 使用例（FileBox）
impl DynamicLibraryConnection {
    fn get_vtable(&self, interface: &str) -> Option<InterfaceVTable> {
        // シンボル名: nyash_{interface}_vtable
        let symbol_name = format!("nyash_{}_vtable\0", interface);
        
        let vtable_ptr: Symbol<*const InterfaceVTable> = unsafe {
            self.library.get(symbol_name.as_bytes()).ok()?
        };
        
        Some(unsafe { (*vtable_ptr).clone() })
    }
}
```

#### 2.3 FileBoxプラグイン移植（Day 5）
```rust
// plugins/nyash-file/src/lib.rs
use nyash_bid::*;

// C ABI関数
#[no_mangle]
pub extern "C" fn nyash_bid_version() -> u32 {
    1
}

#[no_mangle]
pub static NYASH_FILE_VTABLE: InterfaceVTable = InterfaceVTable {
    version: 1,
    interface_id: *b"nyash.file.v1.0\0",
    method_count: 4,
    methods: &FILE_METHODS as *const _,
};

static FILE_METHODS: [MethodEntry; 4] = [
    MethodEntry {
        name: b"open\0" as *const _ as *const c_char,
        function: nyash_file_open as *const _,
        param_count: 2,
        param_types: &[BidTypeId::String, BidTypeId::String] as *const _,
        return_type: BidTypeId::Handle,
    },
    // read, write, close...
];

// 実装
#[no_mangle]
pub extern "C" fn nyash_file_open(
    path: *const c_char,
    mode: *const c_char,
) -> *mut FileHandle {
    // 既存のFileBox実装を再利用
}
```

### Phase 9.75g-3: インタープリター統合（2日）

#### 3.1 BIDローダー統合（Day 6）
```rust
// src/interpreter/bid_loader.rs
pub struct BidPluginLoader {
    connectors: HashMap<TransportType, Box<dyn UniversalConnector>>,
    connections: HashMap<String, Box<dyn Connection>>,
}

impl BidPluginLoader {
    pub fn new() -> Self {
        let mut connectors = HashMap::new();
        
        // Phase 1: 動的ライブラリのみ
        connectors.insert(
            TransportType::DynamicLibrary,
            Box::new(DynamicLibraryConnector::new()),
        );
        
        Self {
            connectors,
            connections: HashMap::new(),
        }
    }
    
    pub fn load_bid(&mut self, yaml_path: &str) -> Result<(), BidError> {
        let content = fs::read_to_string(yaml_path)?;
        let bid = parse_bid(&content)?;
        
        // 適切なコネクターを選択
        let connector = self.connectors
            .get(&bid.transport.transport_type)
            .ok_or_else(|| BidError::Transport(
                format!("Unsupported transport: {:?}", bid.transport.transport_type)
            ))?;
        
        // 接続を確立
        let connection = connector.connect(&bid)?;
        
        // インターフェースごとに登録
        for interface in &bid.interfaces {
            let key = format!("{}.{}", interface.namespace, interface.name);
            self.connections.insert(key, connection.clone());
        }
        
        Ok(())
    }
}
```

#### 3.2 既存コードとの互換性層（Day 7）
```rust
// src/interpreter/objects.rs の修正
impl NyashInterpreter {
    fn execute_new(&mut self, class: &str, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> 
    {
        // 既存のビルトインBox処理
        if is_builtin_box(class) {
            // 従来の処理...
        }
        
        // BIDプラグインチェック
        if let Some(connection) = self.bid_loader.get_connection(class) {
            // BID経由で作成
            let bid_args: Vec<BidValue> = args.iter()
                .map(|arg| nyash_to_bid_value(arg))
                .collect::<Result<_, _>>()?;
            
            let result = connection.invoke(class, "new", &bid_args)?;
            
            return Ok(bid_to_nyash_box(result)?);
        }
        
        // ユーザー定義Box
        // 従来の処理...
    }
}
```

### Phase 9.75g-4: MIR/VM統合（3日）

#### 4.1 ExternCall命令とBID統合（Day 8）
```rust
// src/mir/builder.rs の修正
impl MirBuilder {
    fn build_method_call(&mut self, object: ASTNode, method: String, args: Vec<ASTNode>) 
        -> Result<ValueId, String> 
    {
        // オブジェクトの型を解析
        let obj_type = self.infer_type(&object)?;
        
        // BIDプラグインメソッドかチェック
        if let Some(bid_interface) = self.bid_registry.get_interface(&obj_type) {
            // ExternCall命令を生成
            let dst = self.value_gen.next();
            self.emit_instruction(MirInstruction::ExternCall {
                dst: Some(dst),
                iface_name: bid_interface.name.clone(),
                method_name: method,
                args: arg_values,
                effects: bid_interface.get_method_effects(&method),
            })?;
            
            return Ok(dst);
        }
        
        // 通常のBoxCall
        // 従来の処理...
    }
}
```

#### 4.2 VM実行時BID呼び出し（Day 9）
```rust
// src/backend/vm.rs の修正
impl VM {
    fn execute_extern_call(&mut self, 
        dst: Option<ValueId>,
        iface: &str,
        method: &str,
        args: &[ValueId],
        effects: &EffectMask,
    ) -> Result<(), VMError> {
        // BID接続を取得
        let connection = self.bid_loader
            .get_connection(iface)
            .ok_or_else(|| VMError::InterfaceNotFound(iface.to_string()))?;
        
        // 引数をBidValueに変換
        let bid_args: Vec<BidValue> = args.iter()
            .map(|id| self.vm_to_bid_value(*id))
            .collect::<Result<_, _>>()?;
        
        // 高速パスチェック（vtable利用可能か）
        if let Some(vtable) = connection.get_vtable(iface) {
            // 直接関数ポインタ呼び出し（最速）
            let result = unsafe { 
                call_vtable_method(&vtable, method, &bid_args)? 
            };
            
            if let Some(dst_id) = dst {
                self.set_value(dst_id, bid_to_vm_value(result)?);
            }
        } else {
            // 汎用invoke経路（リモート対応）
            let result = connection.invoke(iface, method, &bid_args)?;
            
            if let Some(dst_id) = dst {
                self.set_value(dst_id, bid_to_vm_value(result)?);
            }
        }
        
        Ok(())
    }
}
```

### Phase 9.75g-5: コード生成ツール（2日）

#### 5.1 BIDからRustスタブ生成（Day 10）
```bash
# CLIツール
nyash-bid-gen --input file.bid.yaml --output src/generated/
```

生成されるコード例:
```rust
// src/generated/nyash_file.rs
pub struct FileBoxClient {
    connection: Arc<dyn Connection>,
}

impl FileBoxClient {
    pub fn open(&self, path: &str, mode: &str) -> Result<FileHandle, BidError> {
        let args = vec![
            BidValue::String(path.to_string()),
            BidValue::String(mode.to_string()),
        ];
        
        let result = self.connection.invoke("nyash.file", "open", &args)?;
        
        match result {
            BidValue::Handle(h) => Ok(FileHandle(h)),
            _ => Err(BidError::TypeMismatch {
                expected: "handle".to_string(),
                actual: format!("{:?}", result),
            }),
        }
    }
}
```

#### 5.2 プラグイン側スケルトン生成（Day 11）
```rust
// 生成されるプラグイン側のスケルトン
pub trait FileBoxImpl {
    fn open(&self, path: &str, mode: &str) -> Result<FileHandle, FileError>;
    fn read(&self, handle: &FileHandle, size: usize) -> Result<Vec<u8>, FileError>;
    fn write(&self, handle: &FileHandle, data: &[u8]) -> Result<usize, FileError>;
    fn close(&self, handle: FileHandle) -> Result<(), FileError>;
}

// C ABIラッパーも自動生成
#[no_mangle]
pub extern "C" fn nyash_file_open(
    path: *const c_char,
    mode: *const c_char,
) -> *mut c_void {
    // 実装への橋渡し
}
```

## 📊 テスト計画

### 統合テスト（Day 12）
```nyash
// test_bid_integration.nyash
using nyashstd

// BIDプラグインのロード
bid.load("plugins/file.bid.yaml")

// 通常のNyashコードで使用（透過的）
local file = new FileBox("test.txt", "w")
file.write("Hello from BID!")
file.close()

// 性能測定
local timer = new TimerBox()
timer.start()

local i = 0
loop(i < 1000) {
    local f = new FileBox("bench.txt", "r")
    f.read(1024)
    f.close()
    i = i + 1
}

timer.stop()
console.log("1000 file operations: " + timer.elapsed() + "ms")
```

### ベンチマーク目標
- C ABI直接呼び出し: < 100ns オーバーヘッド
- 型変換コスト: < 50ns（基本型）
- メモリ効率: 既存実装と同等以下

## 🎯 成功基準

### Phase 9.75g完了時
- [ ] BIDパーサー・型システム・エラーモデル完成
- [ ] DynamicLibraryConnector完全動作
- [ ] FileBoxがBID経由で動作
- [ ] インタープリター/VM/WASMすべてで同じBIDが使える
- [ ] コード生成ツール基本機能
- [ ] 性能目標達成（< 100ns オーバーヘッド）

### 将来の拡張準備
- [ ] Transport抽象化の拡張ポイント確保
- [ ] ストリーミング/非同期の設計考慮
- [ ] セキュリティ拡張ポイント予約

## 🔧 実装の鍵

### 1. 段階的アプローチ
- 完璧を求めない
- 動くものから始める
- フィードバックを早く得る

### 2. 既存資産の活用
- FileBoxProxyの実装を再利用
- 既存のプラグインローダーと共存

### 3. 性能最優先
- C ABI高速パスを最初に実装
- 型変換を最小化
- ゼロコピーを目指す

### 4. 開発者体験
- コード生成で型安全性
- エラーメッセージの充実
- デバッグ支援機能

## 📅 マイルストーン

- **Week 1**: BID基盤 + C ABIコネクター + FileBox移植
- **Week 2**: インタープリター/VM統合 + コード生成 + テスト

## 🚀 期待される成果

1. **統一プラグインシステム**: 全バックエンドで同じプラグインが動く
2. **ビルド時間改善**: 動的ライブラリ化で2分→15秒
3. **将来の拡張性**: REST/gRPC/Python等への道筋
4. **VM性能改善の基盤**: BID経由のプロファイリング・最適化

---

**作成**: 2025-08-17  
**作成者**: Claude (AI大会議の結論を統合)  
**優先度**: 🔥 最高（VM性能改善の前提）