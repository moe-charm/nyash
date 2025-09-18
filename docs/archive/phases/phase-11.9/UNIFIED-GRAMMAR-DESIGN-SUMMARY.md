# Phase 11.9 統一文法設計 - 総合まとめ

## 📋 概要

Nyashの各実行層（Tokenizer/Parser/Interpreter/MIR/VM/JIT）で予約語・文法解釈がバラバラに実装されている問題を解決する統一文法アーキテクチャ設計のまとめです。

## 🎯 核心的な問題

```rust
// 現在: 同じ "me" が6箇所で別々に定義
Tokenizer:   "me" → TokenType::ME
Parser:      独自のme処理ロジック
Interpreter: 独自のself参照実装
MIR Builder: LoadLocal(0)への変換
VM:          OP_LOAD_MEの実行
JIT:         LoadFirstParamの生成
```

## 💡 提案された解決策

### 1. 基本アプローチ: 統一文法エンジン
- 単一の文法定義（YAML/TOML）
- 各層が参照する統一API
- UnifiedSemantics による一貫した実行

### 2. AI提案: ビルド時コード生成
- **Gemini**: 宣言的定義 + build.rs によるコード生成
- **Codex**: MIR中心の統一セマンティクス基盤
- 実行時オーバーヘッドゼロ

### 3. 箱化による疎結合設計
- 各層を独立した「箱」として実装
- 変換箱（TransformerBox）パターン
- パイプライン方式での連結

## 📊 実装アプローチの比較

| アプローチ | 利点 | 欠点 | 推奨度 |
|---------|------|------|-------|
| 統一エンジン | シンプル、理解しやすい | 実行時オーバーヘッド | ★★★ |
| コード生成 | 高性能、型安全 | ビルド複雑化 | ★★★★★ |
| 完全箱化 | 究極の疎結合 | 実装複雑度高 | ★★★★ |

## 🚀 推奨実装計画

### Phase 1: 文法定義ファイル作成
```yaml
# grammar/nyash.yml
tokens:
  me: { id: 1, category: self_reference }
  from: { id: 2, category: delegation }
  loop: { id: 3, category: control_flow }

operators:
  "+": { precedence: 10, associativity: left }
```

### Phase 2: コード生成基盤
```rust
// build.rs
fn generate_from_grammar() {
    // grammar.yml → generated/*.rs
}
```

### Phase 3: 段階的移行
1. Tokenizer を生成コードに移行
2. Parser を統一文法に移行
3. Semantics を一元化
4. MIR/VM/JIT を統合

## 🎯 期待される効果

1. **保守性向上**: 新機能追加が1箇所で完了
2. **一貫性確保**: 全層で同じセマンティクス
3. **AI対応改善**: LLMが正確なコードを生成
4. **性能維持**: ビルド時最適化でオーバーヘッドなし

## 📁 作成されたドキュメント

### 必須ドキュメント（実装に必要）
1. **[統一文法アーキテクチャ設計書](unified-grammar-architecture.md)** - 基本設計
2. **[統一予約語システム仕様](unified-keyword-system.md)** - 具体的実装仕様
3. **[AI深層考察](ai-deep-thoughts-unified-grammar.md)** - Gemini/Codex分析

### 発展的ドキュメント（参考資料）
4. **[Box-First文法アーキテクチャ](box-first-grammar-architecture.md)** - 箱化アプローチ
5. **[根切り文法アーキテクチャ](root-cutting-architecture.md)** - 完全疎結合設計
6. **[ゼロ知識文法アーキテクチャ](zero-knowledge-architecture.md)** - 究極の分離設計

### 既存ドキュメント
- [文法統一化詳細設計](grammar-unification.txt)
- [統一文法定義YAML](nyash-grammar-v1.yaml)
- [実装計画](implementation-plan.txt)

## 🔧 次のステップ

1. `grammar/nyash.yml` の初版作成
2. `crates/nygrammar-gen` の実装開始
3. Tokenizer の移行から着手
4. 段階的に全層を統一

## 📝 結論

コード生成アプローチ（Gemini/Codex推奨）を採用し、`grammar/nyash.yml` を単一の真実の源として、build.rs で各層向けのコードを生成する方式が最も実用的です。

これにより、Nyashの文法が完全に統一され、保守性・一貫性・AI対応すべてが改善されます。