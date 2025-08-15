# 最終実装戦略：標準関数優先namespace/usingシステム

## 🎯 実装戦略まとめ

### 📋 設計完了項目
- ✅ **基本戦略**: nyash.link前の段階的実装
- ✅ **アーキテクチャ**: SharedState統合による高性能設計
- ✅ **標準関数**: 組み込みnyashstd名前空間
- ✅ **実装順序**: Critical → High → Medium

### 🚀 最終実装ロードマップ

## Phase 0: 組み込みnyashstd基盤（1-2週間）

### 🚨 Critical実装（即時）

#### **1. トークナイザー拡張**
```rust
// src/tokenizer.rs
pub enum TokenType {
    // 既存...
    USING,           // using キーワード追加
}

// キーワード認識
fn tokenize_keyword(word: &str) -> TokenType {
    match word {
        // 既存...
        "using" => TokenType::USING,
        _ => TokenType::IDENTIFIER(word.to_string()),
    }
}
```

#### **2. AST最小拡張**
```rust
// src/ast.rs
pub enum ASTNode {
    // 既存...
    UsingStatement {
        namespace_name: String,  // Phase 0: "nyashstd"のみ
        span: Span,
    },
}
```

#### **3. BuiltinStdlib基盤**
```rust
// 新ファイル: src/stdlib/mod.rs
pub mod builtin;
pub use builtin::*;

// 新ファイル: src/stdlib/builtin.rs
// （前回設計したBuiltinStdlib実装）
```

#### **4. SharedState統合**
```rust
// src/interpreter/core.rs
#[derive(Clone)]
pub struct SharedState {
    // 既存フィールド...
    pub builtin_stdlib: Arc<BuiltinStdlib>,
    pub using_imports: Arc<RwLock<HashMap<String, UsingContext>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            // 既存初期化...
            builtin_stdlib: Arc::new(BuiltinStdlib::new()),
            using_imports: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
```

### ⚡ High実装（今週中）

#### **5. using文パーサー**
```rust
// src/parser/statements.rs
impl NyashParser {
    pub fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            // 既存ケース...
            TokenType::USING => self.parse_using(),
            // 他の既存ケース...
        }
    }
    
    fn parse_using(&mut self) -> Result<ASTNode, ParseError> {
        let start_span = self.current_token().span.clone();
        self.advance(); // consume 'using'
        
        if let TokenType::IDENTIFIER(namespace_name) = &self.current_token().token_type {
            let name = namespace_name.clone();
            self.advance();
            
            // Phase 0制限：nyashstdのみ許可
            if name != "nyashstd" {
                return Err(ParseError::UnsupportedFeature(
                    format!("Only 'nyashstd' namespace is supported in Phase 0, got '{}'", name)
                ));
            }
            
            Ok(ASTNode::UsingStatement {
                namespace_name: name,
                span: start_span,
            })
        } else {
            Err(ParseError::ExpectedIdentifier(
                "Expected namespace name after 'using'".to_string()
            ))
        }
    }
}
```

#### **6. 基本string関数実装**
```rust
// src/stdlib/builtin.rs拡張
impl BuiltinStdlib {
    fn register_string_functions(&mut self) {
        // string.upper
        self.register_function("string.upper", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string",
            method_name: "upper",
            implementation: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.upper() takes exactly 1 argument".to_string()
                    ));
                }
                
                let input = &args[0].to_string_box().value;
                let result = StringBox::new(&input.to_uppercase());
                Ok(Box::new(result))
            },
            arg_count: Some(1),
            description: "Convert string to uppercase",
        });
        
        // string.lower
        self.register_function("string.lower", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string", 
            method_name: "lower",
            implementation: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.lower() takes exactly 1 argument".to_string()
                    ));
                }
                
                let input = &args[0].to_string_box().value;
                let result = StringBox::new(&input.to_lowercase());
                Ok(Box::new(result))
            },
            arg_count: Some(1),
            description: "Convert string to lowercase",
        });
        
        // string.split
        self.register_function("string.split", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string",
            method_name: "split", 
            implementation: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.split() takes exactly 2 arguments".to_string()
                    ));
                }
                
                let string_box = StringBox::new(&args[0].to_string_box().value);
                let separator = &args[1].to_string_box().value;
                string_box.split(separator)
            },
            arg_count: Some(2),
            description: "Split string by separator",
        });
        
        // string.join
        self.register_function("string.join", BuiltinFunction {
            namespace: "nyashstd",
            box_name: "string",
            method_name: "join",
            implementation: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments(
                        "string.join() takes exactly 2 arguments".to_string()
                    ));
                }
                
                let array_arg = &args[0];
                let separator = &args[1].to_string_box().value;
                let separator_box = StringBox::new(separator);
                separator_box.join(array_arg.clone())
            },
            arg_count: Some(2), 
            description: "Join array elements with separator",
        });
    }
}
```

#### **7. インタープリター統合**
```rust
// src/interpreter/expressions.rs
impl NyashInterpreter {
    pub fn execute_expression(&mut self, node: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match node {
            // using文処理
            ASTNode::UsingStatement { namespace_name, .. } => {
                self.execute_using(namespace_name)?;
                Ok(Box::new(VoidBox::new()))
            }
            
            // メソッド呼び出し処理拡張
            ASTNode::MethodCall { object, method, args, .. } => {
                // 組み込み関数チェック
                if let ASTNode::Variable { name: box_name, .. } = object.as_ref() {
                    let path = vec![box_name.clone(), method.clone()];
                    if let Some(qualified_name) = self.resolve_qualified_call(&path) {
                        let evaluated_args = self.evaluate_arguments(args)?;
                        return self.call_builtin_function(&qualified_name, evaluated_args);
                    }
                }
                
                // 既存のメソッド呼び出し処理
                // ...
            }
            
            // 既存の他のケース...
        }
    }
}
```

### 📝 Medium実装（来週）

#### **8. math関数実装**
```rust
// math.sin, cos, sqrt, floor, random
```

#### **9. array関数実装**
```rust
// array.length, get, push, slice
```

#### **10. io関数実装**
```rust
// io.print, println, debug
```

## Phase 1: 拡張機能（2-3週間後）

### 🌟 完全修飾名対応
```nyash
# using不要でも使える
nyashstd.string.upper("hello")
nyashstd.math.sin(3.14)
```

#### **実装**
```rust
// ASTNode::QualifiedCall追加
ASTNode::QualifiedCall {
    path: Vec<String>,  // ["nyashstd", "string", "upper"]
    args: Vec<ASTNode>,
    span: Span,
}

// パーサーで "identifier.identifier.identifier()" 構文解析
```

### 🔧 エラーハンドリング強化
```rust
// より詳細なエラーメッセージ
RuntimeError::UndefinedBuiltinMethod {
    namespace: String,
    box_name: String, 
    method_name: String,
    available_methods: Vec<String>,  // "Did you mean: ..."
    span: Span,
}
```

### 📊 IDE補完サポート
```rust
// Language Server連携用API
impl BuiltinStdlib {
    pub fn get_completion_candidates(&self, prefix: &str) -> Vec<CompletionItem> {
        // "ny" -> ["nyashstd"]
        // "nyashstd." -> ["string", "math", "array", "io"]  
        // "nyashstd.string." -> ["upper", "lower", "split", "join"]
    }
}
```

## Phase 2: nyash.link準備（1ヶ月後）

### 🔗 外部モジュール対応基盤
```rust
// ModuleResolver拡張
pub enum NamespaceSource {
    Builtin(Arc<BuiltinStdlib>),     // 組み込み
    External(PathBuf),               // nyash.linkで管理
}

// NamespaceRegistry統合
pub struct NamespaceRegistry {
    builtin: Arc<BuiltinStdlib>,
    external: HashMap<String, ExternalModule>,
}
```

### 📁 nyash.link対応
```toml
[dependencies]
mylib = { path = "./mylib.nyash" }

# using mylib  # Phase 2で対応
```

## 🧪 段階的テスト戦略

### Phase 0テスト
```nyash
# test_phase0_basic.nyash
using nyashstd

# 基本動作
assert(string.upper("hello") == "HELLO")
assert(string.lower("WORLD") == "world")

# エラー処理
try {
    using unknown_namespace
} catch e {
    assert(e.contains("nyashstd"))
}
```

### Phase 1テスト
```nyash
# test_phase1_qualified.nyash
# using不要のテスト
assert(nyashstd.string.upper("hello") == "HELLO")
assert(nyashstd.math.sin(0) == 0)
```

### Phase 2テスト
```nyash
# test_phase2_external.nyash
using mylib

assert(mylib.custom.process("data") == "processed: data")
```

## 📊 実装マイルストーン

### ✅ Phase 0完了条件
- [ ] USINGトークン認識
- [ ] using nyashstd構文解析
- [ ] 組み込みstring関数4種動作
- [ ] 基本テスト全通過
- [ ] エラーハンドリング適切

### ✅ Phase 1完了条件  
- [ ] 完全修飾名 nyashstd.string.upper() 動作
- [ ] math/array/io関数実装
- [ ] IDE補完候補API実装
- [ ] 詳細エラーメッセージ

### ✅ Phase 2完了条件
- [ ] 外部モジュール基盤実装
- [ ] nyash.link基本対応
- [ ] 依存関係解決機能
- [ ] 全機能統合テスト

## 🔥 即座に開始すべき実装

### 今日やること
1. **src/stdlib/mod.rs作成** - モジュール基盤
2. **TokenType::USING追加** - トークナイザー拡張  
3. **BuiltinStdlib::new()実装** - 空の基盤作成

### 今週やること
4. **using文パーサー実装** - 基本構文解析
5. **string.upper()実装** - 最初の関数
6. **基本テスト作成** - 動作確認

### 来週やること
7. **string関数完成** - lower, split, join
8. **math関数開始** - sin, cos, sqrt
9. **IDE補完設計** - Language Server準備

---

**🎯 この段階的戦略で、複雑なnyash.linkなしに即座に実用的なnamespace/usingシステムが実現できるにゃ！**

**🚀 Phase 0実装を今すぐ開始して、Nyashをモダンなプログラミング言語に進化させよう！🐱✨**