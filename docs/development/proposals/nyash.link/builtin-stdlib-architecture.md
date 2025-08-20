# 組み込みnyashstd名前空間アーキテクチャ設計

## 🏗️ 技術的実装アーキテクチャ

### 📊 現在のインタープリター構造分析

#### **NyashInterpreter構造**
```rust
pub struct NyashInterpreter {
    pub(super) shared: SharedState,           // 共有状態
    pub(super) local_vars: HashMap<String, SharedNyashBox>,
    pub(super) outbox_vars: HashMap<String, SharedNyashBox>,
    // その他の制御フロー状態...
}
```

#### **設計判断：SharedStateに組み込み**
- **理由**: 標準ライブラリは不変・全インタープリターで共有可能
- **利点**: メモリ効率、パフォーマンス向上
- **実装**: SharedStateに`builtin_stdlib`フィールド追加

## 🌟 最適化されたアーキテクチャ設計

### 1. SharedState拡張

#### **src/interpreter/core.rs**
```rust
#[derive(Clone)]
pub struct SharedState {
    // 既存フィールド...
    pub global_vars: Arc<RwLock<HashMap<String, SharedNyashBox>>>,
    pub functions: Arc<RwLock<HashMap<String, Function>>>,
    pub box_definitions: Arc<RwLock<HashMap<String, Box<UserDefinedBoxDefinition>>>>,
    pub loop_counter: Arc<AtomicU64>,
    pub included_files: Arc<RwLock<HashSet<String>>>,
    
    // 🌟 新規追加: 組み込み標準ライブラリ
    pub builtin_stdlib: Arc<BuiltinStdlib>,
    pub using_imports: Arc<RwLock<HashMap<String, UsingContext>>>, // ファイル別インポート管理
}

#[derive(Debug, Clone)]
pub struct UsingContext {
    pub imported_namespaces: Vec<String>,  // ["nyashstd"] 
    pub file_id: String,                   // インポート元ファイル識別
}
```

### 2. BuiltinStdlib効率化設計

#### **新ファイル: src/stdlib/builtin.rs**
```rust
//! 🚀 高性能組み込み標準ライブラリ
//! 
//! 設計方針:
//! - Zero-allocation関数実行
//! - 高速名前解決
//! - 既存Box実装の最大活用

use crate::boxes::*;
use std::collections::HashMap;

/// 組み込み標準ライブラリのメイン構造体
#[derive(Debug)]
pub struct BuiltinStdlib {
    /// 高速アクセス用：フラットな関数マップ
    /// "string.upper" -> BuiltinFunction
    pub flat_functions: HashMap<String, BuiltinFunction>,
    
    /// IDE補完用：階層構造
    /// "nyashstd" -> { "string" -> ["upper", "lower", ...] }
    pub hierarchical_map: HashMap<String, HashMap<String, Vec<String>>>,
}

/// 組み込み関数の実装
pub struct BuiltinFunction {
    pub namespace: &'static str,    // "nyashstd"
    pub box_name: &'static str,     // "string"  
    pub method_name: &'static str,  // "upper"
    pub implementation: BuiltinMethodImpl,
    pub arg_count: Option<usize>,   // None = 可変長
    pub description: &'static str,  // エラーメッセージ・ヘルプ用
}

/// 高性能関数実装
pub type BuiltinMethodImpl = fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError>;

impl BuiltinStdlib {
    /// 🚀 標準ライブラリ初期化（起動時1回のみ）
    pub fn new() -> Self {
        let mut stdlib = BuiltinStdlib {
            flat_functions: HashMap::new(),
            hierarchical_map: HashMap::new(),
        };
        
        // 標準関数登録
        stdlib.register_all_functions();
        
        stdlib
    }
    
    /// ⚡ 高速関数解決
    pub fn get_function(&self, qualified_name: &str) -> Option<&BuiltinFunction> {
        // "string.upper" で直接アクセス
        self.flat_functions.get(qualified_name)
    }
    
    /// 🔍 IDE補完用：利用可能関数一覧取得
    pub fn get_available_methods(&self, namespace: &str, box_name: &str) -> Option<&Vec<String>> {
        self.hierarchical_map.get(namespace)?.get(box_name)
    }
    
    /// 📋 全名前空間取得（IDE補完用）
    pub fn get_all_namespaces(&self) -> Vec<&String> {
        self.hierarchical_map.keys().collect()
    }
}
```

### 3. 標準関数実装（高性能版）

#### **文字列関数実装**
```rust
impl BuiltinStdlib {
    fn register_all_functions(&mut self) {
        // === nyashstd.string.* ===
        self.register_function("string.upper", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string", 
            method_name: "upper",
            implementation: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.upper(str) takes exactly 1 argument".to_string()
                    ));
                }
                
                // 🚀 既存StringBox実装活用
                let input_str = args[0].to_string_box().value;
                let result = StringBox::new(&input_str.to_uppercase());
                Ok(Box::new(result))
            },
            arg_count: Some(1),
            description: "Convert string to uppercase",
        });
        
        self.register_function("string.lower", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string",
            method_name: "lower", 
            implementation: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.lower(str) takes exactly 1 argument".to_string()
                    ));
                }
                
                let input_str = args[0].to_string_box().value;
                let result = StringBox::new(&input_str.to_lowercase());
                Ok(Box::new(result))
            },
            arg_count: Some(1),
            description: "Convert string to lowercase",
        });
        
        self.register_function("string.split", BuiltinFunction {
            namespace: "nyashstd", 
            box_name: "string",
            method_name: "split",
            implementation: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.split(str, separator) takes exactly 2 arguments".to_string()
                    ));
                }
                
                // 🚀 既存StringBox.split()メソッド活用
                let string_box = StringBox::new(&args[0].to_string_box().value);
                let separator = &args[1].to_string_box().value;
                string_box.split(separator)
            },
            arg_count: Some(2),
            description: "Split string by separator into array",
        });
        
        // === nyashstd.math.* ===
        self.register_function("math.sin", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "math",
            method_name: "sin",
            implementation: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArguments(
                        "math.sin(x) takes exactly 1 argument".to_string()
                    ));
                }
                
                // 🚀 既存MathBox実装活用
                let math_box = MathBox::new();
                let x = args[0].to_integer_box().value as f64;
                let result = math_box.sin(x)?;
                Ok(result)
            },
            arg_count: Some(1),
            description: "Calculate sine of x (in radians)",
        });
        
        // 階層マップも同時構築
        self.build_hierarchical_map();
    }
    
    fn register_function(&mut self, qualified_name: &str, function: BuiltinFunction) {
        self.flat_functions.insert(qualified_name.to_string(), function);
    }
    
    fn build_hierarchical_map(&mut self) {
        for (qualified_name, function) in &self.flat_functions {
            let namespace_map = self.hierarchical_map
                .entry(function.namespace.to_string())
                .or_insert_with(HashMap::new);
                
            let method_list = namespace_map
                .entry(function.box_name.to_string())
                .or_insert_with(Vec::new);
                
            method_list.push(function.method_name.to_string());
        }
        
        // ソートして一貫性確保
        for namespace_map in self.hierarchical_map.values_mut() {
            for method_list in namespace_map.values_mut() {
                method_list.sort();
            }
        }
    }
}
```

### 4. インタープリター統合

#### **NyashInterpreter拡張**
```rust
impl NyashInterpreter {
    /// using文実行
    pub fn execute_using(&mut self, namespace_name: &str) -> Result<(), RuntimeError> {
        // 組み込み名前空間存在チェック
        if !self.shared.builtin_stdlib.hierarchical_map.contains_key(namespace_name) {
            return Err(RuntimeError::UndefinedNamespace(namespace_name.to_string()));
        }
        
        // 現在ファイルのusingコンテキスト更新
        let file_id = self.get_current_file_id();
        let mut using_imports = self.shared.using_imports.write().unwrap();
        
        let context = using_imports.entry(file_id.clone()).or_insert(UsingContext {
            imported_namespaces: Vec::new(),
            file_id: file_id.clone(),
        });
        
        if !context.imported_namespaces.contains(&namespace_name.to_string()) {
            context.imported_namespaces.push(namespace_name.to_string());
        }
        
        Ok(())
    }
    
    /// ⚡ 高速名前解決：string.upper() → nyashstd.string.upper()
    pub fn resolve_qualified_call(&self, path: &[String]) -> Option<String> {
        if path.len() != 2 {
            return None; // Phase 0では2段階のみ対応
        }
        
        let box_name = &path[0];
        let method_name = &path[1];
        let file_id = self.get_current_file_id();
        
        // 現在ファイルのusingインポート確認
        if let Ok(using_imports) = self.shared.using_imports.read() {
            if let Some(context) = using_imports.get(&file_id) {
                for namespace in &context.imported_namespaces {
                    let qualified_name = format!("{}.{}", box_name, method_name);
                    
                    // 実際に関数が存在するかチェック
                    if self.shared.builtin_stdlib.get_function(&qualified_name).is_some() {
                        return Some(qualified_name);
                    }
                }
            }
        }
        
        None
    }
    
    /// 🚀 組み込み関数実行
    pub fn call_builtin_function(&self, qualified_name: &str, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        if let Some(function) = self.shared.builtin_stdlib.get_function(qualified_name) {
            // 引数数チェック
            if let Some(expected_count) = function.arg_count {
                if args.len() != expected_count {
                    return Err(RuntimeError::InvalidArguments(
                        format!("{}.{}() takes exactly {} arguments, got {}", 
                            function.box_name, function.method_name, 
                            expected_count, args.len())
                    ));
                }
            }
            
            // 関数実行
            (function.implementation)(&args)
        } else {
            Err(RuntimeError::UndefinedMethod(qualified_name.to_string()))
        }
    }
}
```

### 5. 式実行統合

#### **src/interpreter/expressions.rs修正**
```rust
impl NyashInterpreter {
    pub fn execute_expression(&mut self, node: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match node {
            // 既存のケース...
            
            // メソッド呼び出し処理修正
            ASTNode::MethodCall { object, method, args, .. } => {
                // オブジェクトが単純な識別子かチェック
                if let ASTNode::Variable { name: box_name, .. } = object.as_ref() {
                    // using経由での短縮呼び出しチェック
                    let path = vec![box_name.clone(), method.clone()];
                    if let Some(qualified_name) = self.resolve_qualified_call(&path) {
                        // 引数評価
                        let evaluated_args = self.evaluate_arguments(args)?;
                        // 組み込み関数実行
                        return self.call_builtin_function(&qualified_name, evaluated_args);
                    }
                }
                
                // 既存のメソッド呼び出し処理
                // ...
            }
            
            // using文実行
            ASTNode::UsingStatement { namespace_name, .. } => {
                self.execute_using(namespace_name)?;
                Ok(Box::new(VoidBox::new()))
            }
            
            // 他の既存ケース...
        }
    }
}
```

## 📊 パフォーマンス特性

### ⚡ 最適化ポイント

#### **1. Zero-Allocation関数解決**
```rust
// ❌ 遅い：毎回文字列生成
let qualified = format!("{}.{}", box_name, method_name);

// ✅ 高速：事前計算済みマップ
if let Some(func) = stdlib.flat_functions.get(&qualified_name) { ... }
```

#### **2. 高速名前解決**
```rust
// O(1)アクセス：HashMap直接ルックアップ
// "string.upper" -> BuiltinFunction
```

#### **3. 既存Box実装活用**
```rust
// 既存の最適化済みStringBox.split()を直接使用
string_box.split(separator)  // 新規実装不要
```

## 🧪 テストカバレッジ

### Phase 0必須テスト

#### **基本機能テスト**
```nyash
# test_builtin_stdlib_basic.nyash
using nyashstd

# 文字列操作
assert(string.upper("hello") == "HELLO")
assert(string.lower("WORLD") == "world") 
assert(string.split("a,b,c", ",").length() == 3)

# 数学関数
assert(math.sin(0) == 0)
assert(math.cos(0) == 1)

# 配列操作
local arr = [1, 2, 3]
assert(array.length(arr) == 3)
assert(array.get(arr, 1) == 2)
```

#### **エラーハンドリング**
```nyash
# test_builtin_stdlib_errors.nyash
using nyashstd

# 引数数エラー
try {
    string.upper("hello", "extra")  # 2引数でエラー
    assert(false, "Should have thrown error")
} catch e {
    assert(e.contains("takes exactly 1 argument"))
}

# 未定義名前空間
try {
    using nonexistent
    assert(false, "Should have thrown error")
} catch e {
    assert(e.contains("UndefinedNamespace"))
}
```

#### **IDE補完サポート**
```rust
// テスト：補完候補取得
let methods = stdlib.get_available_methods("nyashstd", "string");
assert!(methods.unwrap().contains(&"upper".to_string()));
assert!(methods.unwrap().contains(&"lower".to_string()));
```

## 🎯 実装順序

### 🚨 Critical（即時実装）
1. **BuiltinStdlib基盤** - src/stdlib/builtin.rs作成
2. **SharedState統合** - builtin_stdlibフィールド追加  
3. **using文パーサー** - ASTNode::UsingStatement

### ⚡ High（今週中）
4. **string関数4種** - upper, lower, split, join
5. **基本テスト** - using nyashstd動作確認
6. **エラーハンドリング** - 適切なエラーメッセージ

### 📝 Medium（来週）
7. **math関数5種** - sin, cos, sqrt, floor, random
8. **array関数4種** - length, get, push, slice
9. **io関数3種** - print, println, debug

---

**⚡ この高性能アーキテクチャで、複雑なファイル依存関係なしに即座に実用的なnamespace/usingが実現できるにゃ！🚀**