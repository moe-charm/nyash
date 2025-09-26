# ANCP (AI-Nyash Compact Notation Protocol) 仕様書

このフォルダには、ANCP圧縮技法に関する全ての仕様書と技術文書が含まれています。

## 📄 ドキュメント一覧

### 🎯 中核仕様
- **[ANCP-Token-Specification-v1.md](ANCP-Token-Specification-v1.md)** - ChatGPT5作成のトークン仕様書 v1.0
  - P（Pretty）→ C（Compact）変換の完全仕様
  - EBNF文法定義
  - トークン変換ルール
  - 衝突回避メカニズム

### 🔥 圧縮体系
- **[ULTIMATE-AI-CODING-GUIDE.md](ULTIMATE-AI-CODING-GUIDE.md)** - 5層圧縮体系の統合ガイド
  - L0: Standard (通常のNyash)
  - L1: Pretty (整形済み)
  - L2: Compact (48%圧縮)
  - L3: Sugar (75%圧縮)
  - L4: Fusion (90%圧縮)

### ⚡ 糖衣構文
- **[extreme-sugar-proposals.txt](extreme-sugar-proposals.txt)** - 極限糖衣構文の提案集
  - パイプライン演算子 `|>`
  - 安全アクセス演算子 `?.`
  - ディレクティブ記法 `/:`
  - その他の革新的構文

### 🔄 ツール仕様
- **[sugar-formatter-tool.txt](sugar-formatter-tool.txt)** - 可逆フォーマッターの設計
  - 双方向変換の保証
  - ソースマップ2.0仕様
  - VSCode統合計画

### 📚 参考資料
- **[compression-reference-libraries.md](compression-reference-libraries.md)** - 関連技術の調査
  - 既存圧縮ツールの比較
  - 学術研究の参照
  - 実装のヒント

## 🚀 実装優先順位

1. **Week 1**: ANCP-Token-Specification-v1 に基づく基本実装
2. **Week 2**: 糖衣構文の統合
3. **Week 3**: Fusion層（F）の追加
4. **Week 4**: ツール・IDE統合

## 💡 重要な設計原則

- **完全可逆性**: P ↔ C ↔ F の変換で情報損失ゼロ
- **安全性優先**: 文字列・コメント内は変換しない
- **段階的導入**: まずCから、次にF層へ
- **AI最適化**: トークン削減率を最大化

---

最新の仕様については、ANCP-Token-Specification-v1.md を参照してください。