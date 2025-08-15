# nyash.linkシステム実装計画

## 🎯 実装戦略

### 📊 現状確認
- ✅ **include**: 限定的使用（text_adventure例のみ）→廃止OK
- ✅ **using**: 未実装→完全新規作成
- ✅ **namespace**: 設計完了→実装のみ
- ✅ **Gemini推奨**: 技術的妥当性確認済み

## 📋 段階的実装ロードマップ

### 🚀 **Phase 1: 基盤構築（1-2週間）**

#### 1.1 トークナイザー拡張
```rust
// src/tokenizer.rs
pub enum TokenType {
    // 既存...
    USING,           // using キーワード
    NAMESPACE,       // namespace キーワード  
    AS,              // as キーワード（将来のエイリアス用）
}

// キーワード認識追加
fn tokenize_identifier(input: &str) -> TokenType {
    match input {
        // 既存...
        "using" => TokenType::USING,
        "namespace" => TokenType::NAMESPACE,
        "as" => TokenType::AS,
        _ => TokenType::IDENTIFIER(input.to_string()),
    }
}
```

#### 1.2 AST拡張
```rust
// src/ast.rs
pub enum ASTNode {
    // 既存...
    UsingStatement {
        module_path: Vec<String>,  // ["nyashstd"] or ["mylib"]
        alias: Option<String>,     // using mylib as lib
        span: Span,
    },
    NamespaceDeclaration {
        name: String,
        body: Vec<ASTNode>,
        span: Span,
    },
    QualifiedCall {
        path: Vec<String>,         // ["nyashstd", "string", "upper"]
        args: Vec<ASTNode>,
        span: Span,
    },
}
```

#### 1.3 パーサー基本実装
```rust
// src/parser/statements.rs
impl NyashParser {
    pub fn parse_using(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'using'
        
        let module_path = self.parse_module_path()?;
        // using mylib → ["mylib"] 
        // using nyashstd.string → ["nyashstd", "string"]
        
        Ok(ASTNode::UsingStatement {
            module_path,
            alias: None, // Phase 1では未サポート
            span: self.current_span(),
        })
    }
    
    fn parse_module_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut path = vec![];
        
        // 最初の識別子
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            path.push(name.clone());
            self.advance();
        } else {
            return Err(ParseError::ExpectedIdentifier);
        }
        
        // ドット区切りで追加パス（将来拡張）
        // using nyashstd.string のような構文
        
        Ok(path)
    }
}
```

### ⚡ **Phase 2: nyash.link基盤（2-3週間）**

#### 2.1 nyash.linkパーサー
```rust
// 新ファイル: src/link_file.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct NyashLink {
    pub project: Option<ProjectInfo>,
    pub dependencies: HashMap<String, Dependency>,
    pub search_paths: Option<HashMap<String, String>>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    Path { path: String },
    Stdlib { stdlib: bool },
    Registry { version: String, registry: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildConfig {
    pub entry_point: Option<String>,
}

impl NyashLink {
    pub fn from_file(path: &Path) -> Result<Self, LinkError> {
        let content = std::fs::read_to_string(path)?;
        let link: NyashLink = toml::from_str(&content)?;
        Ok(link)
    }
    
    pub fn resolve_dependency(&self, name: &str) -> Option<PathBuf> {
        if let Some(dep) = self.dependencies.get(name) {
            match dep {
                Dependency::Path { path } => Some(PathBuf::from(path)),
                Dependency::Stdlib { .. } => {
                    // 標準ライブラリパス解決ロジック
                    self.resolve_stdlib_path(name)
                }
                _ => None, // Phase 2では未サポート
            }
        } else {
            None
        }
    }
}
```

#### 2.2 依存関係解決エンジン
```rust
// 新ファイル: src/module_resolver.rs
pub struct ModuleResolver {
    nyash_link: NyashLink,
    loaded_modules: HashMap<String, Arc<ParsedModule>>,
    loading_stack: Vec<String>, // 循環依存検出用
}

impl ModuleResolver {
    pub fn new(link_path: &Path) -> Result<Self, ResolverError> {
        let nyash_link = NyashLink::from_file(link_path)?;
        Ok(ModuleResolver {
            nyash_link,
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
        })
    }
    
    pub fn resolve_using(&mut self, module_name: &str) -> Result<Arc<ParsedModule>, ResolverError> {
        // 既にロード済みかチェック
        if let Some(module) = self.loaded_modules.get(module_name) {
            return Ok(module.clone());
        }
        
        // 循環依存チェック
        if self.loading_stack.contains(&module_name.to_string()) {
            return Err(ResolverError::CircularDependency(
                self.loading_stack.clone()
            ));
        }
        
        // ファイルパス解決
        let file_path = self.nyash_link.resolve_dependency(module_name)
            .ok_or(ResolverError::ModuleNotFound(module_name.to_string()))?;
            
        // 再帰的読み込み防止
        self.loading_stack.push(module_name.to_string());
        
        // ファイル読み込み・パース
        let content = std::fs::read_to_string(&file_path)?;
        let ast = NyashParser::parse_from_string(&content)?;
        
        // モジュール作成
        let module = Arc::new(ParsedModule {
            name: module_name.to_string(),
            file_path,
            ast,
            exports: self.extract_exports(&ast)?,
        });
        
        // キャッシュに保存
        self.loaded_modules.insert(module_name.to_string(), module.clone());
        self.loading_stack.pop();
        
        Ok(module)
    }
}
```

### 📈 **Phase 3: 名前空間システム（3-4週間）**

#### 3.1 namespace解析
```rust
impl NyashParser {
    pub fn parse_namespace(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'namespace'
        
        let name = self.expect_identifier()?;
        self.expect_token(TokenType::LBRACE)?;
        
        let mut body = vec![];
        while !self.check_token(&TokenType::RBRACE) {
            body.push(self.parse_statement()?);
        }
        
        self.expect_token(TokenType::RBRACE)?;
        
        Ok(ASTNode::NamespaceDeclaration {
            name,
            body,
            span: self.current_span(),
        })
    }
}
```

#### 3.2 名前空間レジストリ
```rust
// 新ファイル: src/namespace_registry.rs
pub struct NamespaceRegistry {
    namespaces: HashMap<String, Namespace>,
    using_imports: HashMap<String, Vec<String>>, // ファイル別インポート
}

pub struct Namespace {
    pub name: String,
    pub static_boxes: HashMap<String, StaticBox>,
}

pub struct StaticBox {
    pub name: String,
    pub static_methods: HashMap<String, MethodSignature>,
}

impl NamespaceRegistry {
    pub fn register_namespace(&mut self, name: String, namespace: Namespace) {
        self.namespaces.insert(name, namespace);
    }
    
    pub fn add_using_import(&mut self, file_id: String, namespace_name: String) {
        self.using_imports
            .entry(file_id)
            .or_insert_with(Vec::new)
            .push(namespace_name);
    }
    
    pub fn resolve_call(&self, file_id: &str, path: &[String]) -> Option<MethodSignature> {
        // 例: string.upper() → nyashstd.string.upper()
        if path.len() == 2 {
            let box_name = &path[0];
            let method_name = &path[1];
            
            // usingでインポートされた名前空間を検索
            if let Some(imports) = self.using_imports.get(file_id) {
                for namespace_name in imports {
                    if let Some(namespace) = self.namespaces.get(namespace_name) {
                        if let Some(static_box) = namespace.static_boxes.get(box_name) {
                            if let Some(method) = static_box.static_methods.get(method_name) {
                                return Some(method.clone());
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
}
```

### 🎯 **Phase 4: インタープリター統合（4-5週間）**

#### 4.1 using文実行
```rust
// src/interpreter/core.rs
impl NyashInterpreter {
    pub fn execute_using(&mut self, module_path: &[String]) -> Result<(), RuntimeError> {
        let module_name = module_path.join(".");
        
        // モジュール解決・読み込み
        let module = self.module_resolver.resolve_using(&module_name)?;
        
        // 名前空間登録
        if let Some(namespace) = self.extract_namespace_from_module(&module) {
            self.namespace_registry.register_namespace(module_name.clone(), namespace);
            self.namespace_registry.add_using_import(
                self.current_file_id.clone(), 
                module_name
            );
        }
        
        Ok(())
    }
    
    fn extract_namespace_from_module(&self, module: &ParsedModule) -> Option<Namespace> {
        // ASTからnamespace宣言を探して解析
        for node in &module.ast {
            if let ASTNode::NamespaceDeclaration { name, body, .. } = node {
                return Some(self.build_namespace_from_body(name, body));
            }
        }
        None
    }
}
```

#### 4.2 qualified call実行
```rust
impl NyashInterpreter {
    pub fn execute_qualified_call(&mut self, path: &[String], args: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        // 名前解決
        if let Some(method_sig) = self.namespace_registry.resolve_call(
            &self.current_file_id, 
            path
        ) {
            // 引数評価
            let evaluated_args = self.evaluate_args(args)?;
            
            // メソッド実行（既存のBox呼び出しシステム活用）
            return self.call_static_method(&method_sig, evaluated_args);
        }
        
        // 完全修飾名として試行
        if path.len() >= 3 {
            // nyashstd.string.upper() の場合
            let namespace_name = &path[0];
            let box_name = &path[1]; 
            let method_name = &path[2];
            
            if let Some(namespace) = self.namespace_registry.namespaces.get(namespace_name) {
                if let Some(static_box) = namespace.static_boxes.get(box_name) {
                    if let Some(method) = static_box.static_methods.get(method_name) {
                        let evaluated_args = self.evaluate_args(args)?;
                        return self.call_static_method(method, evaluated_args);
                    }
                }
            }
        }
        
        Err(RuntimeError::UndefinedMethod(path.join(".")))
    }
}
```

## 🧪 テスト戦略

### Phase 1テスト
```nyash
# test_basic_using.nyash
# 基本using文テスト

# ファイル: mylib.nyash
static function hello() {
    return "Hello from mylib!"
}

# ファイル: main.nyash  
using mylib
local result = mylib.hello()
assert(result == "Hello from mylib!")
```

### Phase 2テスト
```nyash
# test_nyash_link.nyash
# nyash.linkファイル連携テスト

# nyash.link内容:
# [dependencies]
# mylib = { path = "./mylib.nyash" }

using mylib
local result = mylib.process("data")
assert(result == "processed: data")
```

### Phase 3テスト
```nyash
# test_namespace.nyash
# 名前空間システムテスト

# nyashstd.nyash:
# namespace nyashstd {
#     static box string {
#         static upper(str) { ... }
#     }
# }

using nyashstd
local result = string.upper("hello")
assert(result == "HELLO")

# 完全修飾名
local result2 = nyashstd.string.upper("world")
assert(result2 == "WORLD")
```

## 📊 実装マイルストーン

### ✅ 完了条件

#### Phase 1
- [ ] USING/NAMESPACE トークン認識
- [ ] using文AST構築
- [ ] 基本パーサーテスト通過

#### Phase 2  
- [ ] nyash.linkファイル読み込み
- [ ] 依存関係解決
- [ ] モジュールキャッシュ機能

#### Phase 3
- [ ] namespace宣言解析
- [ ] 名前空間レジストリ動作
- [ ] 静的メソッド解決

#### Phase 4
- [ ] インタープリター統合
- [ ] qualified call実行  
- [ ] 全テストケース通過

## 🔮 将来拡張

### Phase 5: 高度機能
- エイリアス（`using mylib as lib`）
- 選択インポート（`using nyashstd.string`）
- 動的モジュール読み込み

### Phase 6: 標準ライブラリ  
- nyashstd.nyash完全実装
- string/math/io/http モジュール
- ドキュメント生成

### Phase 7: エコシステム
- パッケージレジストリ設計
- CLI ツール（nyash init/install）
- IDE Language Server連携

---

**🎯 この実装計画でnyash.linkシステムを段階的に完成させるにゃ！**