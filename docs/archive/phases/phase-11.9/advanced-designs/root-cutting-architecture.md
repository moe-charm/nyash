# 根切り文法アーキテクチャ - 真の疎結合設計

## 🌳 「根が這う」問題の本質

### 現在の設計の根本的な問題
```rust
// 🌳 根が這っている例: 一つの変更が全体に波及
struct Keyword {
    name: String,
    token_type: TokenType,     // Tokenizer層の型
    parser_rule: ParserRule,   // Parser層の型
    mir_op: MIROpcode,         // MIR層の型
    vm_handler: VMHandler,     // VM層の型
    // → 一つのstructが全層の型を知っている！
}
```

## 🎯 根切り設計: レイヤー完全分離

### 核心思想: 「各層は自分の関心事だけを知る」

```
【Tokenizer層】          【Parser層】           【Semantic層】
    "me"        →      Token::Me      →     SelfReference
    知識:文字列のみ      知識:トークンのみ      知識:意味のみ
    
【MIR層】               【VM層】              【JIT層】
 LoadLocal(0)     →    OP_LOAD_0      →    mov rax, [rbp]
 知識:MIRのみ          知識:オペコードのみ    知識:機械語のみ
```

## 📦 真の箱化: 変換箱（TransformerBox）パターン

### 1. 各層は純粋な箱
```rust
// Tokenizer層: 文字列→トークンの変換のみ
box StringToTokenBox {
    init { }  // 依存なし！
    
    transform(text: String) -> TokenStream {
        // 純粋な文字列処理
        local tokens = []
        local chars = text.chars()
        
        loop(chars.hasNext()) {
            local ch = chars.next()
            if ch.isLetter() {
                local word = me.readWord(chars, ch)
                tokens.push(me.classifyWord(word))
            }
            // ...
        }
        return TokenStream(tokens)
    }
    
    classifyWord(word: String) -> Token {
        // ローカルな判定のみ
        match word {
            "me" => Token::Me,
            "from" => Token::From,
            "loop" => Token::Loop,
            _ => Token::Identifier(word)
        }
    }
}
```

### 2. 層間の変換も箱
```rust
// Token→AST変換箱
box TokenToASTBox {
    init { }  // 依存なし！
    
    transform(tokens: TokenStream) -> AST {
        local parser = PrattParser()
        return parser.parse(tokens)
    }
}

// AST→MIR変換箱
box ASTToMIRBox {
    init { }  // 依存なし！
    
    transform(ast: AST) -> MIR {
        match ast {
            AST::BinaryOp(op, left, right) => {
                local leftMIR = me.transform(left)
                local rightMIR = me.transform(right)
                return me.selectMIROp(op, leftMIR, rightMIR)
            }
            // ...
        }
    }
    
    selectMIROp(op: String, left: MIR, right: MIR) -> MIR {
        // ローカルな判断のみ
        if op == "+" {
            if left.type == "String" and right.type == "String" {
                return MIR::StringConcat(left, right)
            }
            if left.type == "Integer" and right.type == "Integer" {
                return MIR::AddI64(left, right)
            }
        }
        // ...
    }
}
```

## 🔄 パイプライン: 箱の連鎖

### 純粋関数的パイプライン
```rust
// 各箱は前の箱の出力を入力として受け取るだけ
box NyashPipeline {
    init { }
    
    compile(source: String) -> ExecutableCode {
        // 各変換箱を順番に適用
        local tokens = StringToTokenBox().transform(source)
        local ast = TokenToASTBox().transform(tokens)
        local mir = ASTToMIRBox().transform(ast)
        local bytecode = MIRToVMBox().transform(mir)
        return bytecode
    }
}
```

## 📐 設定の分離: ConfigBox

### 文法定義も実行時から分離
```rust
// ビルド時のみ使用される設定箱
box GrammarConfigBox {
    init { yamlPath }
    
    load() -> GrammarConfig {
        // YAMLを読み込んで設定オブジェクトを返す
        return YAML.parse(File.read(me.yamlPath))
    }
}

// ビルド時コード生成箱
box CodeGeneratorBox {
    init { config }
    
    generate() {
        // 設定から各層のコードを生成
        me.generateTokenizerTable(me.config.keywords)
        me.generateParserTable(me.config.syntax)
        me.generateMIRTable(me.config.semantics)
    }
    
    generateTokenizerTable(keywords) {
        // キーワードマッチング用の完全ハッシュ関数生成
        local code = "fn classify_keyword(s: &str) -> Token {\n"
        code += "    match s {\n"
        keywords.forEach((word, info) => {
            code += '        "' + word + '" => Token::' + info.token + ',\n'
        })
        code += "        _ => Token::Identifier(s.to_string())\n"
        code += "    }\n"
        code += "}\n"
        File.write("src/generated/keywords.rs", code)
    }
}
```

## 🎯 セマンティクスの分離

### セマンティクスも変換箱として実装
```rust
// 型強制変換箱
box TypeCoercionBox {
    init { }  // 依存なし！
    
    coerceToString(value: Value) -> StringValue {
        match value {
            Value::String(s) => StringValue(s),
            Value::Integer(i) => StringValue(i.toString()),
            Value::Float(f) => StringValue(f.toString()),
            Value::Bool(b) => StringValue(b ? "true" : "false"),
            _ => panic("Cannot coerce to string")
        }
    }
}

// 演算子実行箱
box OperatorExecutorBox {
    init { coercionBox }
    
    executeAdd(left: Value, right: Value) -> Value {
        // ローカルな判断
        match (left, right) {
            (Value::String(s1), Value::String(s2)) => {
                Value::String(s1 + s2)
            }
            (Value::String(s), other) => {
                local s2 = me.coercionBox.coerceToString(other)
                Value::String(s + s2.value)
            }
            (Value::Integer(i1), Value::Integer(i2)) => {
                Value::Integer(i1 + i2)
            }
            // ...
        }
    }
}
```

## 🔧 テスト可能性の向上

### 各箱が独立してテスト可能
```rust
// StringToTokenBoxのテスト
test "tokenize keywords" {
    local box = StringToTokenBox()
    local tokens = box.transform("me loop from")
    assert tokens == [Token::Me, Token::Loop, Token::From]
}

// ASTToMIRBoxのテスト
test "binary op to MIR" {
    local box = ASTToMIRBox()
    local ast = AST::BinaryOp("+", 
        AST::Literal(Value::Integer(1)),
        AST::Literal(Value::Integer(2))
    )
    local mir = box.transform(ast)
    assert mir == MIR::AddI64(
        MIR::Const(Value::Integer(1)),
        MIR::Const(Value::Integer(2))
    )
}
```

## 📊 依存グラフ: 完全なDAG（有向非巡環グラフ）

```
StringToTokenBox (依存: 0)
    ↓
TokenToASTBox (依存: 0)
    ↓
ASTToMIRBox (依存: 0)
    ↓               ↓
MIRToVMBox (依存: 0)  MIRToJITBox (依存: 0)

TypeCoercionBox (依存: 0)
    ↓
OperatorExecutorBox (依存: 1)
```

## 🚀 この設計の利点

### 1. 真の疎結合
- 各箱は入力と出力の型だけを知る
- 他の箱の実装を一切知らない
- インターフェースすら不要（型だけで十分）

### 2. 並行開発可能
- チームAがTokenizer開発
- チームBがParser開発
- チームCがMIR開発
- 全員が独立して作業可能

### 3. 差し替え可能
```rust
// 別実装への差し替えが容易
local pipeline = NyashPipeline()
pipeline.tokenizer = OptimizedStringToTokenBox()  // 高速版
pipeline.parser = ErrorRecoveringTokenToASTBox()  // エラー回復版
```

### 4. 段階的最適化
```rust
// 最適化も箱として追加
box MIROptimizerBox {
    transform(mir: MIR) -> MIR {
        // 定数畳み込み、死んだコード除去など
        return optimized
    }
}

// パイプラインに挿入
local mir = ASTToMIRBox().transform(ast)
mir = MIROptimizerBox().transform(mir)  // 追加
local bytecode = MIRToVMBox().transform(mir)
```

## 🎯 まとめ: 根を完全に切る

1. **データ中心設計**: 各層は入力データを出力データに変換するだけ
2. **状態を持たない**: すべての箱が純粋関数的
3. **設定と実装の分離**: ビルド時と実行時を明確に分離
4. **変換の連鎖**: パイプラインで箱をつなぐ

これにより、真に「根が這わない」アーキテクチャが実現されます。