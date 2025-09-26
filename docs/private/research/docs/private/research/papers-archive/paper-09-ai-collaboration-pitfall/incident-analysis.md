# 事件分析：Python統合における型システム特殊化の危機

## 1. 背景
- 開発開始から1ヶ月でインタープリター→VM→JIT→AOT/ネイティブまで到達
- 最終段階でPython統合（PyRuntimeBox）を実装中
- eval方式では完全にAOT対応（unsupported=0）達成
- import/getattr/callの実装で問題発生

## 2. ChatGPT5の提案（技術的には正しい）
```
Lowerer強化（Step 2）: 
PyRuntimeBox.import/getattr → PyObjectBox
PyObjectBox.call を emit_plugin_invoke で実装
```

### 提案の内容
- Lowererで型推論を実装
- `py.import("math")` → PyObjectBox型として記録
- `math.getattr("sqrt")` → 型情報を伝搬
- `sqrt.call(16)` → 適切なplugin_invokeを生成

## 3. 潜在的な危険性

### 3.1 設計哲学の崩壊
```
Nyash: Everything is Box
　↓
Python専用の型システム導入
　↓
Everything is... Special Case???
```

### 3.2 実装の肥大化
```rust
// もし実装していたら...
match (receiver_type, method) {
    ("PyRuntimeBox", "import") => "PyObjectBox",
    ("FileBox", "open") => "FileHandle",
    ("DBBox", "query") => "ResultSet",
    ("GameBox", "spawn") => "GameObject",
    // 無限に増える特殊ケース
}
```

### 3.3 保守性の悪夢
- 新しいプラグインごとにコンパイラ改修
- JIT/Lowererが特定プラグインに依存
- 汎用性の完全な喪失

## 4. 危機回避の瞬間

### 開発者の直感的疑問
```
「ん？大丈夫？JITのpython用のハードコーディングにならない？汎用的につかえるやつ？」
```

### この一言が引き金となり：
1. 設計の本質的な問題に気づく
2. ChatGPT5も即座に方向転換
3. Handle-First の汎用設計へ

## 5. 根本原因の分析

### 5.1 爆速開発の心理
- 「あと少しで完成」の興奮
- 批判的思考の低下
- 目の前の問題解決に集中しすぎ

### 5.2 パターン認識の罠
- 他言語の常識（Module型、Function型）を持ち込む
- Nyashの独自性を忘れる

### 5.3 AI協調の盲点
- 技術的解決に集中
- 「そもそも論」を問わない
- お互いの提案を無批判に受け入れる傾向

## 6. 結論
技術的に正しい解決策が、必ずしも設計哲学的に正しいとは限らない。「Everything is Box」という原則を守ることで、より美しく保守性の高い設計が維持された。