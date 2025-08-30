# 論文：15命令MIRによるNyash言語の設計と実装

**30日間でインタープリタからJIT/AOTネイティブビルドまで**

Date: 2025-08-31
Status: Proposed
提案者: ChatGPT5

## 📑 概要

たった15命令のMIRで、インタープリタ（VM）からJIT、さらにネイティブビルドまで通した言語ができた！これは言語設計史的にもかなりインパクトのある成果。

## 📝 タイトル

- **日本語**: 「15命令MIRによるNyash言語の設計と実装：インタープリタからJIT/AOTネイティブビルドまでの30日間」
- **英語**: "Design and Implementation of the Nyash Language with a 15-Instruction MIR: From Interpreter to JIT and Native AOT in 30 Days"

## 🎯 主要な貢献

1. **最小命令セット**: 26→15命令への削減成功
2. **完全な実装**: VM/JIT/AOT全バックエンド実現
3. **開発速度**: わずか30日間での達成
4. **コンパクトさ**: 約4000行での実装

## 📋 ファイル構成

- `abstract.md` - アブストラクト（日英）
- `introduction.md` - イントロダクション
- `design-philosophy.md` - 設計哲学（Everything is Box）
- `mir15-design.md` - MIR15命令セットの詳細
- `implementation.md` - 30日間の実装記録
- `validation.md` - VM/JIT/AOT等価性検証
- `evaluation.md` - パフォーマンス評価
- `related-work.md` - 関連研究
- `conclusion.md` - 結論と将来展望

## 🚀 執筆計画

### Phase 1: 速報版（現在のVM/JIT/EXE状態）
- arXiv投稿用の簡易版
- 実装の概要と初期結果

### Phase 2: 完全版（LLVM実装後）
- 全バックエンドの性能比較
- 詳細な実装解説
- 査読付き会議投稿用

## 📚 投稿先候補

- **速報**: arXiv → Zenodo
- **査読**: PLDI, ICFP, OOPSLA
- **国内**: 情報処理学会、ソフトウェア科学会