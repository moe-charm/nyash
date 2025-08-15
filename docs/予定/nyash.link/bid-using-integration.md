# BID×usingシステム統合：技術実装詳細

## 🎯 統合設計の核心

### 📊 既存システムとの整合性
- ✅ **MIR ExternCall**: 既にFFI-ABI対応実装済み
- ✅ **WASM RuntimeImports**: BID→WASM自動生成基盤あり
- ✅ **VM ExternStub**: スタブ実行環境実装済み
- 🔧 **統合課題**: usingシステムとBIDの橋渡し実装

### 🚀 統合アーキテクチャ概要
```
User Code (using statements)
     ↓
UniversalNamespaceRegistry
     ↓
CallTarget Resolution
     ↓ ↓ ↓
Builtin  FFI-ABI  NyashModule
     ↓ ↓ ↓
MIR Generation (BuiltinCall/ExternCall/ModuleCall)
     ↓
Backend Execution (VM/WASM/AOT)
```

## 🏗️ 詳細技術実装

### 1. BID定義システム

#### **BIDファイル構造拡張**
```yaml
# apis/enhanced_canvas.yaml
version: 1
metadata:
  name: "Enhanced Canvas API"
  description: "Extended Canvas API with batch operations"
  target_environments: ["browser", "node-canvas", "skia"]
  nyash_namespace: "canvas_api"  # usingで使用する名前空間

interfaces:
  - name: canvas_api.canvas
    box: Canvas
    methods:
      # 基本描画
      - name: fillRect
        params:
          - {string: canvas_id, description: "Canvas element ID"}
          - {i32: x, description: "X coordinate"}
          - {i32: y, description: "Y coordinate"}
          - {i32: width, description: "Rectangle width"}
          - {i32: height, description: "Rectangle height"}
          - {string: color, description: "Fill color (CSS format)"}
        returns: void
        effect: io
        optimization_hints:
          batch_compatible: true  # バッチ処理可能
          gpu_accelerated: true   # GPU加速対応
        
      # バッチ描画（最適化版）
      - name: fillRectBatch
        params:
          - {string: canvas_id}
          - {array_of_rect: rects, element_type: "CanvasRect"}
        returns: void
        effect: io
        optimization_hints:
          prefer_over: ["fillRect"]  # 複数fillRectの代替
          min_batch_size: 3
          
      # テキスト描画
      - name: fillText
        params:
          - {string: canvas_id}
          - {string: text}
          - {i32: x}
          - {i32: y}
          - {string: font}
          - {string: color}
        returns: void
        effect: io

# カスタム型定義
custom_types:
  - name: CanvasRect
    fields:
      - {i32: x}
      - {i32: y}
      - {i32: width}
      - {i32: height}
      - {string: color}
```

#### **BID読み込み・検証システム**
```rust
// 新ファイル: src/bid/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidDefinition {
    pub version: u32,
    pub metadata: BidMetadata,
    pub interfaces: Vec<BidInterface>,
    pub custom_types: Option<Vec<BidCustomType>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidMetadata {
    pub name: String,
    pub description: String,
    pub target_environments: Vec<String>,
    pub nyash_namespace: String,  // using文で使用する名前空間名
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidInterface {
    pub name: String,           // "canvas_api.canvas"
    pub box_name: String,       // "Canvas" 
    pub methods: Vec<BidMethod>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidMethod {
    pub name: String,
    pub params: Vec<BidParam>,
    pub returns: BidType,
    pub effect: BidEffect,
    pub optimization_hints: Option<BidOptimizationHints>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidOptimizationHints {
    pub batch_compatible: Option<bool>,
    pub gpu_accelerated: Option<bool>,
    pub prefer_over: Option<Vec<String>>,
    pub min_batch_size: Option<usize>,
}

impl BidDefinition {
    pub fn load_from_file(path: &Path) -> Result<Self, BidError> {
        let content = std::fs::read_to_string(path)?;
        let bid: BidDefinition = serde_yaml::from_str(&content)?;
        
        // バリデーション
        bid.validate()?;
        
        Ok(bid)
    }
    
    pub fn validate(&self) -> Result<(), BidError> {
        // バージョン確認
        if self.version > 1 {
            return Err(BidError::UnsupportedVersion(self.version));
        }
        
        // 名前空間重複チェック
        let mut interface_names = HashSet::new();
        for interface in &self.interfaces {
            if interface_names.contains(&interface.name) {
                return Err(BidError::DuplicateInterface(interface.name.clone()));
            }
            interface_names.insert(interface.name.clone());
        }
        
        // パラメータ型確認
        for interface in &self.interfaces {
            for method in &interface.methods {
                for param in &method.params {
                    self.validate_type(&param.param_type)?;
                }
                self.validate_type(&method.returns)?;
            }
        }
        
        Ok(())
    }
    
    pub fn resolve_method(&self, box_name: &str, method_name: &str) 
        -> Option<&BidMethod> {
        
        for interface in &self.interfaces {
            // インターフェース名から最後の部分を取得
            // "canvas_api.canvas" → "canvas"
            let interface_box_name = interface.name.split('.').last().unwrap_or(&interface.name);
            
            if interface_box_name == box_name {
                for method in &interface.methods {
                    if method.name == method_name {
                        return Some(method);
                    }
                }
            }
        }
        
        None
    }
}
```

### 2. 統合名前空間レジストリ詳細

#### **UniversalNamespaceRegistry実装**
```rust
// src/registry/universal.rs
use crate::stdlib::BuiltinStdlib;
use crate::bid::BidDefinition;
use crate::module::ExternalModule;
use crate::mir::Effect;

pub struct UniversalNamespaceRegistry {
    /// 組み込み標準ライブラリ
    builtin_stdlib: Arc<BuiltinStdlib>,
    
    /// FFI-ABI定義（BID）
    bid_definitions: HashMap<String, Arc<BidDefinition>>,
    
    /// Nyashモジュール（従来）
    nyash_modules: HashMap<String, Arc<ExternalModule>>,
    
    /// ファイル別usingコンテキスト
    using_contexts: Arc<RwLock<HashMap<String, UsingContext>>>,
    
    /// 最適化情報キャッシュ
    optimization_cache: Arc<RwLock<OptimizationCache>>,
}

#[derive(Debug, Clone)]
pub struct UsingContext {
    pub file_id: String,
    pub builtin_namespaces: Vec<String>,    // ["nyashstd"]
    pub bid_namespaces: Vec<String>,        // ["canvas_api", "console_api"]
    pub module_namespaces: Vec<String>,     // ["mylib", "utils"]
}

impl UniversalNamespaceRegistry {
    pub fn new() -> Self {
        UniversalNamespaceRegistry {
            builtin_stdlib: Arc::new(BuiltinStdlib::new()),
            bid_definitions: HashMap::new(),
            nyash_modules: HashMap::new(),
            using_contexts: Arc::new(RwLock::new(HashMap::new())),
            optimization_cache: Arc::new(RwLock::new(OptimizationCache::new())),
        }
    }
    
    pub fn load_from_nyash_link(&mut self, nyash_link: &NyashLink) 
        -> Result<(), RegistryError> {
        
        // BID依存関係読み込み
        for (namespace_name, dependency) in &nyash_link.dependencies {
            match dependency {
                Dependency::Bid { bid_path, .. } => {
                    let bid = BidDefinition::load_from_file(Path::new(bid_path))?;
                    self.bid_definitions.insert(namespace_name.clone(), Arc::new(bid));
                },
                Dependency::Path { path } => {
                    let module = ExternalModule::load_from_file(Path::new(path))?;
                    self.nyash_modules.insert(namespace_name.clone(), Arc::new(module));
                },
                Dependency::Builtin { .. } => {
                    // 組み込みライブラリは既に初期化済み
                },
            }
        }
        
        Ok(())
    }
    
    /// 統合using処理
    pub fn process_using(&mut self, namespace_name: &str, file_id: &str) 
        -> Result<(), RuntimeError> {
        
        let mut contexts = self.using_contexts.write().unwrap();
        let context = contexts.entry(file_id.to_string()).or_insert_with(|| {
            UsingContext {
                file_id: file_id.to_string(),
                builtin_namespaces: Vec::new(),
                bid_namespaces: Vec::new(),
                module_namespaces: Vec::new(),
            }
        });
        
        // 組み込み標準ライブラリチェック
        if self.builtin_stdlib.has_namespace(namespace_name) {
            if !context.builtin_namespaces.contains(&namespace_name.to_string()) {
                context.builtin_namespaces.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        // BID定義チェック
        if let Some(bid) = self.bid_definitions.get(namespace_name) {
            if !context.bid_namespaces.contains(&namespace_name.to_string()) {
                context.bid_namespaces.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        // Nyashモジュールチェック
        if let Some(_module) = self.nyash_modules.get(namespace_name) {
            if !context.module_namespaces.contains(&namespace_name.to_string()) {
                context.module_namespaces.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        Err(RuntimeError::UndefinedNamespace(namespace_name.to_string()))
    }
    
    /// 統合関数解決
    pub fn resolve_call(&self, file_id: &str, call_path: &[String]) 
        -> Result<ResolvedCall, RuntimeError> {
        
        if call_path.len() != 2 {
            return Err(RuntimeError::InvalidCallPath(call_path.join(".")));
        }
        
        let box_name = &call_path[0];
        let method_name = &call_path[1];
        
        let contexts = self.using_contexts.read().unwrap();
        if let Some(context) = contexts.get(file_id) {
            
            // 1. 組み込み標準ライブラリ解決
            for namespace in &context.builtin_namespaces {
                if let Some(method) = self.builtin_stdlib.resolve_method(namespace, box_name, method_name) {
                    return Ok(ResolvedCall::Builtin {
                        namespace: namespace.clone(),
                        box_name: box_name.clone(),
                        method_name: method_name.clone(),
                        method_info: method,
                    });
                }
            }
            
            // 2. BID定義解決
            for namespace in &context.bid_namespaces {
                if let Some(bid) = self.bid_definitions.get(namespace) {
                    if let Some(method) = bid.resolve_method(box_name, method_name) {
                        return Ok(ResolvedCall::BidCall {
                            namespace: namespace.clone(),
                            interface_name: format!("{}.{}", namespace, box_name),
                            method_name: method_name.clone(),
                            method_info: method.clone(),
                            bid_definition: bid.clone(),
                        });
                    }
                }
            }
            
            // 3. Nyashモジュール解決
            for namespace in &context.module_namespaces {
                if let Some(module) = self.nyash_modules.get(namespace) {
                    if let Some(function) = module.resolve_function(box_name, method_name) {
                        return Ok(ResolvedCall::ModuleCall {
                            namespace: namespace.clone(),
                            module_name: namespace.clone(),
                            function_name: format!("{}.{}", box_name, method_name),
                            function_info: function,
                        });
                    }
                }
            }
        }
        
        Err(RuntimeError::UndefinedMethod(format!("{}.{}", box_name, method_name)))
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedCall {
    Builtin {
        namespace: String,
        box_name: String,
        method_name: String,
        method_info: BuiltinMethodInfo,
    },
    BidCall {
        namespace: String,
        interface_name: String,
        method_name: String,
        method_info: BidMethod,
        bid_definition: Arc<BidDefinition>,
    },
    ModuleCall {
        namespace: String,
        module_name: String,
        function_name: String,
        function_info: ModuleFunctionInfo,
    },
}
```

### 3. MIR生成統合

#### **統合MIR Builder**
```rust
// src/mir/builder.rs拡張
impl MirBuilder {
    pub fn build_unified_method_call(&mut self, resolved_call: ResolvedCall, args: Vec<ValueId>) 
        -> Result<Option<ValueId>, MirError> {
        
        match resolved_call {
            ResolvedCall::Builtin { method_info, .. } => {
                let result = self.new_value_id();
                
                self.emit(MirInstruction::BuiltinCall {
                    qualified_name: method_info.qualified_name(),
                    args,
                    result,
                    effect: method_info.effect(),
                });
                
                Ok(Some(result))
            },
            
            ResolvedCall::BidCall { interface_name, method_name, method_info, .. } => {
                let result = if method_info.returns == BidType::Void {
                    None
                } else {
                    Some(self.new_value_id())
                };
                
                self.emit(MirInstruction::ExternCall {
                    interface: interface_name,
                    method: method_name,
                    args,
                    result,
                    effect: self.bid_effect_to_mir_effect(&method_info.effect),
                    bid_signature: BidSignature::from_method(&method_info),
                });
                
                Ok(result)
            },
            
            ResolvedCall::ModuleCall { module_name, function_name, function_info, .. } => {
                let result = self.new_value_id();
                
                self.emit(MirInstruction::ModuleCall {
                    module: module_name,
                    function: function_name,
                    args,
                    result,
                    effect: Effect::Io, // Nyashモジュールはデフォルトでio
                });
                
                Ok(Some(result))
            },
        }
    }
    
    fn bid_effect_to_mir_effect(&self, bid_effect: &BidEffect) -> Effect {
        match bid_effect {
            BidEffect::Pure => Effect::Pure,
            BidEffect::Mut => Effect::Mut,
            BidEffect::Io => Effect::Io,
            BidEffect::Control => Effect::Control,
        }
    }
}
```

### 4. バックエンド統合

#### **WASM生成統合**
```rust
// src/backend/wasm/codegen.rs拡張
impl WasmCodegen {
    pub fn generate_unified_call(&mut self, instruction: &MirInstruction) 
        -> Result<(), WasmError> {
        
        match instruction {
            MirInstruction::ExternCall { interface, method, args, bid_signature, .. } => {
                // BIDから自動生成されたWASM import名
                let wasm_import_name = self.bid_to_wasm_import_name(interface, method);
                
                // 引数の型変換・マーシャリング
                let marshalled_args = self.marshal_args_for_wasm(args, &bid_signature.params)?;
                
                // WASM関数呼び出し生成
                self.emit_call(&wasm_import_name, &marshalled_args)?;
                
                // 戻り値のアンマーシャリング
                if bid_signature.returns != BidType::Void {
                    self.unmarshal_return_value(&bid_signature.returns)?;
                }
                
                Ok(())
            },
            
            // 他の命令は既存実装
            _ => self.generate_instruction_legacy(instruction),
        }
    }
    
    fn bid_to_wasm_import_name(&self, interface: &str, method: &str) -> String {
        // "canvas_api.canvas" + "fillRect" → "canvas_api_canvas_fillRect"
        format!("{}_{}", interface.replace(".", "_"), method)
    }
    
    fn marshal_args_for_wasm(&mut self, args: &[ValueId], params: &[BidParam]) 
        -> Result<Vec<WasmValue>, WasmError> {
        
        let mut marshalled = Vec::new();
        
        for (i, param) in params.iter().enumerate() {
            let arg_value = self.get_value(args[i])?;
            
            match &param.param_type {
                BidType::String => {
                    // 文字列を (ptr, len) にマーシャル
                    let (ptr, len) = self.string_to_wasm_memory(&arg_value)?;
                    marshalled.push(WasmValue::I32(ptr));
                    marshalled.push(WasmValue::I32(len));
                },
                BidType::I32 => {
                    marshalled.push(WasmValue::I32(arg_value.to_i32()?));
                },
                BidType::F64 => {
                    marshalled.push(WasmValue::F64(arg_value.to_f64()?));
                },
                // その他の型...
            }
        }
        
        Ok(marshalled)
    }
}
```

#### **VM実行統合**
```rust
// src/backend/vm.rs拡張
impl VmBackend {
    pub fn execute_unified_instruction(&mut self, instruction: &MirInstruction) 
        -> Result<(), VmError> {
        
        match instruction {
            MirInstruction::ExternCall { interface, method, args, bid_signature, .. } => {
                // VM環境ではスタブまたはネイティブ呼び出し
                let evaluated_args = self.evaluate_args(args)?;
                
                if let Some(native_impl) = self.find_native_implementation(interface, method) {
                    // ネイティブ実装がある場合（例：ファイルI/O）
                    let result = native_impl.call(evaluated_args, bid_signature)?;
                    if let Some(result_id) = &instruction.result {
                        self.set_value(*result_id, result);
                    }
                } else {
                    // スタブ実装（ログ出力等）
                    self.execute_stub_call(interface, method, evaluated_args, bid_signature)?;
                }
                
                Ok(())
            },
            
            // 他の命令は既存実装
            _ => self.execute_instruction_legacy(instruction),
        }
    }
    
    fn find_native_implementation(&self, interface: &str, method: &str) 
        -> Option<&dyn NativeImplementation> {
        
        // VM環境で利用可能なネイティブ実装を検索
        match (interface, method) {
            ("env.console", "log") => Some(&self.console_impl),
            ("env.filesystem", "read") => Some(&self.filesystem_impl),
            ("env.filesystem", "write") => Some(&self.filesystem_impl),
            _ => None,
        }
    }
}
```

## 🧪 統合テスト戦略

### Phase別テスト実装

#### **Phase 0: 基本統合テスト**
```nyash
# test_basic_integration.nyash
using nyashstd

# 組み込み標準ライブラリのみ
assert(string.upper("test") == "TEST")
assert(math.sin(0) == 0)
```

#### **Phase 1: BID統合テスト**
```nyash
# test_bid_integration.nyash  
using nyashstd
using console_api

# 組み込み + FFI-ABI
string.upper("hello")     # 組み込み
console.log("Testing")    # FFI-ABI
```

#### **Phase 2: 完全統合テスト**
```nyash
# test_full_integration.nyash
using nyashstd
using console_api
using mylib

# 3種類すべて
string.upper("test")         # 組み込み
console.log("Integration")   # FFI-ABI  
mylib.process("data")        # Nyashモジュール
```

### エラーハンドリングテスト
```nyash
# test_error_handling.nyash
try {
    using nonexistent_api
} catch error {
    assert(error.type == "UndefinedNamespace")
}

try {
    console.nonexistent_method("test")
} catch error {
    assert(error.type == "UndefinedMethod")
    assert(error.message.contains("Available methods:"))
}
```

## 📊 実装マイルストーン

### ✅ Phase 0完了条件
- [ ] UniversalNamespaceRegistry基盤実装
- [ ] 組み込み標準ライブラリ統合
- [ ] 基本using文処理
- [ ] MIR BuiltinCall生成

### ✅ Phase 1完了条件  
- [ ] BID定義読み込み・検証
- [ ] BID→MIR ExternCall統合
- [ ] WASM RuntimeImports自動生成
- [ ] VM スタブ実行

### ✅ Phase 2完了条件
- [ ] Nyashモジュール統合
- [ ] 統合エラーハンドリング
- [ ] 最適化キャッシュ
- [ ] 全バックエンド対応

---

**🎯 この詳細実装により、BIDとusingシステムの完全統合が実現でき、「なんでもAPI計画」の技術基盤が完成するにゃ！🚀🐱**