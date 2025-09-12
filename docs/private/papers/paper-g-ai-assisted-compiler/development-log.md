# 開発ログ: AI支援によるLLVM層構築の記録

## 🔥 主要な問題と解決の軌跡

### 1. PHI配線問題（初期）
```
エラー: PHINode should have one entry for each predecessor
原因: emit側とto側での二重配線、責務の曖昧さ
解決: Sealed SSAによる需要駆動型配線への統一
```

### 2. 支配関係違反（中期）
```
エラー: Instruction does not dominate all uses!
症状: Main.node_json/3, Main.esc_json/1での頻発
原因: 値解決が分散、型変換の場所が不統一
解決: Resolver APIによる統一的値解決
```

### 3. 型システムの混乱
```
問題: i64 handle vs i8* pointer の混在
症状: console.log(handle) → 不正メモリアクセス
解決: nyash.console.log_handle(i64)への変更
```

### 4. LoopForm導入（後期）
```
動機: 既存手法で解決困難、最終手段として投入
効果: 問題を顕在化、PHI集中化への道筋
誤解: 「LoopForm特有の問題」→ 実は既存問題の露呈
```

## 💡 重要な設計決定

### Resolver API
```rust
// Before: 分散した値解決
let val = vmap.get(&value_id)  // どの時点の値？

// After: 統一API
let val = resolver.resolve_i64(value_id, current_bb)
```

### BuilderCursor
- post-terminator挿入を構造的に防止
- 全lowering経路に適用完了

### Python llvmlite導入
- 「簡単最高」の精神
- Rust/inkwellの複雑性からの解放
- 検証ハーネスとしての活用

## 🤖 AI活用の実態

### ChatGPT
- 8分の深い調査「Investigating the builder issue」
- 設計のブレを的確に指摘
- Resolver APIの提案

### Claude
- 日々の実装支援
- エラー解析と解決策提示
- ドキュメント作成

### Gemini
- アーキテクチャ相談
- セルフホスティング戦略議論

## 📊 成果
- MIR14（14命令）でのLLVM層実装
- LoopForm実験的実装
- Resolver API設計・実装
- BuilderCursor全域適用