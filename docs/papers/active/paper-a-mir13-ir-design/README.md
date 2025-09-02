# 論文A: MIR13命令とIR設計

## 📚 概要

**タイトル**: Minimal Yet Universal: The MIR-13 Instruction Set and Everything-is-Box Philosophy

**主題**: 中間表現（MIR）の統合設計とコンパイラ最適化

**対象読者**: コンパイラ・言語処理系の研究者、PL実装者

## 🎯 研究ポイント

### 1. MIR-13命令セット
- 26命令 → 15命令 → 13命令への段階的削減
- ArrayGet/Set などを BoxCall に吸収する革新的設計
- 最小限でチューリング完全性を保証

### 2. 最適化技術
- **IC（Inline Caching）**: 33倍の高速化
- **AOT（Ahead-of-Time）コンパイル**: ネイティブ性能
- **TypedArray最適化**: 型特化による効率化

### 3. Everything is Box哲学
- すべてをBoxCallに統一する設計思想
- MIRレベルでの哲学の具現化
- 最小の接着剤、無限の可能性

## 📊 実験計画

### ベンチマーク項目
- array_access_sequential: 配列順次アクセス
- array_access_random: 配列ランダムアクセス  
- field_access: フィールド読み書き
- arithmetic_loop: 算術演算ループ

### 性能目標
- 速度: ベースライン ±5%
- メモリ: ベースライン ±10%
- MIRサイズ: -50%削減（26→13命令）

## 📁 ディレクトリ構造

```
paper-a-mir13-ir-design/
├── README.md              # このファイル
├── abstract.md           # 論文概要
├── main-paper.md         # 本文
├── chapters/             # 章別ファイル
│   ├── 01-introduction.md
│   ├── 02-mir-evolution.md
│   ├── 03-boxcall-unification.md
│   ├── 04-optimization-techniques.md
│   ├── 05-evaluation.md
│   └── 06-conclusion.md
├── figures/              # 図表
│   ├── mir-instruction-reduction.png
│   ├── performance-comparison.png
│   └── boxcall-architecture.svg
├── data/                 # 実験データ
│   ├── benchmark-results/
│   └── mir-statistics/
└── related-work.md       # 関連研究

```

## 🗓️ スケジュール

- **2025年9月前半**: 実験実施・データ収集
- **2025年9月中旬**: 執筆開始
- **2025年9月末**: arXiv投稿（速報版）
- **2025年11月**: POPL/PLDI 2026投稿

## 📝 執筆メモ

### 強調すべき貢献
1. **命令数の劇的削減**: 26→13（50%削減）でも性能維持
2. **統一的設計**: BoxCallによる操作の一元化
3. **実用的な性能**: JIT/AOTによる最適化で実用レベル

### 新規性
- 既存のIR（LLVM IR、Java bytecode等）より極小
- Box中心の統一的操作モデル
- 段階的削減による実証的アプローチ

## 🔗 関連ドキュメント

- [MIR Instruction Set](../../../../reference/mir/INSTRUCTION_SET.md)
- [Phase 11.8 MIR Cleanup](../../../../development/roadmap/phases/phase-11.8_mir_cleanup/)
- [Phase 12 TypeBox統合](../../../../development/roadmap/phases/phase-12/)