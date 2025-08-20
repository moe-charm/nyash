# 最小実装：標準関数優先namespace/usingシステム

## 🎯 基本戦略：nyash.link前の段階的実装

### 📊 現状分析
- **既存Box型**: 25種類以上の豊富なBox実装
- **include使用**: 限定的（text_adventure例のみ）
- **using実装**: 完全未実装→新規作成可能
- **最優先課題**: 複雑なファイル依存関係システムより、まず標準関数のIDE補完

### 🌟 段階的実装アプローチ

#### **Phase 0: 組み込みnyashstd（最小実装）**
```
ファイル読み込み一切なし → インタープリターに直接組み込み
```

#### **Phase 1: using構文**
```nyash
using nyashstd
string.upper("hello")  # ✅ 動作
```

#### **Phase 2: 将来のnyash.link対応**
```
外部ファイル・依存関係システム（後日実装）
```

## 🏗️ 組み込みnyashstd設計

### 優先順位別Box分類

#### 🚨 **Tier 1: 最優先基本機能**
```rust
// 使用頻度最高・IDE補完必須
- string_box.rs    → nyashstd.string.*
- math_box.rs      → nyashstd.math.*  
- array/mod.rs     → nyashstd.array.*
- console_box.rs   → nyashstd.io.*
```

#### ⚡ **Tier 2: 重要機能**
```rust
// 標準的な機能
- time_box.rs      → nyashstd.time.*
- random_box.rs    → nyashstd.random.*
- map_box.rs       → nyashstd.map.*
```

#### 📝 **Tier 3: 特殊用途**
```rust
// 特定用途・後で追加
- debug_box.rs     → nyashstd.debug.*
- http_server_box.rs → nyashstd.http.*
- p2p_box.rs       → nyashstd.p2p.*
```

### 最小実装スコープ（Phase 0）

#### **nyashstd.string機能**
```nyash
using nyashstd

string.upper("hello")      # "HELLO"
string.lower("WORLD")      # "world"
string.split("a,b,c", ",") # ["a", "b", "c"]
string.join(["a","b"], "-") # "a-b"
string.length("test")      # 4
```

#### **nyashstd.math機能**
```nyash
using nyashstd

math.sin(3.14159)    # 0.0 (approximately)
math.cos(0)          # 1.0
math.sqrt(16)        # 4.0
math.floor(3.7)      # 3
math.random()        # 0.0-1.0のランダム値
```

#### **nyashstd.array機能**
```nyash
using nyashstd

array.length([1,2,3])          # 3
array.push([1,2], 3)           # [1,2,3]
array.get([1,2,3], 1)          # 2
array.slice([1,2,3,4], 1, 3)   # [2,3]
```

#### **nyashstd.io機能**
```nyash
using nyashstd

io.print("Hello")              # コンソール出力
io.println("World")            # 改行付き出力
io.debug("Debug info")         # デバッグ出力
```

## 💻 技術実装戦略

### 1. インタープリター組み込み方式

#### **新ファイル: `src/stdlib/mod.rs`**
```rust
//! 組み込み標準ライブラリ
//! nyash.linkなしで動作する基本的な標準関数群

use crate::boxes::*;
use std::collections::HashMap;

pub struct BuiltinStdlib {
    pub namespaces: HashMap<String, BuiltinNamespace>,
}

pub struct BuiltinNamespace {
    pub name: String,
    pub static_boxes: HashMap<String, BuiltinStaticBox>,
}

pub struct BuiltinStaticBox {
    pub name: String,
    pub methods: HashMap<String, BuiltinMethod>,
}

pub type BuiltinMethod = fn(&[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError>;

impl BuiltinStdlib {
    pub fn new() -> Self {
        let mut stdlib = BuiltinStdlib {
            namespaces: HashMap::new(),
        };
        
        // nyashstd名前空間登録
        stdlib.register_nyashstd();
        
        stdlib
    }
    
    fn register_nyashstd(&mut self) {
        let mut nyashstd = BuiltinNamespace {
            name: "nyashstd".to_string(),
            static_boxes: HashMap::new(),
        };
        
        // string static box
        nyashstd.static_boxes.insert("string".to_string(), self.create_string_box());
        // math static box  
        nyashstd.static_boxes.insert("math".to_string(), self.create_math_box());
        // array static box
        nyashstd.static_boxes.insert("array".to_string(), self.create_array_box());
        // io static box
        nyashstd.static_boxes.insert("io".to_string(), self.create_io_box());
        
        self.namespaces.insert("nyashstd".to_string(), nyashstd);
    }
}
```

#### **文字列関数実装例**
```rust
impl BuiltinStdlib {
    fn create_string_box(&self) -> BuiltinStaticBox {
        let mut string_box = BuiltinStaticBox {
            name: "string".to_string(),
            methods: HashMap::new(),
        };
        
        // string.upper(str) -> String
        string_box.methods.insert("upper".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidArguments(
                    "string.upper() takes exactly 1 argument".to_string()
                ));
            }
            
            let string_arg = args[0].to_string_box();
            let result = StringBox::new(&string_arg.value.to_uppercase());
            Ok(Box::new(result))
        });
        
        // string.lower(str) -> String
        string_box.methods.insert("lower".to_string(), |args| {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidArguments(
                    "string.lower() takes exactly 1 argument".to_string()
                ));
            }
            
            let string_arg = args[0].to_string_box();
            let result = StringBox::new(&string_arg.value.to_lowercase());
            Ok(Box::new(result))
        });
        
        // string.split(str, separator) -> Array
        string_box.methods.insert("split".to_string(), |args| {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidArguments(
                    "string.split() takes exactly 2 arguments".to_string()
                ));
            }
            
            let string_arg = args[0].to_string_box();
            let sep_arg = args[1].to_string_box();
            
            let string_box = StringBox::new(&string_arg.value);
            let result = string_box.split(&sep_arg.value)?;
            Ok(result)
        });
        
        string_box
    }
}
```

### 2. インタープリター統合

#### **インタープリター拡張: `src/interpreter/core.rs`**
```rust
use crate::stdlib::BuiltinStdlib;

pub struct NyashInterpreter {
    // 既存フィールド...
    pub builtin_stdlib: BuiltinStdlib,
    pub using_imports: HashMap<String, Vec<String>>, // ファイル別インポート
}

impl NyashInterpreter {
    pub fn new() -> Self {
        NyashInterpreter {
            // 既存初期化...
            builtin_stdlib: BuiltinStdlib::new(),
            using_imports: HashMap::new(),
        }
    }
    
    // using文実行
    pub fn execute_using(&mut self, namespace_name: &str) -> Result<(), RuntimeError> {
        // 組み込み名前空間かチェック
        if self.builtin_stdlib.namespaces.contains_key(namespace_name) {
            // 現在ファイルのインポートリストに追加
            self.using_imports
                .entry(self.current_file_id.clone())
                .or_insert_with(Vec::new)
                .push(namespace_name.to_string());
            
            Ok(())
        } else {
            Err(RuntimeError::UndefinedNamespace(namespace_name.to_string()))
        }
    }
    
    // 短縮名解決: string.upper() -> nyashstd.string.upper()
    pub fn resolve_short_call(&self, box_name: &str, method_name: &str) 
        -> Option<(&str, &str, &str)> { // (namespace, box, method)
        
        if let Some(imports) = self.using_imports.get(&self.current_file_id) {
            for namespace_name in imports {
                if let Some(namespace) = self.builtin_stdlib.namespaces.get(namespace_name) {
                    if namespace.static_boxes.contains_key(box_name) {
                        return Some((namespace_name, box_name, method_name));
                    }
                }
            }
        }
        
        None
    }
    
    // 組み込み関数呼び出し
    pub fn call_builtin_method(&self, namespace: &str, box_name: &str, method_name: &str, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        if let Some(ns) = self.builtin_stdlib.namespaces.get(namespace) {
            if let Some(static_box) = ns.static_boxes.get(box_name) {
                if let Some(method) = static_box.methods.get(method_name) {
                    return method(&args);
                }
            }
        }
        
        Err(RuntimeError::UndefinedMethod(
            format!("{}.{}.{}", namespace, box_name, method_name)
        ))
    }
}
```

### 3. パーサー最小拡張

#### **トークナイザー: `src/tokenizer.rs`**
```rust
pub enum TokenType {
    // 既存...
    USING,           // using キーワード
    // NAMESPACE は後のPhaseで追加
}
```

#### **AST最小拡張: `src/ast.rs`**
```rust
pub enum ASTNode {
    // 既存...
    UsingStatement {
        namespace_name: String,  // "nyashstd" のみ対応
        span: Span,
    },
    // QualifiedCall は後のPhaseで追加
}
```

#### **パーサー: `src/parser/statements.rs`**
```rust
impl NyashParser {
    pub fn parse_using(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'using'
        
        if let TokenType::IDENTIFIER(namespace_name) = &self.current_token().token_type {
            let name = namespace_name.clone();
            self.advance();
            
            // Phase 0では "nyashstd" のみ許可
            if name != "nyashstd" {
                return Err(ParseError::UnsupportedNamespace(name));
            }
            
            Ok(ASTNode::UsingStatement {
                namespace_name: name,
                span: self.current_span(),
            })
        } else {
            Err(ParseError::ExpectedIdentifier)
        }
    }
}
```

## 🧪 テスト戦略

### Phase 0テストケース

#### **基本using文テスト**
```nyash
# test_using_basic.nyash
using nyashstd

local result = string.upper("hello")
assert(result == "HELLO")

local lower = string.lower("WORLD")  
assert(lower == "world")
```

#### **数学関数テスト**
```nyash
# test_math_basic.nyash
using nyashstd

local sin_result = math.sin(0)
assert(sin_result == 0)

local sqrt_result = math.sqrt(16)
assert(sqrt_result == 4)
```

#### **配列操作テスト**
```nyash
# test_array_basic.nyash
using nyashstd

local arr = [1, 2, 3]
local length = array.length(arr)
assert(length == 3)

local item = array.get(arr, 1)
assert(item == 2)
```

## 📊 実装マイルストーン

### ✅ Phase 0完了条件
- [ ] USING トークン認識
- [ ] using nyashstd 構文解析
- [ ] 組み込みnyashstd.string実装
- [ ] 組み込みnyashstd.math実装  
- [ ] 組み込みnyashstd.array実装
- [ ] 組み込みnyashstd.io実装
- [ ] 基本テストケース全通過

### 🔮 将来の発展

#### **Phase 1: 完全修飾名対応**
```nyash
# using不要でも使える
nyashstd.string.upper("hello")
```

#### **Phase 2: namespace構文対応**
```nyash
# 組み込み以外の名前空間
namespace mylib {
    static box utils {
        static process(data) { ... }
    }
}
```

#### **Phase 3: nyash.link統合**
```toml
# nyash.link
[dependencies]
mylib = { path = "./mylib.nyash" }
```

## 🎯 実装優先順位

### 🚨 Critical（今すぐ）
1. **USINGトークナイザー** - Token::USING追加
2. **using文パーサー** - "using nyashstd"解析
3. **BuiltinStdlib基盤** - src/stdlib/mod.rs作成

### ⚡ High（今週中）
4. **string関数実装** - upper, lower, split, join
5. **math関数実装** - sin, cos, sqrt, floor
6. **基本テスト** - using nyashstd動作確認

### 📝 Medium（来週）
7. **array関数実装** - length, get, push, slice
8. **io関数実装** - print, println, debug
9. **エラーハンドリング** - 適切なエラーメッセージ

---

**🎉 この戦略なら複雑なファイル依存関係システムなしで、すぐに実用的なnamespace/usingが実現できるにゃ！🐱**