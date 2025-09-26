# Pattern Matching基盤実装計画 - Macro Revolution前哨戦

**策定日**: 2025-09-18  
**優先度**: 最高（Gemini/Codex推奨）
**期間見積もり**: 2週間
**依存関係**: Phase 16マクロ実装の必須前提条件

## 🎯 なぜPattern Matchingが最優先なのか？

### Gemini の洞察
> **Pattern Matchingは、代数的データ型を扱うための根源的な機能です。これにより、条件分岐やデータ分解のロジックが劇的にクリーンで安全になります。**

### Codex の技術的理由
> **ASTは複雑なツリー構造** → **Pattern Matchingが最適なツール**
> **マクロシステム自体の実装がクリーンになる**
> **膨大なif let、switch、visitorパターンの回避**

### 戦略的重要性
1. **基礎機能**: 言語のデータ操作能力という土台を固める
2. **マクロ実装ツール**: ASTパターンマッチングで安全な操作が可能
3. **段階的成功**: 単体でも価値の明確な機能
4. **実装準備**: マクロシステムの基盤ツールとして機能

## 🏗️ 実装アーキテクチャ設計

### Pattern Matching構文設計
```nyash
// 基本的なmatch式
local result = match value {
    0 => "zero",
    1 => "one", 
    2..10 => "small",
    _ => "other"
}

// Box destructuring（構造パターン）
match user_box {
    UserBox(name, age) => {
        print("User: " + name + ", Age: " + age)
    },
    AdminBox(name, permissions) => {
        print("Admin: " + name)
    },
    _ => {
        print("Unknown box type")
    }
}

// ガード付きパターン
match request {
    HttpRequest(method, path) if method == "GET" => handle_get(path),
    HttpRequest(method, path) if method == "POST" => handle_post(path),
    HttpRequest(method, _) => error("Unsupported method: " + method)
}

// ネストした構造パターン
match response {
    Ok(UserBox(name, ProfileBox(email, age))) => {
        print("Success: " + name + " (" + email + ")")
    },
    Err(ErrorBox(code, message)) => {
        print("Error " + code + ": " + message)
    }
}
```

### AST表現設計
```rust
// Rust側での内部表現
#[derive(Debug, Clone)]
pub enum Pattern {
    // リテラルパターン
    Literal(LiteralValue),
    
    // 変数バインディング
    Variable(String),
    
    // ワイルドカード
    Wildcard,
    
    // 範囲パターン
    Range { start: Box<Pattern>, end: Box<Pattern> },
    
    // 構造パターン（Box destructuring）
    Struct { 
        box_name: String, 
        fields: Vec<Pattern> 
    },
    
    // OR パターン
    Or(Vec<Pattern>),
    
    // ガード付きパターン
    Guard { pattern: Box<Pattern>, condition: Expr },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Expr,  // マッチ対象
    pub arms: Vec<MatchArm>,
}
```

## 🔧 実装フェーズ

### Phase PM.1: 基本構文実装（1週間）

#### Day 1-2: パーサー拡張
- [ ] `match` キーワードの追加
- [ ] パターン構文の解析
- [ ] `=>` 記号の処理
- [ ] ガード条件（`if`）の解析

#### Day 3-4: AST構築
- [ ] Pattern/MatchExpr AST nodes
- [ ] パターンバインディングの処理
- [ ] スコープ管理の実装
- [ ] 型検査の基礎

#### Day 5-7: 基本動作確認
- [ ] リテラルパターンのテスト
- [ ] 変数バインディングのテスト  
- [ ] ワイルドカードのテスト
- [ ] 基本的なmatch式の動作確認

### Phase PM.2: 高度パターン実装（1週間）

#### Day 8-10: 構造パターン
- [ ] Box destructuring実装
- [ ] ネストした構造の処理
- [ ] フィールド名による分解
- [ ] 型安全性の確保

#### Day 11-12: 範囲・ガード
- [ ] 範囲パターン（`1..10`）
- [ ] ガード条件（`if`）
- [ ] 複雑な条件式の処理
- [ ] パフォーマンス最適化

#### Day 13-14: 統合テスト
- [ ] 複雑なパターンの組み合わせ
- [ ] エラーハンドリング
- [ ] 網羅性チェック（exhaustiveness）
- [ ] コード生成の確認

## 🎯 MIR Lowering戦略

### Pattern Matching → MIR14変換

```nyash
// 入力コード
match value {
    0 => "zero",
    1..5 => "small", 
    _ => "other"
}
```

```
// 生成されるMIR14（概念的）
%temp1 = compare %value, 0
branch %temp1, @case_zero, @check_range

@check_range:
%temp2 = compare %value, 1, gte
%temp3 = compare %value, 5, lte
%temp4 = binop %temp2, %temp3, and
branch %temp4, @case_small, @case_default

@case_zero:
%result = const "zero"
jump @match_end

@case_small:
%result = const "small"
jump @match_end

@case_default:
%result = const "other"
jump @match_end

@match_end:
// %result contains the final value
```

### 最適化戦略
- **Jump table**: 整数パターンの最適化
- **Decision tree**: 複雑なパターンの効率的分岐
- **Exhaustiveness**: コンパイル時の網羅性チェック

## 🧪 テスト戦略

### 最小テストケース
```nyash
// Test 1: 基本パターン
local result1 = match 42 {
    0 => "zero",
    42 => "answer",
    _ => "other"
}
assert result1 == "answer"

// Test 2: 範囲パターン  
local result2 = match 7 {
    1..5 => "small",
    6..10 => "medium",
    _ => "large"
}
assert result2 == "medium"

// Test 3: Box destructuring
local user = new UserBox("Alice", 25)
local greeting = match user {
    UserBox(name, age) => "Hello " + name + "!",
    _ => "Hello stranger!"
}
assert greeting == "Hello Alice!"

// Test 4: ガード条件
local result4 = match 15 {
    x if x < 10 => "small",
    x if x < 20 => "medium", 
    _ => "large"
}
assert result4 == "medium"
```

### エラーケース
```nyash
// 網羅性エラー（意図的）
match value {
    0 => "zero"
    // エラー: 他のケースが網羅されていない
}

// 型エラー（意図的）
match string_value {
    42 => "number"  // エラー: 型が合わない
}

// 到達不可能コード（意図的）
match value {
    _ => "catch all",
    0 => "unreachable"  // 警告: 到達不可能
}
```

## 🔗 マクロシステムとの統合準備

### AST Pattern Matching API

Pattern Matchingが完成すると、マクロ実装で以下のAPIが使用可能になる：

```rust
// マクロでのAST操作例
fn expand_derive_equals(input: &AstNode) -> Result<AstNode> {
    match input {
        AstNode::BoxDef { name, fields, .. } => {
            // パターンマッチングでBoxの構造を安全に分解
            let method_body = generate_equals_body(fields)?;
            Ok(AstNode::MethodDef {
                name: "equals".to_string(),
                body: method_body,
                ..
            })
        },
        _ => Err("@derive can only be applied to box definitions")
    }
}
```

### マクロ展開での活用例
```nyash
// マクロが受け取るAST
box UserBox {
    name: StringBox
    age: IntegerBox  
}

// パターンマッチングによる安全な変換
match target_ast {
    BoxDef(name, fields) => {
        match fields {
            [Field("name", StringBox), Field("age", IntegerBox)] => {
                // 特定の構造に対する最適化生成
                generate_optimized_equals(name, fields)
            },
            _ => {
                // 汎用的な生成
                generate_generic_equals(name, fields)
            }
        }
    }
}
```

## 📋 完了条件と受け入れ基準

### Phase PM.1完了条件
- [ ] 基本的なmatch式が動作
- [ ] リテラル・変数・ワイルドカードパターン実装
- [ ] MIR14への正常なLowering
- [ ] PyVM・LLVMバックエンドで実行可能

### Phase PM.2完了条件  
- [ ] Box destructuringが動作
- [ ] 範囲パターン・ガード条件実装
- [ ] 網羅性チェックの基本実装
- [ ] 複雑なネストパターンの処理

### 最終受け入れ基準
- [ ] 全テストケースの通過
- [ ] エラーメッセージの品質確保
- [ ] パフォーマンス基準の達成
- [ ] マクロシステム実装への準備完了

## 🚀 次のステップ

### Pattern Matching完了後
1. **AST操作基盤実装**（Phase 16.2）
2. **HIRパッチエンジン設計**（Phase 16.3）
3. **@derive(Equals)最小実装**（Phase 16.4）

### 期待される効果
- **マクロ実装の土台**が確実に構築される
- **複雑なAST操作**が安全かつ簡潔に記述可能
- **コード品質**の大幅な向上
- **開発効率**の革命的改善

---

**Pattern Matching基盤により、Nyash Macro Revolutionの成功が確実になる。**

*「まず土台を固める」- 全AI賢者の一致した戦略的判断。*