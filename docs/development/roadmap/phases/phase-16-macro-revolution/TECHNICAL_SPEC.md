# Phase 16 Technical Specification: Box-Based Macro System

Date: 2025-09-19  
Version: 0.1.0  
Status: **DRAFT** - 実装前仕様  

## 🏗️ **アーキテクチャ概要**

### **システム構成**
```
Nyash Source Code
        ↓
   Lexer/Parser  
        ↓
    Raw AST
        ↓
┌─────────────────────┐
│  Macro Expansion    │ ← 新規実装部分
│  ┌───────────────┐  │
│  │ AST Pattern   │  │ ← Phase 1
│  │ Matching      │  │  
│  └───────────────┘  │
│  ┌───────────────┐  │  
│  │ Quote/Unquote │  │ ← Phase 2
│  │ System        │  │
│  └───────────────┘  │
│  ┌───────────────┐  │
│  │ HIR Patch     │  │ ← Phase 3  
│  │ Engine        │  │
│  └───────────────┘  │
└─────────────────────┘
        ↓
   Expanded AST
        ↓
   MIR Lowering  ← 既存システム（無変更）
        ↓
   MIR14 Instructions
        ↓
   VM/JIT/AOT Execution
```

## 🎯 **Phase 1: AST Pattern Matching**

### **新規AST構造体**

#### **PatternAst**
```rust
#[derive(Debug, Clone)]
pub enum PatternAst {
    // 基本パターン
    Wildcard { span: Span },                                    // _
    Identifier { name: String, span: Span },                   // variable
    Literal { value: LiteralValue, span: Span },               // 42, "hello"
    
    // 構造パターン  
    BoxPattern {
        name: String,                                           // BoxDeclaration
        fields: Vec<FieldPattern>,                              // { field1, field2, .. }
        span: Span,
    },
    
    // 配列パターン
    ArrayPattern {
        elements: Vec<PatternAst>,                              // [first, second, ...]
        rest: Option<String>,                                   // ...rest
        span: Span,
    },
    
    // OR パターン
    OrPattern {
        patterns: Vec<PatternAst>,                              // pattern1 | pattern2
        span: Span,
    },
    
    // バインドパターン
    BindPattern {
        name: String,                                           // @variable
        pattern: Box<PatternAst>,                               // @var pattern
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: PatternAst,
    pub span: Span,
}
```

#### **MatchExpression**
```rust
#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub target: Box<ASTNode>,                                  // match対象
    pub arms: Vec<MatchArm>,                                   // マッチアーム
    pub span: Span,
}

#[derive(Debug, Clone)]  
pub struct MatchArm {
    pub pattern: PatternAst,                                   // パターン
    pub guard: Option<Box<ASTNode>>,                           // if guard
    pub body: Vec<ASTNode>,                                    // 実行文
    pub span: Span,
}
```

### **パターンマッチング構文**

#### **基本構文**
```nyash
match ast_node {
    BoxDeclaration { name, fields, .. } => {
        // Box宣言の処理
    }
    FunctionDeclaration { name: "main", .. } => {
        // main関数の特別処理
    }
    _ => {
        // その他
    }
}
```

#### **高度なパターン**
```nyash
match ast_node {
    // 束縛パターン
    BoxDeclaration { name: @box_name, fields: [first, ...rest] } => {
        // box_nameにnameをバインド
        // firstに最初のフィールド、restに残り
    }
    
    // ORパターン
    Literal { value: IntegerValue | StringValue } => {
        // 整数または文字列リテラル
    }
    
    // ガード
    BoxDeclaration { fields } if fields.length > 5 => {
        // フィールドが5個より多いBox
    }
}
```

### **実装詳細**

#### **Parser拡張**
```rust
impl NyashParser {
    /// match式のパース
    pub fn parse_match_expression(&mut self) -> Result<MatchExpression, ParseError> {
        self.consume(TokenType::MATCH)?;
        let target = self.parse_expression()?;
        self.consume(TokenType::LBRACE)?;
        
        let mut arms = Vec::new();
        while !self.match_token(&TokenType::RBRACE) {
            arms.push(self.parse_match_arm()?);
        }
        
        self.consume(TokenType::RBRACE)?;
        Ok(MatchExpression { target: Box::new(target), arms, span: Span::unknown() })
    }
    
    /// パターンのパース
    pub fn parse_pattern(&mut self) -> Result<PatternAst, ParseError> {
        // パターン実装...
    }
}
```

#### **PatternMatcher**
```rust
pub struct PatternMatcher {
    bindings: HashMap<String, ASTNode>,
}

impl PatternMatcher {
    /// パターンマッチング実行
    pub fn match_pattern(&mut self, pattern: &PatternAst, value: &ASTNode) -> bool {
        match (pattern, value) {
            (PatternAst::Wildcard { .. }, _) => true,
            
            (PatternAst::Identifier { name, .. }, value) => {
                self.bindings.insert(name.clone(), value.clone());
                true
            }
            
            (PatternAst::BoxPattern { name, fields, .. }, 
             ASTNode::BoxDeclaration { name: box_name, fields: box_fields, .. }) => {
                if name == box_name {
                    self.match_fields(fields, box_fields)
                } else {
                    false
                }
            }
            
            // その他のパターン...
            _ => false,
        }
    }
}
```

## 🎯 **Phase 2: Quote/Unquote System**

### **新規AST構造体**

#### **QuoteExpression**
```rust
#[derive(Debug, Clone)]
pub struct QuoteExpression {
    pub template: Vec<ASTNode>,                                // テンプレート
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UnquoteExpression {
    pub template: Box<QuoteExpression>,                        // 展開するテンプレート
    pub substitutions: HashMap<String, ASTNode>,               // 置換マップ
    pub span: Span,
}

// テンプレート内の変数展開
#[derive(Debug, Clone)]
pub struct TemplateVariable {
    pub name: String,                                          // $(variable_name)
    pub span: Span,
}
```

### **Quote/Unquote構文**

#### **基本構文**
```nyash
// コードテンプレート作成
let template = quote! {
    $(method_name)(other) {
        return me.$(field_name).equals(other.$(field_name))
    }
}

// テンプレート展開
let generated = unquote! {
    template with {
        method_name: "equals",
        field_name: "name"
    }
}
```

#### **高度な展開**
```nyash
// リスト展開
let field_comparisons = quote! {
    $(for field in fields) {
        me.$(field.name).equals(other.$(field.name))
    }
}

// 条件展開
let method_body = quote! {
    $(if has_fields) {
        return $(field_comparisons)
    } else {
        return true
    }
}
```

### **実装詳細**

#### **TemplateEngine**
```rust
pub struct TemplateEngine {
    substitutions: HashMap<String, ASTNode>,
}

impl TemplateEngine {
    /// テンプレート展開
    pub fn expand_template(&self, template: &QuoteExpression) -> Result<Vec<ASTNode>, MacroError> {
        let mut result = Vec::new();
        
        for node in &template.template {
            match self.expand_node(node)? {
                ExpandResult::Single(n) => result.push(n),
                ExpandResult::Multiple(nodes) => result.extend(nodes),
            }
        }
        
        Ok(result)
    }
    
    /// 単一ノード展開
    fn expand_node(&self, node: &ASTNode) -> Result<ExpandResult, MacroError> {
        match node {
            ASTNode::TemplateVariable { name, .. } => {
                if let Some(substitution) = self.substitutions.get(name) {
                    Ok(ExpandResult::Single(substitution.clone()))
                } else {
                    Err(MacroError::UnboundVariable(name.clone()))
                }
            }
            
            // 再帰的展開
            _ => self.expand_node_recursive(node),
        }
    }
}
```

## 🎯 **Phase 3: HIRパッチ式マクロエンジン**

### **MacroEngine Core**

#### **MacroRegistry**
```rust
pub struct MacroRegistry {
    derive_macros: HashMap<String, Box<dyn DeriveMacro>>,
    attribute_macros: HashMap<String, Box<dyn AttributeMacro>>,
    function_macros: HashMap<String, Box<dyn FunctionMacro>>,
}

impl MacroRegistry {
    pub fn register_derive<T: DeriveMacro + 'static>(&mut self, name: &str, macro_impl: T) {
        self.derive_macros.insert(name.to_string(), Box::new(macro_impl));
    }
    
    pub fn expand_derive(&self, name: &str, input: &BoxDeclaration) -> Result<Vec<ASTNode>, MacroError> {
        if let Some(macro_impl) = self.derive_macros.get(name) {
            macro_impl.expand(input)
        } else {
            Err(MacroError::UnknownDeriveMacro(name.to_string()))
        }
    }
}
```

#### **DeriveMacro Trait**
```rust
pub trait DeriveMacro {
    /// derive マクロ展開
    fn expand(&self, input: &BoxDeclaration) -> Result<Vec<ASTNode>, MacroError>;
    
    /// サポートする型チェック
    fn supports_box(&self, box_decl: &BoxDeclaration) -> bool {
        true  // デフォルト：すべてのBoxをサポート
    }
}
```

### **@derive実装例**

#### **EqualsDeriveMacro**
```rust
pub struct EqualsDeriveMacro;

impl DeriveMacro for EqualsDeriveMacro {
    fn expand(&self, input: &BoxDeclaration) -> Result<Vec<ASTNode>, MacroError> {
        let method_name = "equals";
        let param_name = "other";
        
        // フィールド比較の生成
        let field_comparisons = self.generate_field_comparisons(&input.fields)?;
        
        // equals メソッドの生成
        let equals_method = ASTNode::FunctionDeclaration {
            name: method_name.to_string(),
            params: vec![param_name.to_string()],
            body: vec![
                ASTNode::Return {
                    value: Some(Box::new(field_comparisons)),
                    span: Span::unknown(),
                }
            ],
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        };
        
        Ok(vec![equals_method])
    }
    
    fn generate_field_comparisons(&self, fields: &[String]) -> Result<ASTNode, MacroError> {
        if fields.is_empty() {
            // フィールドなし：常にtrue
            return Ok(ASTNode::Literal {
                value: LiteralValue::Bool(true),
                span: Span::unknown(),
            });
        }
        
        // フィールド比較の連鎖
        let mut comparison = self.generate_single_field_comparison(&fields[0])?;
        
        for field in &fields[1..] {
            let field_comp = self.generate_single_field_comparison(field)?;
            comparison = ASTNode::BinaryOp {
                operator: BinaryOperator::And,
                left: Box::new(comparison),
                right: Box::new(field_comp),
                span: Span::unknown(),
            };
        }
        
        Ok(comparison)
    }
    
    fn generate_single_field_comparison(&self, field: &str) -> Result<ASTNode, MacroError> {
        // me.field.equals(other.field)
        Ok(ASTNode::MethodCall {
            object: Box::new(ASTNode::FieldAccess {
                object: Box::new(ASTNode::Me { span: Span::unknown() }),
                field: field.to_string(),
                span: Span::unknown(),
            }),
            method: "equals".to_string(),
            arguments: vec![
                ASTNode::FieldAccess {
                    object: Box::new(ASTNode::Variable {
                        name: "other".to_string(),
                        span: Span::unknown(),
                    }),
                    field: field.to_string(),
                    span: Span::unknown(),
                }
            ],
            span: Span::unknown(),
        })
    }
}
```

#### **TestMacro**
```rust
pub struct TestMacro;

impl AttributeMacro for TestMacro {
    fn expand(&self, input: &FunctionDeclaration) -> Result<Vec<ASTNode>, MacroError> {
        // テスト関数をテストレジストリに登録
        let register_call = ASTNode::MethodCall {
            object: Box::new(ASTNode::Variable {
                name: "TestRegistry".to_string(),
                span: Span::unknown(),
            }),
            method: "register".to_string(),
            arguments: vec![
                ASTNode::Literal {
                    value: LiteralValue::String(input.name.clone()),
                    span: Span::unknown(),
                },
                ASTNode::Variable {
                    name: input.name.clone(),
                    span: Span::unknown(),
                }
            ],
            span: Span::unknown(),
        };
        
        Ok(vec![
            register_call,
            ASTNode::FunctionDeclaration {
                name: input.name.clone(),
                params: input.params.clone(),
                body: input.body.clone(),
                is_static: input.is_static,
                is_override: input.is_override,
                span: input.span.clone(),
            }
        ])
    }
}
```

## 🛡️ **エラーハンドリング**

### **MacroError定義**
```rust
#[derive(Debug, Clone)]
pub enum MacroError {
    // パターンマッチングエラー
    PatternMismatch { expected: String, found: String },
    UnboundVariable(String),
    
    // Quote/Unquoteエラー  
    TemplateExpansionFailed(String),
    InvalidSubstitution { variable: String, reason: String },
    
    // Deriveマクロエラー
    UnknownDeriveMacro(String),
    UnsupportedBoxType { derive_name: String, box_name: String },
    
    // 一般エラー
    RecursionLimitExceeded,
    CircularDependency(Vec<String>),
}
```

### **エラーメッセージ**
```rust
impl MacroError {
    pub fn user_message(&self) -> String {
        match self {
            MacroError::UnknownDeriveMacro(name) => {
                format!("Unknown derive trait '{}'
Available traits: Equals, ToString, Clone, Debug
Did you mean 'ToString'?", name)
            }
            
            MacroError::PatternMismatch { expected, found } => {
                format!("Pattern mismatch: expected {}, found {}", expected, found)
            }
            
            _ => format!("Macro error: {:?}", self),
        }
    }
}
```

## 🎨 **CLI統合**

### **新規コマンドオプション**

既に実装済み：
```rust
// src/cli.rs で確認済み
.arg(
    Arg::new("expand")
        .long("expand")  
        .help("Macro: enable macro engine and dump expansion traces")
        .action(clap::ArgAction::SetTrue)
)
.arg(
    Arg::new("run-tests")
        .long("run-tests")
        .help("Run tests: enable macro engine and inject test harness")  
        .action(clap::ArgAction::SetTrue)
)
.arg(
    Arg::new("test-filter")
        .long("test-filter")
        .value_name("SUBSTR")
        .help("Only run tests whose name contains SUBSTR")
)
```

### **環境変数**
```rust
// マクロエンジン有効化
NYASH_MACRO_ENABLE=1

// マクロ展開トレース
NYASH_MACRO_TRACE=1  

// テスト実行
NYASH_TEST_RUN=1

// テストフィルタ
NYASH_TEST_FILTER="substring"
```

## 📊 **パフォーマンス要件**

### **目標指標**
- **展開時間**: < 100ms（中規模プロジェクト）
- **メモリ使用量**: < 20% 増加（ベースコンパイラ比）
- **型チェック**: 100% コンパイル時
- **エラー検出**: 100% コンパイル時

### **最適化戦略**
1. **遅延展開**: 必要な時のみマクロ展開
2. **キャッシュ**: 同一パターンの結果をキャッシュ
3. **並列処理**: 独立なマクロの並列展開
4. **メモリプール**: AST ノードの効率的割り当て

## 🚀 **実装順序**

### **Week 1-2: AST Pattern Matching**
1. PatternAst構造体定義
2. Parser拡張（match式）
3. PatternMatcher実装
4. 基本テストケース

### **Week 3-4: Quote/Unquote** 
1. QuoteExpression定義
2. TemplateEngine実装
3. 変数置換システム
4. エラーハンドリング

### **Week 5-6: HIRパッチエンジン**
1. MacroRegistry実装
2. DeriveMacro trait定義
3. EqualsDeriveMacro実装
4. TestMacro実装

### **Week 7-8: 統合・最適化**
1. CLI統合
2. エラーメッセージ改善
3. パフォーマンス最適化
4. ドキュメント作成

---

**この仕様に基づいて、世界最強のBox-Basedマクロシステムを実装する！** 🌟