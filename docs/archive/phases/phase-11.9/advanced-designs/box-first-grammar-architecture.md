# Box-First統一文法アーキテクチャ再設計

## 🚨 現在の設計の問題点

### 1. 密結合の罠
```rust
// ❌ 現在の設計: 各層がUnifiedGrammarEngineに直接依存
impl Tokenizer {
    fn tokenize(&mut self) {
        self.engine.is_keyword() // 直接参照！
    }
}
```

### 2. 根が這う実装
```rust
// ❌ UnifiedKeyword構造体が全層の情報を持つ
struct UnifiedKeyword {
    token_type: TokenType,      // Tokenizer層
    semantic_action: Action,    // Parser層
    mir_instruction: MirOp,     // MIR層
    vm_opcode: VmOp,           // VM層
    jit_pattern: JitPattern,    // JIT層
    // すべてが絡み合っている！
}
```

### 3. 巨大な神オブジェクト
```rust
// ❌ UnifiedGrammarEngineが全てを知っている
struct UnifiedGrammarEngine {
    keywords: KeywordRegistry,
    syntax: SyntaxRules,
    semantics: SemanticRules,
    execution: ExecutionSemantics,
    // 責任が多すぎる！
}
```

## 🎯 Box-First再設計

### 核心思想: 「箱に入れて、箱同士をつなぐ」

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ GrammarBox  │     │ TokenBox    │     │ ParserBox   │
│ (定義のみ)  │ --> │ (Token化)   │ --> │ (構文解析)  │
└─────────────┘     └─────────────┘     └─────────────┘
       |                                         |
       v                                         v
┌─────────────┐                         ┌─────────────┐
│ SemanticBox │ <---------------------- │   ASTBox    │
│ (意味解釈)  │                         │ (構文木)    │
└─────────────┘                         └─────────────┘
       |
       v
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   MIRBox    │ --> │    VMBox    │     │   JITBox    │
│ (中間表現)  │     │ (実行)      │     │ (コンパイル) │
└─────────────┘     └─────────────┘     └─────────────┘
```

## 📦 各箱の責任と境界

### 1. GrammarBox - 純粋な定義の箱
```rust
// 定義だけを持つ、実装を持たない
box GrammarBox {
    init { definitions }
    
    // キーワード定義を返すだけ
    getKeywordDef(word) {
        return me.definitions.keywords.get(word)
    }
    
    // 演算子定義を返すだけ
    getOperatorDef(symbol) {
        return me.definitions.operators.get(symbol)
    }
}

// キーワード定義は純粋なデータ
box KeywordDef {
    init { literal, category, aliases }
    // 実装なし、データのみ
}
```

### 2. TokenBox - トークン化だけの責任
```rust
box TokenBox {
    init { grammarBox }  // 定義への参照のみ
    
    tokenize(text) {
        local tokens = []
        // GrammarBoxに聞くだけ、自分では判断しない
        loop(text.hasMore()) {
            local word = text.readWord()
            local def = me.grammarBox.getKeywordDef(word)
            if def {
                tokens.push(new Token(def.category, word))
            } else {
                tokens.push(new Token("IDENTIFIER", word))
            }
        }
        return tokens
    }
}
```

### 3. SemanticBox - 意味解釈の箱
```rust
box SemanticBox {
    init { }  // 他の箱に依存しない！
    
    // 純粋関数として実装
    add(left, right) {
        // String + String
        if left.isString() and right.isString() {
            return new StringBox(left.value + right.value)
        }
        // Number + Number
        if left.isNumber() and right.isNumber() {
            return new IntegerBox(left.value + right.value)
        }
        // エラー
        return new ErrorBox("Type mismatch")
    }
    
    coerceToString(value) {
        // 各型の変換ロジック
        if value.isString() { return value }
        if value.isNumber() { return new StringBox(value.toString()) }
        // ...
    }
}
```

### 4. MIRBuilderBox - AST→MIR変換の箱
```rust
box MIRBuilderBox {
    init { semanticBox }  // セマンティクスへの参照のみ
    
    buildFromAST(ast) {
        // ASTの種類に応じてMIRを生成
        if ast.type == "BinaryOp" {
            return me.buildBinaryOp(ast)
        }
        // ...
    }
    
    buildBinaryOp(ast) {
        local left = me.buildFromAST(ast.left)
        local right = me.buildFromAST(ast.right)
        
        // セマンティクスに聞いて、適切なMIR命令を選択
        if ast.op == "+" {
            // SemanticBoxに型情報を聞く
            local mirOp = me.selectAddInstruction(left.type, right.type)
            return new MIRNode(mirOp, left, right)
        }
    }
}
```

## 🔄 疎結合の実現方法

### 1. インターフェース（契約）による結合
```rust
// 各箱は最小限のインターフェースだけを公開
trait TokenProvider {
    fn next_token(&mut self) -> Option<Token>;
}

trait SemanticProvider {
    fn apply_operator(&self, op: &str, args: &[Value]) -> Result<Value>;
}

trait MIRProvider {
    fn get_instruction(&self, index: usize) -> &MIRInstruction;
}
```

### 2. メッセージパッシング
```rust
// 箱同士は直接呼び出さず、メッセージで通信
box ParserBox {
    parseExpression() {
        // TokenBoxにメッセージを送る
        local token = me.sendMessage(me.tokenBox, "nextToken")
        
        // 結果を処理
        if token.type == "NUMBER" {
            return new NumberNode(token.value)
        }
    }
}
```

### 3. イベント駆動
```rust
// 文法変更時の通知システム
box GrammarBox {
    updateKeyword(word, newDef) {
        me.definitions.keywords.set(word, newDef)
        // 変更を通知（購読者に伝える）
        me.notify("keyword_changed", word)
    }
}

box TokenBox {
    init { grammarBox }
    
    constructor() {
        // 文法変更を購読
        me.grammarBox.subscribe("keyword_changed", me.onKeywordChanged)
    }
    
    onKeywordChanged(word) {
        // キャッシュをクリア
        me.clearCache()
    }
}
```

## 📐 ビルド時生成の箱化

### GeneratorBox - コード生成も箱
```rust
box GeneratorBox {
    init { grammarBox, outputPath }
    
    generate() {
        local grammar = me.grammarBox.getDefinitions()
        
        // 各層向けのコードを生成
        me.generateTokens(grammar.keywords)
        me.generateParseTables(grammar.syntax)
        me.generateSemanticTables(grammar.operators)
    }
    
    generateTokens(keywords) {
        local code = "pub enum Token {\n"
        keywords.forEach((name, def) => {
            code += "    " + name + ",\n"
        })
        code += "}\n"
        
        me.writeFile("generated/tokens.rs", code)
    }
}
```

## 🎯 密結合を避ける設計原則

### 1. 単一責任の原則
- GrammarBox: 定義の管理のみ
- TokenBox: トークン化のみ
- ParserBox: 構文解析のみ
- SemanticBox: 意味解釈のみ

### 2. 依存関係の逆転
```rust
// ❌ 悪い例: 具象に依存
box VMBox {
    init { mirBuilder: MIRBuilderBox }  // 具象型に依存
}

// ✅ 良い例: 抽象に依存
box VMBox {
    init { mirProvider: MIRProvider }  // インターフェースに依存
}
```

### 3. Open/Closed原則
```rust
// 新しい演算子の追加が既存コードを変更しない
box OperatorRegistry {
    init { operators }
    
    register(symbol, handler) {
        me.operators.set(symbol, handler)
    }
    
    apply(symbol, args) {
        local handler = me.operators.get(symbol)
        if handler {
            return handler.apply(args)
        }
        return new ErrorBox("Unknown operator")
    }
}
```

## 🔧 段階的移行（箱単位）

### Phase 1: GrammarBox導入
- grammar.yamlをGrammarBoxでラップ
- 既存コードはGrammarBox経由でアクセス

### Phase 2: TokenBox分離
- Tokenizerの機能をTokenBoxに移動
- GrammarBoxへの依存を最小化

### Phase 3: SemanticBox独立
- 演算子実装をSemanticBoxに集約
- 純粋関数として実装

### Phase 4: 箱間通信の確立
- メッセージパッシング導入
- イベントシステム構築

## 📊 疎結合度の測定

### 1. 依存関係グラフ
```
GrammarBox (依存なし)
    ↓
TokenBox → GrammarBox (1依存)
ParserBox → TokenBox (1依存)
SemanticBox (依存なし)
MIRBox → SemanticBox (1依存)
VMBox → MIRBox (1依存)
JITBox → MIRBox (1依存)
```

### 2. 変更影響範囲
- 新キーワード追加: GrammarBoxのみ
- 新演算子追加: GrammarBox + SemanticBoxのみ
- 新バックエンド追加: 既存箱への変更なし

## 🚀 期待される効果

1. **真の疎結合**: 各箱が独立して開発・テスト可能
2. **容易な拡張**: 新しい箱の追加が既存を壊さない
3. **明確な境界**: 責任の所在が明確
4. **並行開発**: チームが独立して各箱を開発可能

これで「Everything is Box」哲学に忠実な、真に疎結合な統一文法アーキテクチャが実現されます。