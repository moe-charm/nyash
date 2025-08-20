# なんでもAPI計画：nyash.link × FFI-ABI × MIR 統合設計

## 🌟 革命的統合ビジョン

### 📊 現状把握
- ✅ **nyash.linkシステム**: 標準関数・モジュール管理設計完了
- ✅ **FFI-ABI仕様**: BID(Box Interface Definition)による外部API統一
- ✅ **MIR ExternCall**: 外部関数呼び出しのMIRレベル実装
- 🎯 **統合目標**: 3つのシステムを統合し「なんでもAPI」を実現

### 🚀 統合後の開発体験
```nyash
# === 単一のusing構文ですべてが使える！ ===
using nyashstd        # 組み込み標準ライブラリ
using console_api     # ブラウザConsole API (FFI-ABI)
using canvas_api      # Canvas API (FFI-ABI)
using opencv_api      # OpenCV外部ライブラリ (FFI-ABI)
using mylib          # 自作Nyashモジュール

# 全部同じ記法で使える！
string.upper("hello")                    # 組み込み標準ライブラリ
console.log("Hello Nyash!")              # ブラウザAPI
canvas.fillRect("game", 10, 10, 80, 60, "red")  # Canvas API
opencv.loadImage("photo.jpg")            # 外部ライブラリ
mylib.processData("input")               # 自作モジュール
```

## 🏗️ 統合アーキテクチャ設計

### 1. 拡張nyash.link仕様

#### **依存関係タイプの統合**
```toml
# nyash.link - 全API統一管理
[project]
name = "awesome-nyash-app"
version = "1.0.0"

[dependencies]
# === 組み込み標準ライブラリ ===
nyashstd = { builtin = true }

# === FFI-ABI経由外部API ===
console_api = { bid = "./apis/console.yaml" }
canvas_api = { bid = "./apis/canvas.yaml" }
webgl_api = { bid = "./apis/webgl.yaml" }
dom_api = { bid = "./apis/dom.yaml" }

# === システムライブラリ ===
libc = { bid = "./apis/libc.yaml", library = "system" }
math_lib = { bid = "./apis/math.yaml", library = "libm" }

# === 外部共有ライブラリ ===
opencv = { bid = "./apis/opencv.yaml", library = "./libs/opencv.so" }
sqlite = { bid = "./apis/sqlite.yaml", library = "./libs/sqlite.so" }

# === Nyashモジュール（従来通り） ===
mylib = { path = "./src/mylib.nyash" }
utils = { path = "./src/utils.nyash" }
models = { path = "./src/models/" }

# === 将来の外部パッケージ ===
# http_client = { version = "1.0.0", registry = "nyash-pkg" }

[build]
entry_point = "./src/main.nyash"
backends = ["vm", "wasm", "aot"]  # 対象バックエンド指定
```

#### **BIDファイル例**
```yaml
# apis/console.yaml - Console API定義
version: 0
metadata:
  name: "Browser Console API"
  description: "Standard browser console interface"
  target_environments: ["browser", "node"]

interfaces:
  - name: console_api.console
    box: Console
    namespace: console_api
    methods:
      - name: log
        params: [ {string: message} ]
        returns: void
        effect: io
        description: "Output message to console"
        
      - name: warn
        params: [ {string: message} ]
        returns: void
        effect: io
        
      - name: error
        params: [ {string: message} ]
        returns: void
        effect: io

# apis/canvas.yaml - Canvas API定義
version: 0
interfaces:
  - name: canvas_api.canvas
    box: Canvas
    namespace: canvas_api
    methods:
      - name: fillRect
        params:
          - {string: canvas_id}
          - {i32: x}
          - {i32: y} 
          - {i32: width}
          - {i32: height}
          - {string: color}
        returns: void
        effect: io
        
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
```

### 2. 統合名前空間レジストリ

#### **UniversalNamespaceRegistry設計**
```rust
// 新ファイル: src/registry/universal.rs
use crate::stdlib::BuiltinStdlib;
use crate::bid::BidDefinition;
use crate::module::ExternalModule;

pub struct UniversalNamespaceRegistry {
    /// 組み込み標準ライブラリ
    builtin: Arc<BuiltinStdlib>,
    
    /// FFI-ABI経由の外部API
    ffi_apis: HashMap<String, Arc<BidDefinition>>,
    
    /// Nyashモジュール
    nyash_modules: HashMap<String, Arc<ExternalModule>>,
    
    /// using imports（ファイル別）
    using_imports: Arc<RwLock<HashMap<String, UsingContext>>>,
}

#[derive(Debug, Clone)]
pub struct UsingContext {
    pub builtin_imports: Vec<String>,     // ["nyashstd"]
    pub ffi_imports: Vec<String>,         // ["console_api", "canvas_api"]  
    pub module_imports: Vec<String>,      // ["mylib", "utils"]
    pub file_id: String,
}

impl UniversalNamespaceRegistry {
    pub fn new(nyash_link: &NyashLink) -> Result<Self, RegistryError> {
        let mut registry = UniversalNamespaceRegistry {
            builtin: Arc::new(BuiltinStdlib::new()),
            ffi_apis: HashMap::new(),
            nyash_modules: HashMap::new(),
            using_imports: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // nyash.linkからFFI-ABI定義読み込み
        registry.load_ffi_apis(nyash_link)?;
        
        // Nyashモジュール読み込み
        registry.load_nyash_modules(nyash_link)?;
        
        Ok(registry)
    }
    
    /// 統合using文処理
    pub fn execute_using(&mut self, namespace_name: &str, file_id: &str) 
        -> Result<(), RuntimeError> {
        
        let context = self.using_imports
            .write().unwrap()
            .entry(file_id.to_string())
            .or_insert_with(|| UsingContext {
                builtin_imports: Vec::new(),
                ffi_imports: Vec::new(),
                module_imports: Vec::new(),
                file_id: file_id.to_string(),
            });
        
        // 組み込み標準ライブラリ
        if self.builtin.has_namespace(namespace_name) {
            if !context.builtin_imports.contains(&namespace_name.to_string()) {
                context.builtin_imports.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        // FFI-ABI API
        if self.ffi_apis.contains_key(namespace_name) {
            if !context.ffi_imports.contains(&namespace_name.to_string()) {
                context.ffi_imports.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        // Nyashモジュール
        if self.nyash_modules.contains_key(namespace_name) {
            if !context.module_imports.contains(&namespace_name.to_string()) {
                context.module_imports.push(namespace_name.to_string());
            }
            return Ok(());
        }
        
        Err(RuntimeError::UndefinedNamespace(namespace_name.to_string()))
    }
    
    /// 統合関数解決
    pub fn resolve_call(&self, file_id: &str, path: &[String]) 
        -> Result<CallTarget, RuntimeError> {
        
        if path.len() != 2 { 
            return Err(RuntimeError::InvalidQualifiedName(path.join(".")));
        }
        
        let box_name = &path[0];
        let method_name = &path[1];
        
        if let Ok(imports) = self.using_imports.read() {
            if let Some(context) = imports.get(file_id) {
                
                // 1. 組み込み標準ライブラリ検索
                for namespace in &context.builtin_imports {
                    if let Some(target) = self.builtin.resolve_call(namespace, box_name, method_name) {
                        return Ok(CallTarget::Builtin(target));
                    }
                }
                
                // 2. FFI-ABI API検索
                for namespace in &context.ffi_imports {
                    if let Some(bid) = self.ffi_apis.get(namespace) {
                        if let Some(target) = bid.resolve_method(box_name, method_name) {
                            return Ok(CallTarget::FfiAbi(target));
                        }
                    }
                }
                
                // 3. Nyashモジュール検索
                for namespace in &context.module_imports {
                    if let Some(module) = self.nyash_modules.get(namespace) {
                        if let Some(target) = module.resolve_method(box_name, method_name) {
                            return Ok(CallTarget::NyashModule(target));
                        }
                    }
                }
            }
        }
        
        Err(RuntimeError::UndefinedMethod(format!("{}.{}", box_name, method_name)))
    }
}

#[derive(Debug)]
pub enum CallTarget {
    Builtin(BuiltinMethodTarget),
    FfiAbi(FfiMethodTarget),
    NyashModule(NyashMethodTarget),
}
```

### 3. MIRレベル統合

#### **MIR命令拡張**
```rust
// src/mir/instruction.rs拡張
#[derive(Debug, Clone)]
pub enum MirInstruction {
    // 既存命令...
    
    // === 統合関数呼び出し ===
    
    /// 組み込み標準ライブラリ呼び出し
    BuiltinCall {
        target: String,          // "string.upper"
        args: Vec<ValueId>,
        result: ValueId,
        effect: Effect,
    },
    
    /// FFI-ABI外部API呼び出し  
    ExternCall {
        interface: String,       // "console_api.console"
        method: String,          // "log"
        args: Vec<ValueId>,
        result: Option<ValueId>,
        effect: Effect,
        bid_signature: BidMethodSignature,
    },
    
    /// Nyashモジュール関数呼び出し
    ModuleCall {
        module: String,          // "mylib"
        function: String,        // "processData"
        args: Vec<ValueId>,
        result: ValueId,
        effect: Effect,
    },
}

#[derive(Debug, Clone)]
pub enum Effect {
    Pure,      // 副作用なし、並び替え可能
    Mut,       // 同リソース内で順序保持
    Io,        // プログラム順序保持
    Control,   // 制御フロー影響
}
```

#### **MIR生成統合**
```rust
// src/mir/builder.rs拡張
impl MirBuilder {
    pub fn build_unified_call(&mut self, target: CallTarget, args: Vec<ValueId>) 
        -> Result<ValueId, MirError> {
        
        match target {
            CallTarget::Builtin(builtin_target) => {
                let result = self.new_value_id();
                self.emit(MirInstruction::BuiltinCall {
                    target: builtin_target.qualified_name(),
                    args,
                    result,
                    effect: builtin_target.effect(),
                });
                Ok(result)
            },
            
            CallTarget::FfiAbi(ffi_target) => {
                let result = if ffi_target.returns_void() {
                    None
                } else {
                    Some(self.new_value_id())
                };
                
                self.emit(MirInstruction::ExternCall {
                    interface: ffi_target.interface_name(),
                    method: ffi_target.method_name(),
                    args,
                    result,
                    effect: ffi_target.effect(),
                    bid_signature: ffi_target.signature().clone(),
                });
                
                result.ok_or(MirError::VoidReturn)
            },
            
            CallTarget::NyashModule(module_target) => {
                let result = self.new_value_id();
                self.emit(MirInstruction::ModuleCall {
                    module: module_target.module_name(),
                    function: module_target.function_name(),
                    args,
                    result,
                    effect: Effect::Io, // デフォルト
                });
                Ok(result)
            },
        }
    }
}
```

### 4. バックエンド統合実装

#### **VM実行統合**
```rust
// src/backend/vm.rs拡張
impl VmBackend {
    pub fn execute_instruction(&mut self, instr: &MirInstruction) 
        -> Result<(), VmError> {
        
        match instr {
            MirInstruction::BuiltinCall { target, args, result, .. } => {
                let evaluated_args = self.evaluate_args(args)?;
                let output = self.builtin_executor.call(target, evaluated_args)?;
                self.set_value(*result, output);
                Ok(())
            },
            
            MirInstruction::ExternCall { interface, method, args, result, bid_signature, .. } => {
                // VM環境ではスタブ実装
                let evaluated_args = self.evaluate_args(args)?;
                let output = self.extern_stub.call(interface, method, evaluated_args, bid_signature)?;
                if let Some(res_id) = result {
                    self.set_value(*res_id, output);
                }
                Ok(())
            },
            
            MirInstruction::ModuleCall { module, function, args, result, .. } => {
                let evaluated_args = self.evaluate_args(args)?;
                let output = self.module_executor.call(module, function, evaluated_args)?;
                self.set_value(*result, output);
                Ok(())
            },
            
            // 既存命令処理...
        }
    }
}
```

#### **WASM生成統合**
```rust
// src/backend/wasm/codegen.rs拡張
impl WasmCodegen {
    pub fn generate_instruction(&mut self, instr: &MirInstruction) 
        -> Result<(), WasmError> {
        
        match instr {
            MirInstruction::BuiltinCall { target, args, result, .. } => {
                // 組み込み関数は直接実装
                self.generate_builtin_call(target, args, *result)
            },
            
            MirInstruction::ExternCall { interface, method, args, bid_signature, .. } => {
                // BIDから自動生成されたWASM import呼び出し
                let import_name = format!("{}_{}", 
                    interface.replace(".", "_"),
                    method
                );
                
                self.generate_extern_call(&import_name, args, bid_signature)
            },
            
            MirInstruction::ModuleCall { module, function, args, result, .. } => {
                // 内部関数呼び出し
                let function_name = format!("{}_{}", module, function);
                self.generate_function_call(&function_name, args, *result)
            },
        }
    }
    
    /// BIDからWASM RuntimeImports自動生成
    pub fn generate_runtime_imports(&mut self, bid_definitions: &[BidDefinition]) 
        -> Result<String, WasmError> {
        
        let mut imports = String::new();
        
        for bid in bid_definitions {
            for interface in &bid.interfaces {
                for method in &interface.methods {
                    let import_name = format!("{}_{}", 
                        interface.name.replace(".", "_"),
                        method.name
                    );
                    
                    let signature = self.bid_to_wasm_signature(&method.params, &method.returns)?;
                    imports.push_str(&format!(
                        "(import \"env\" \"{}\" {})\n",
                        import_name, signature
                    ));
                }
            }
        }
        
        Ok(imports)
    }
}
```

#### **AOT生成統合**
```rust
// src/backend/aot/compiler.rs拡張
impl AotCompiler {
    pub fn compile_instruction(&mut self, instr: &MirInstruction) 
        -> Result<(), AotError> {
        
        match instr {
            MirInstruction::ExternCall { interface, method, args, bid_signature, .. } => {
                // LLVM IR外部関数宣言生成
                let extern_func_name = format!("{}_{}", 
                    interface.replace(".", "_"),
                    method
                );
                
                let signature = self.bid_to_llvm_signature(bid_signature)?;
                self.declare_external_function(&extern_func_name, &signature)?;
                self.generate_call(&extern_func_name, args)?;
                
                Ok(())
            },
            
            // その他の命令処理...
        }
    }
}
```

## 🎯 段階的実装戦略

### Phase 0: 基盤統合（2-3週間）
1. **UniversalNamespaceRegistry実装** - 全API統一管理
2. **nyash.link拡張** - BID依存関係サポート
3. **統合using文** - 3種類のAPI統一インポート

### Phase 1: FFI-ABI統合（3-4週間）
1. **BID読み込み機能** - YAML解析・検証
2. **MIR ExternCall統合** - FFI-ABI→MIR変換
3. **WASM RuntimeImports自動生成** - BID→WASM import

### Phase 2: 完全統合（4-6週間）
1. **全バックエンド対応** - VM/WASM/AOT統合実装
2. **エラーハンドリング統合** - 統一エラーモデル
3. **パフォーマンス最適化** - 高速名前解決

## 🧪 統合テスト戦略

### 基本統合テスト
```nyash
# test_universal_integration.nyash
using nyashstd
using console_api
using mylib

# 3種類のAPIが同じように使える
assert(string.upper("test") == "TEST")           # 組み込み
console.log("Integration test successful")       # FFI-ABI
assert(mylib.process("data") == "processed")     # Nyash
```

### FFI-ABI統合テスト
```nyash
# test_ffi_abi_integration.nyash
using canvas_api

# Canvas API経由での描画
canvas.fillRect("game-canvas", 10, 10, 100, 100, "red")
canvas.fillText("game-canvas", "Score: 100", 10, 30, "16px Arial", "white")
```

## 🌟 期待される革命的効果

### 🚀 開発者体験
- **統一API**: 組み込み・外部・自作すべて同じ書き方
- **IDE補完**: すべてのAPIが`ny`で補完される
- **エラー処理**: 統一エラーモデルで一貫性

### 🏗️ アーキテクチャ
- **MIRレベル統合**: 全バックエンドで同じパフォーマンス最適化
- **Effect System**: pure/mut/io/controlによる安全性保証
- **言語非依存**: BIDによる外部ライブラリ標準化

### 🌍 エコシステム
- **なんでもAPI**: あらゆる外部ライブラリがNyashから使える
- **バックエンド統一**: 同じコードがVM/WASM/AOTで動作
- **将来拡張**: パッケージレジストリでBID共有

---

**🎉 この統合設計で、Nyashが真に「なんでもできる」モダン言語になるにゃ！🚀🐱**