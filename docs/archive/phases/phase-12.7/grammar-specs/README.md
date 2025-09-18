# Nyash文法改革仕様書

このフォルダには、Phase 12.7で決定されたNyash文法改革の仕様書が含まれています。

## 📄 ドキュメント一覧

### 📝 最終決定事項
- **[grammar-reform-final-decision.txt](grammar-reform-final-decision.txt)** - 文法改革の最終決定
  - 予約語15個への削減
  - peek構文の導入
  - birth統一コンストラクタ
  - フィールド宣言の明示化

### 📐 技術仕様
- **[grammar-technical-spec.txt](grammar-technical-spec.txt)** - 詳細な技術仕様書
  - 構文のBNF定義
  - パーサー実装ガイド
  - 後方互換性の考慮事項

## 🎯 文法改革の要点

### 15個の予約語
```
box, new, me, public, if, else, loop, break, continue, 
peek, return, import, from, birth, fn
```

### 主要な変更点

#### 1. peek構文（switch/case代替）
```nyash
peek value {
    "hello" => print("Hi!")
    42 => print("The answer")
    else => print("Other")
}
```

#### 2. birth統一（コンストラクタ）
```nyash
box Life {
    init { name, energy }
    
    birth(lifeName) {  // すべてのBoxでbirth使用
        me.name = lifeName
        me.energy = 100
    }
}
```

#### 3. fn{}でFunctionBox作成
```nyash
local add = fn{a, b => a + b}
```

#### 4. フィールド宣言の明示化
```nyash
box Person {
    init { name, age }  // フィールドを明示的に宣言
}
```

## 🔄 実装状況

- ✅ 仕様決定完了
- ✅ ChatGPT5による基本実装
- 🔄 テスト作成中
- 📅 完全移行（Phase 12.7-B）

---

詳細な実装については、implementation/フォルダを参照してください。