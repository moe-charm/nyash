# Phase 10: Function Declaration MIR サポート課題

作成日: 2025-08-27
発見者: Claude & ChatGPT5 & ニャー

## 🚨 現在の問題

### 症状
```bash
❌ MIR compilation error: Unsupported AST node type: FunctionDeclaration
```

### 影響範囲
- 通常の`function`定義がMIRでコンパイルできない
- JITテストで関数を使った配列操作ができない
- Static Boxメソッドは動作するが、通常関数が使えない

### 具体例
```nyash
// ❌ 動作しない - FunctionDeclarationがMIR未対応
function len_of(arr) {
    return arr.length()
}

// ✅ 動作する - Static Boxメソッド
static box Utils {
    len_of(arr) {
        return arr.length()
    }
}

// ✅ 動作する - 直接呼び出し
local arr = new ArrayBox()
local len = arr.length()
```

## 🔧 解決案

### Option 1: MIRビルダーにFunctionDeclaration対応追加
```rust
// mir/builder.rs に追加
AstNode::FunctionDeclaration { name, params, body, .. } => {
    // 関数をMirFunctionとして登録
    let mir_func = self.build_function(name, params, body)?;
    self.functions.insert(name.clone(), mir_func);
}
```

### Option 2: ASTレベルでStatic Boxに変換
- FunctionDeclarationを内部的にStatic Boxメソッドに変換
- 互換性を保ちつつ既存の仕組みを活用

### Option 3: 当面の回避策を公式化
- ドキュメントで「VMモードではStatic Boxメソッドを推奨」と明記
- 将来のサポートとして計画に含める

## 📊 優先度評価

- **重要度**: 中（基本的な言語機能だが回避策あり）
- **緊急度**: 低（Static Boxで代替可能）
- **実装難度**: 中（MIRビルダーの拡張が必要）

## 🎯 推奨アクション

1. **短期**: Static Boxメソッドを使った回避策をドキュメント化
2. **中期**: Phase 10.1でFunctionDeclaration対応を実装
3. **長期**: 関数定義の最適化（インライン化等）も検討

## 📝 関連イシュー
- JIT配列操作テスト
- MIRビルダー拡張
- 言語仕様の完全性