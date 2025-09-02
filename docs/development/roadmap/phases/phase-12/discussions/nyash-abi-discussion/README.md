# Nyash ABI議論アーカイブ

このディレクトリには、Nyash ABIの設計に関する重要な議論や考察が保存されています。

## 📚 ドキュメント一覧

### 🌟 AI専門家の深い考察
- **[gemini-codex-deep-thoughts.md](gemini-codex-deep-thoughts.md)** - Gemini先生とCodex先生によるNyash ABI C実装の深い考察（2025年9月2日）
  - セルフホスティング実現への技術的妥当性
  - 具体的な実装提案（16バイトアライメント、セレクターキャッシング等）
  - 哲学的観点からの分析

### 🎯 設計の進化過程
- 初期提案: 複雑なFactory設計
- 転換点: 「型をC ABIの箱として渡す共通ルール」という洞察
- 最終形: TypeBox + Nyash ABI C実装による統一設計

## 💡 重要な洞察

### 「Everything is Box」の究極形
ABIそのものをBoxとして扱うことで、言語の哲学が技術的実装と完全に一致しました。

### セルフホスティングへの道
1. C Shim実装（既存Rustへのラッパー）
2. フルC実装（基本型・参照カウント）
3. Nyashで再実装（AOTでC ABI公開）
4. Nyashコンパイラ自身をNyashで実装

### AI専門家の評価
- **Gemini**: 「技術的妥当性が高く、言語哲学とも合致した、極めて優れた設計」
- **Codex**: 「Feasible and attractive: ABI-as-Box completes the idea」
- **ChatGPT5**: 「実装に耐える設計。10の改善点で完璧」

## 📝 関連ドキュメント
- [../NYASH-ABI-C-IMPLEMENTATION.md](../NYASH-ABI-C-IMPLEMENTATION.md) - 実装仕様書
- [../UNIFIED-ABI-DESIGN.md](../UNIFIED-ABI-DESIGN.md) - 統合ABI設計
- [../README.md](../README.md) - Phase 12概要