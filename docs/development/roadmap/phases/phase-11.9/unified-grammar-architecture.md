# Phase 11.9: 統一文法アーキテクチャ設計書

## 🔴 現在の根本的問題

現在のNyashは、各層で予約語・文法解釈がバラバラに実装されている：

### 1. Tokenizer層での分散
```rust
// src/tokenizer.rs
match word.as_str() {
    "me" => TokenType::ME,       // ハードコード
    "from" => TokenType::FROM,    // ハードコード
    "loop" => TokenType::LOOP,    // ハードコード
    // ... 各予約語が個別定義
}
```

### 2. Parser層での独自解釈
```rust
// src/parser/mod.rs
fn parse_loop(&mut self) {
    // loop構文の独自解釈
    // while/forとの混同を個別チェック
}
```

### 3. Interpreter層での独自実行
```rust
// src/interpreter/expressions.rs
fn execute_from_call(&mut self) {
    // fromの独自解釈
    // デリゲーションロジックが埋め込み
}
```

### 4. MIR Builder層での独自変換
```rust
// src/mir/builder.rs
fn build_loop(&mut self) {
    // loop → MIRへの独自変換
    // セーフポイント挿入も個別判断
}
```

### 5. VM/JIT層での独自実行
- VM: 独自のセマンティクス実装
- JIT: 独自のコード生成戦略

## 🎯 統一文法アーキテクチャの設計

### 核心コンセプト: "Grammar as THE Source of Truth"

```
┌─────────────────────────────────────────────┐
│         Unified Grammar Engine              │
│  (単一の文法定義・解釈・実行エンジン)          │
├─────────────────────────────────────────────┤
│  - Keyword Registry (予約語レジストリ)        │
│  - Syntax Rules (構文規則)                   │
│  - Semantic Rules (意味規則)                 │
│  - Execution Semantics (実行セマンティクス)   │
└─────────────────────────────────────────────┘
                    ↓
    ┌──────────┬──────────┬──────────┬──────────┐
    │Tokenizer │ Parser   │Interpreter│MIR/VM/JIT│
    │ (利用)   │ (利用)   │  (利用)   │  (利用)  │
    └──────────┴──────────┴──────────┴──────────┘
```

## 📐 統一文法エンジンの実装設計

### 1. コア構造体
```rust
// src/grammar/engine.rs
pub struct UnifiedGrammarEngine {
    // 予約語の統一定義
    keywords: KeywordRegistry,
    
    // 構文規則の統一定義
    syntax: SyntaxRuleEngine,
    
    // 意味規則の統一定義
    semantics: SemanticRuleEngine,
    
    // 実行セマンティクスの統一定義
    execution: ExecutionSemantics,
}

impl UnifiedGrammarEngine {
    /// 単一のソースから全情報を読み込み
    pub fn load() -> Self {
        // YAML/TOML/Rustコードから読み込み
        let config = include_str!("../../../grammar/unified-grammar.toml");
        Self::from_config(config)
    }
    
    /// Tokenizerが使う統一API
    pub fn is_keyword(&self, word: &str) -> Option<TokenType> {
        self.keywords.lookup(word)
    }
    
    /// Parserが使う統一API
    pub fn parse_rule(&self, rule_name: &str) -> &SyntaxRule {
        self.syntax.get_rule(rule_name)
    }
    
    /// Interpreter/VM/JITが使う統一API
    pub fn execute_semantic(&self, op: &str, args: &[Value]) -> Value {
        self.execution.apply(op, args)
    }
}
```

### 2. 予約語レジストリ
```rust
pub struct KeywordRegistry {
    // 正規形マップ
    canonical: HashMap<String, KeywordDef>,
    
    // エイリアスマップ（非推奨含む）
    aliases: HashMap<String, String>,
    
    // コンテキスト別解釈
    contextual: HashMap<(String, Context), KeywordDef>,
}

pub struct KeywordDef {
    pub token: TokenType,
    pub category: KeywordCategory,
    pub semantic_action: SemanticAction,
    pub mir_opcode: Option<MirOpcode>,
    pub vm_handler: Option<VmHandler>,
    pub jit_lowering: Option<JitLowering>,
}
```

### 3. 構文規則エンジン
```rust
pub struct SyntaxRuleEngine {
    rules: HashMap<String, SyntaxRule>,
}

pub struct SyntaxRule {
    pub pattern: Pattern,
    pub precedence: i32,
    pub associativity: Associativity,
    pub semantic_transform: Box<dyn Fn(&AST) -> MIR>,
}
```

### 4. 実行セマンティクス統一
```rust
pub struct ExecutionSemantics {
    // 演算子の統一実装
    operators: HashMap<String, OperatorSemantics>,
    
    // 型変換の統一ルール
    coercions: CoercionRules,
    
    // エラー処理の統一
    error_handling: ErrorSemantics,
}

impl ExecutionSemantics {
    /// すべてのバックエンドが使う統一演算
    pub fn add(&self, left: Value, right: Value) -> Value {
        // Interpreter/VM/JIT すべて同じロジック
        match (&left, &right) {
            (Value::String(s1), Value::String(s2)) => {
                Value::String(s1.clone() + s2)
            }
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 + i2)
            }
            _ => self.coerce_and_add(left, right)
        }
    }
}
```

## 🔄 統一化の実装手順

### Phase 1: 基盤構築
1. `src/grammar/engine.rs` - 統一エンジン本体
2. `grammar/unified-grammar.toml` - 統一定義ファイル
3. 既存コードとの並行実行（デバッグ用）

### Phase 2: Tokenizer統合
```rust
impl NyashTokenizer {
    fn new() -> Self {
        Self {
            engine: UnifiedGrammarEngine::load(),
            // ...
        }
    }
    
    fn read_keyword_or_identifier(&mut self) -> TokenType {
        let word = self.read_word();
        // 統一エンジンに委譲
        self.engine.is_keyword(&word)
            .unwrap_or(TokenType::IDENTIFIER(word))
    }
}
```

### Phase 3: Parser統合
```rust
impl Parser {
    fn parse_statement(&mut self) -> Result<ASTNode> {
        // 統一エンジンから構文規則を取得
        let rule = self.engine.get_syntax_rule("statement");
        rule.parse(self)
    }
}
```

### Phase 4: セマンティクス統合
```rust
// Interpreter
impl Interpreter {
    fn execute_binop(&mut self, op: &str, left: Value, right: Value) -> Value {
        // 統一エンジンに委譲
        self.engine.execute_semantic(op, &[left, right])
    }
}

// VM
impl VM {
    fn execute_add(&mut self) -> Result<()> {
        let right = self.pop()?;
        let left = self.pop()?;
        // 統一エンジンに委譲
        let result = self.engine.execute_semantic("add", &[left, right]);
        self.push(result)
    }
}

// JIT
impl JitBuilder {
    fn lower_add(&mut self, left: Value, right: Value) {
        // 統一エンジンから最適化ヒントを取得
        let strategy = self.engine.get_jit_strategy("add", &left, &right);
        strategy.emit(self, left, right)
    }
}
```

## 🎯 統一定義ファイルの例

```toml
# grammar/unified-grammar.toml

[keywords.me]
token = "ME"
category = "self_reference"
deprecated_aliases = ["this", "self", "@"]
semantic_action = "load_self"
mir_opcode = "LoadSelf"
vm_handler = "OP_LOAD_ME"
jit_lowering = "emit_load_local(0)"

[keywords.from]
token = "FROM"
category = "delegation"
contexts = [
    { context = "class_decl", meaning = "inheritance" },
    { context = "method_call", meaning = "delegation_call" }
]
semantic_action = "delegate"
mir_opcode = "DelegateCall"

[keywords.loop]
token = "LOOP"
category = "control_flow"
deprecated_aliases = ["while", "for"]
semantic_action = "loop_construct"
mir_opcode = "Loop"
safepoint_insertion = true

[operators.add]
symbol = "+"
precedence = 10
associativity = "left"
type_rules = [
    { left = "String", right = "String", result = "String", action = "concat" },
    { left = "Integer", right = "Integer", result = "Integer", action = "add_i64" },
    { left = "Float", right = "Float", result = "Float", action = "add_f64" },
]
coercion_strategy = "string_priority"  # String + anything = String

[semantics.string_concat]
interpreter = "rust:concatenate_strings"
vm = "CONCAT_STRINGS"
jit = "call @nyash.string.concat_hh"
```

## 🚀 期待される効果

1. **完全な一貫性**
   - すべての層が同じ予約語定義を使用
   - すべての層が同じ文法解釈を実行
   - すべての層が同じセマンティクスを適用

2. **保守性の劇的向上**
   - 新しい予約語/演算子の追加が1箇所
   - 文法変更が全層に自動反映
   - バグの削減（解釈の不一致がなくなる）

3. **AI開発の簡素化**
   - 単一の文法定義をAIに学習させる
   - 正しいNyashコードの生成率向上
   - エラーメッセージの一貫性

4. **性能最適化の余地**
   - 統一エンジンでの最適化が全層に効果
   - JITヒントの統一管理
   - キャッシュ戦略の一元化

## 📊 実装優先度

1. **最優先**: 予約語レジストリ（すぐ効果が出る）
2. **高優先**: セマンティクス統一（バグ削減効果大）
3. **中優先**: 構文規則エンジン（段階的移行可能）
4. **低優先**: JIT最適化ヒント（性能向上は後回し）

## 🔧 移行戦略

1. **並行実行期間**
   - 新旧両方の実装を維持
   - デバッグモードで差分検出
   - 段階的に新実装に切り替え

2. **テスト駆動**
   - 各層の動作一致を自動テスト
   - スナップショットテストで回帰防止
   - ファズテストで edge case 発見

3. **ドキュメント駆動**
   - 統一文法仕様書を先に作成
   - 実装は仕様書に従う
   - AIトレーニングデータも同時生成

これにより、Nyashの全層で完全に統一された文法解釈と実行が実現される。