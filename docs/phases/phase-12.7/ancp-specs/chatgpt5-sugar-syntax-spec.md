# ChatGPT5糖衣構文仕様書

**Phase 12.7-B実装仕様（2025-09-04作成・更新）**

## 📋 概要

ChatGPT5アドバイザーから提案された糖衣構文を統合し、予約語を増やさずに表現力を劇的に向上させる。

## 🎯 設計原則

1. **予約語を増やさない** - 演算子・記号で実現
2. **可逆変換** - 糖衣構文⇔通常構文の完全な相互変換
3. **曖昧性ゼロ** - パース時の明確な優先順位
4. **MIR13への直接変換** - Phase 15セルフホスティングを意識
5. **使いたい人が使いたい構文を選択** - 強制ではなく選択
6. **超圧縮対応** - AIコンテキスト最大化のための極限記法

## 🔧 実装仕様

### 1. パイプライン演算子（|>）

**構文**
```ebnf
PipeExpr = Expr ( "|>" CallExpr )*
```

**変換規則**
```nyash
# 糖衣構文
x |> f |> g(y) |> h

# デシュガー後
h(g(f(x), y))
```

**MIR変換**
- 一時変数を使った直線的な命令列に変換
- 最適化で一時変数を削減

### 2. セーフアクセス（?.）とデフォルト値（??）

**構文**
```ebnf
SafeAccess = Primary ( ("?." | ".") Identifier )*
NullCoalesce = SafeAccess ( "??" SafeAccess )*
```

**変換規則**
```nyash
# 糖衣構文
user?.profile?.name ?? "Guest"

# デシュガー後
local t0, t1, t2
if user != null {
    t0 = user.profile
    if t0 != null {
        t1 = t0.name
        t2 = t1
    } else {
        t2 = "Guest"
    }
} else {
    t2 = "Guest"
}
```

### 3. デストラクチャリング

**構文**
```ebnf
DestructLet = "let" ( ObjectPattern | ArrayPattern ) "=" Expr
ObjectPattern = "{" Identifier ("," Identifier)* "}"
ArrayPattern = "[" Identifier ("," Identifier)* ("," "..." Identifier)? "]"
```

**変換規則**
```nyash
# オブジェクトパターン
let {x, y} = point
# →
local x = point.x
local y = point.y

# 配列パターン
let [a, b, ...rest] = array
# →
local a = array.get(0)
local b = array.get(1)
local rest = array.slice(2)
```

### 4. 増分代入演算子

**構文**
```ebnf
CompoundAssign = LValue ("+=" | "-=" | "*=" | "/=" | "%=") Expr
```

**変換規則**
```nyash
# 糖衣構文
count += 1
arr[i] *= 2

# デシュガー後
count = count + 1
arr.set(i, arr.get(i) * 2)
```

### 5. 範囲演算子（..）

**構文**
```ebnf
Range = Expr ".." Expr
```

**変換規則**
```nyash
# 糖衣構文
for i in 0..n {
    print(i)
}

# デシュガー後
local _range = new RangeBox(0, n)
for i in _range {
    print(i)
}
```

### 6. 高階関数演算子

**構文（3つの選択肢）**
```ebnf
# 演算子形式（超圧縮向け）
MapOp = Expr "/:" LambdaExpr
FilterOp = Expr "\:" LambdaExpr  
ReduceOp = Expr "//" LambdaExpr

# メソッド形式（バランス型）
MapMethod = Expr ".map" "(" LambdaExpr ")"
FilterMethod = Expr ".filter" "(" LambdaExpr ")"
ReduceMethod = Expr ".reduce" "(" LambdaExpr ["," InitValue] ")"
```

**変換規則（すべて等価）**
```nyash
# 1. 明示的形式（学習・デバッグ向け）
evens = users.filter(function(u) { return u.age >= 18 })
              .map(function(u) { return u.name })

# 2. 糖衣構文メソッド形式（通常開発向け）
evens = users.filter{$_.age >= 18}.map{$_.name}

# 3. 糖衣構文演算子形式（圧縮重視）
evens = users \: {$_.age>=18} /: {$_.name}

# 4. ANCP極限形式（AI協働向け）
e=u\:_.a>=18/:_.n
```

**暗黙変数**
- `$_` - 単一引数の暗黙変数
- `$1`, `$2` - 複数引数の位置指定
- 省略時の`_.`プロパティアクセス（ANCP）

### 7. ラベル付き引数

**構文**
```ebnf
LabeledArg = Identifier ":" Expr
Call = Identifier "(" (LabeledArg | Expr) ("," (LabeledArg | Expr))* ")"
```

**変換規則**
```nyash
# 糖衣構文
Http.request(
    url: "/api",
    method: "POST",
    body: data
)

# デシュガー後
local _args = new MapBox()
_args.set("url", "/api")
_args.set("method", "POST")
_args.set("body", data)
Http.request(_args)
```

## 📊 優先順位表

| 優先度 | 演算子 | 結合性 |
|--------|--------|--------|
| 1 | `?.` | 左結合 |
| 2 | `??` | 左結合 |
| 3 | `\>` | 左結合 |
| 4 | `/:` `\:` `//` | 左結合 |
| 5 | `+=` `-=` etc | 右結合 |
| 6 | `..` | なし |

## 🔄 実装段階

### Stage 1: トークナイザー拡張
- 新しいトークンタイプの追加
- 演算子の最長一致ルール

### Stage 2: パーサー拡張
- 演算子優先順位の実装
- デシュガー変換の実装

### Stage 3: MIR変換
- 効率的なMIR命令列への変換
- 最適化パスの追加

### Stage 4: テスト・ドキュメント
- 包括的なテストケース
- エラーメッセージの改善
- チュートリアル作成

## 🎨 使い分けガイドライン

### 用途別推奨記法
```nyash
# 同じ処理の4段階表現

# 1. 学習用（超明示的）- 60文字
local result = []
for item in data {
    if item.isValid() {
        result.push(transform(normalize(item)))
    }
}

# 2. 通常開発（メソッド糖衣）- 45文字
result = data.filter{$_.isValid()}
    .map{$_ |> normalize |> transform}

# 3. 圧縮開発（演算子糖衣）- 35文字
result = data \: {$_.isValid()} 
    /: {$_ |> normalize |> transform}

# 4. AI協働（ANCP極限）- 20文字
r=d\:_.isValid()/:_|>n|>t
```

**最大67%のコード削減を実現！**

### 可逆変換の保証
```bash
# どの形式からでも相互変換可能
nyash format --from=explicit --to=sugar code.nyash
nyash format --from=sugar --to=ancp code.nyash
nyash format --from=ancp --to=explicit code.nyash
```

## 🚀 Phase 15との相乗効果

セルフホスティングコンパイラでの活用：
```nyash
box MirBuilder {
    // 1. 明示的（デバッグ時）
    buildExpr(ast) {
        local desugared = me.desugar(ast)
        local lowered = me.lower(desugared)
        local checked = me.typeCheck(lowered)
        return me.optimize(checked)
    }
    
    // 2. パイプライン糖衣（通常開発）
    buildExpr(ast) {
        return ast
            |> me.desugar
            |> me.lower
            |> me.typeCheck
            |> me.optimize
    }
    
    // 3. ANCP極限（AIとの共同作業）
    buildExpr(a){r a|>m.desugar|>m.lower|>m.typeCheck|>m.optimize}
}
```

## 💡 重要な設計哲学

**「糖衣構文は使いたい人が使いたいものを選ぶ」**
- 強制ではなく選択
- プロジェクトごとに設定可能
- チームメンバーごとに表示形式を変更可能
- **重要なのは可逆変換できること**

これにより、Nyashは初心者からAI協働まで、あらゆるレベルの開発者に最適な記法を提供します。