# ゼロ知識文法アーキテクチャ - 究極の疎結合

## 🔍 さらに深い問題: 暗黙知識の漏洩

### 現在の設計でもまだ残る問題
```rust
// 🚨 TokenToASTBoxがTokenの意味を知っている
transform(tokens: TokenStream) -> AST {
    if token == Token::Me {  // Tokenの意味を知っている！
        return AST::SelfReference
    }
}

// 🚨 ASTToMIRBoxがASTの構造を知っている
transform(ast: AST) -> MIR {
    match ast {
        AST::BinaryOp(op, left, right) => {  // AST構造を知っている！
            // ...
        }
    }
}
```

## 🎯 ゼロ知識原則: 「箱は変換ルールだけを知る」

### 純粋な変換テーブル駆動設計

```rust
// 各箱は変換テーブルだけを持つ
box TokenClassifierBox {
    init { table: Map<String, u32> }  // 文字列→数値のマッピングのみ
    
    classify(word: String) -> u32 {
        return me.table.get(word).unwrapOr(0)  // 0 = unknown
    }
}

// ビルド時に生成される純粋なマッピング
const TOKEN_TABLE: Map<String, u32> = {
    "me" => 1,
    "from" => 2,
    "loop" => 3,
    // ...
}
```

## 📊 統一中間表現（UIR: Unified Intermediate Representation）

### すべての層が数値タグで通信

```
Source Code     UIR Tags        Execution
-----------     --------        ---------
"me"         →  [1]          →  LoadLocal(0)
"+"          →  [100]        →  Add
"loop"       →  [200]        →  Branch
1 + 2        →  [300,1,300,2,100] → Const(1), Const(2), Add
```

### UIRTag: 意味を持たない純粋な識別子
```rust
box UIRTag {
    init { id: u32, children: Array<UIRTag> }
    
    // タグは意味を持たない、ただの番号
    isLeaf() { return me.children.isEmpty() }
    getChildren() { return me.children }
}
```

## 🔄 完全分離された変換パイプライン

### 1. 字句解析: 文字列→UIRタグ
```rust
box LexicalTransformerBox {
    init { charTable: Array<u32> }  // 文字→タグのテーブル
    
    transform(text: String) -> Array<UIRTag> {
        local tags = []
        local chars = text.chars()
        
        loop(chars.hasNext()) {
            local ch = chars.next()
            local tag = me.charTable[ch.code()]
            
            if tag == TAG_LETTER {
                local word = me.collectWhile(chars, TAG_LETTER)
                tags.push(me.lookupWord(word))
            } else if tag == TAG_DIGIT {
                local num = me.collectWhile(chars, TAG_DIGIT)
                tags.push(UIRTag(TAG_NUMBER, num))
            }
            // ...
        }
        return tags
    }
    
    // 単語検索も純粋なハッシュ値
    lookupWord(word: String) -> UIRTag {
        local hash = me.perfectHash(word)
        return UIRTag(hash, [])
    }
}
```

### 2. 構文解析: UIRタグ→UIRツリー
```rust
box SyntaxTransformerBox {
    init { 
        // 優先順位テーブル（タグ→優先度）
        precedence: Map<u32, u32>,
        // 結合性テーブル（タグ→左/右）
        associativity: Map<u32, u8>
    }
    
    transform(tags: Array<UIRTag>) -> UIRTag {
        // Prattパーサーだが、意味を知らない
        return me.parseExpression(tags, 0)
    }
    
    parseExpression(tags: Array<UIRTag>, minPrec: u32) -> UIRTag {
        local left = me.parsePrimary(tags)
        
        loop(tags.hasNext()) {
            local op = tags.peek()
            local prec = me.precedence.get(op.id).unwrapOr(0)
            
            if prec < minPrec { break }
            
            tags.next()  // consume operator
            local assoc = me.associativity.get(op.id).unwrapOr(LEFT)
            local nextPrec = if assoc == LEFT { prec + 1 } else { prec }
            local right = me.parseExpression(tags, nextPrec)
            
            // 構造だけ作る、意味は知らない
            left = UIRTag(op.id, [left, right])
        }
        
        return left
    }
}
```

### 3. 意味解析: UIRツリー→実行可能形式
```rust
box SemanticTransformerBox {
    init {
        // タグ→実行アクションのテーブル
        actions: Map<u32, ExecutionAction>
    }
    
    transform(tree: UIRTag) -> ExecutableCode {
        local action = me.actions.get(tree.id)
        
        if action {
            return action.generate(tree.children.map(child => {
                me.transform(child)
            }))
        }
        
        return ExecutableCode.Noop()
    }
}
```

## 📐 ビルド時の統一: マスターテーブル生成

### grammar.yaml → 各種テーブル生成
```yaml
# grammar.yaml - 真の単一情報源
tokens:
  me: { id: 1, type: self_reference }
  from: { id: 2, type: delegation }
  loop: { id: 3, type: control_flow }

operators:
  "+": { id: 100, precedence: 10, associativity: left }
  "*": { id: 101, precedence: 20, associativity: left }

semantics:
  1: { action: load_self }
  2: { action: delegate_call }
  3: { action: loop_construct }
  100: { action: add_operation }
```

### ビルド時生成
```rust
// build.rs
fn generate_tables(grammar: GrammarDef) {
    // 1. 完全ハッシュ関数生成
    generate_perfect_hash(grammar.tokens)
    
    // 2. 優先順位テーブル生成
    generate_precedence_table(grammar.operators)
    
    // 3. セマンティクステーブル生成
    generate_semantic_table(grammar.semantics)
    
    // 4. 各層の定数生成
    generate_constants(grammar)
}
```

## 🎯 究極の利点: 完全な知識分離

### 1. 各箱が知っていること
- **LexicalTransformer**: 文字の分類とハッシュ計算のみ
- **SyntaxTransformer**: 優先順位と結合性のみ
- **SemanticTransformer**: タグとアクションの対応のみ

### 2. 各箱が知らないこと
- **すべての箱**: 他の層の存在、Nyashという言語名すら知らない
- **すべての箱**: キーワードの意味、演算子の意味
- **すべての箱**: 最終的な実行形式

### 3. テストの単純化
```rust
test "lexical transformer" {
    local table = { "hello" => 42 }
    local box = LexicalTransformerBox(table)
    assert box.transform("hello") == [UIRTag(42)]
}

test "syntax transformer" {
    local prec = { 100 => 10, 101 => 20 }
    local box = SyntaxTransformerBox(prec, {})
    // 1 + 2 * 3
    local tags = [UIRTag(1), UIRTag(100), UIRTag(2), UIRTag(101), UIRTag(3)]
    local tree = box.transform(tags)
    // 期待: (+ 1 (* 2 3))
    assert tree == UIRTag(100, [
        UIRTag(1),
        UIRTag(101, [UIRTag(2), UIRTag(3)])
    ])
}
```

## 🔧 動的拡張: プラグインテーブル

### 実行時のテーブル拡張
```rust
box PluginLoaderBox {
    init { transformers: Map<String, TransformerBox> }
    
    loadPlugin(path: String) {
        local plugin = Plugin.load(path)
        
        // プラグインは新しいタグを登録
        local newTags = plugin.getTags()
        
        // 各変換器のテーブルを拡張
        me.transformers.get("lexical").extendTable(newTags.lexical)
        me.transformers.get("syntax").extendTable(newTags.syntax)
        me.transformers.get("semantic").extendTable(newTags.semantic)
    }
}
```

## 📊 性能特性

### 1. キャッシュ効率
- 各テーブルは連続メモリに配置
- CPUキャッシュに収まるサイズ
- ランダムアクセスなし

### 2. 並列化可能
- 各変換は状態を持たない
- 入力を分割して並列処理可能
- ロックフリー実装

### 3. 最適化の余地
- テーブルのコンパクト化
- SIMDによる並列検索
- JITによるテーブル特化

## 🚀 最終形: 言語に依存しない変換エンジン

```rust
// このエンジンはNyashを知らない！
box UniversalTransformEngine {
    init { 
        pipeline: Array<TransformerBox>,
        tables: Map<String, Table>
    }
    
    execute(input: String) -> Output {
        local data = input
        
        // 各変換を順番に適用
        me.pipeline.forEach(transformer => {
            data = transformer.transform(data)
        })
        
        return data
    }
}

// Nyash = 特定のテーブルセット
const NYASH_TABLES = load_tables("nyash-grammar.yaml")
local engine = UniversalTransformEngine(STANDARD_PIPELINE, NYASH_TABLES)
```

これが究極の「根を切った」設計です。各箱は純粋な変換器であり、Nyashという言語の存在すら知りません。