# Paper E: LoopSignal IR - Unifying Control as Values

## 📊 論文概要

**タイトル候補**:
- "LoopSignal IR: Unifying Control Structures as Values in Intermediate Representation"
- "Signal Loop IR: A Value-based Approach to Control Flow Unification"
- "Boxed Loop Semantics: Bridging Theory and Implementation in Language Design"

**主要な貢献**:
1. 制御構造（分岐/ループ/関数/スコープ）を統一的な値（Signal）として扱うIR設計
2. 「Everything is Box」×「Everything is Loop」による空間・時間の統一
3. 理論（CPS/継続/代数的効果）と実装の実用的な橋渡し

## 🎯 ポジショニング

### 既存研究との差分
- **CPS/継続**: 汎用だが複雑。本提案は最小Signal集合で実装容易
- **代数的効果**: 強力だが実装困難。本提案はloop.*命令に限定して実用性優先
- **コルーチン**: 特定用途。本提案は関数/スコープまで統一

### Nyashとの相性
- Box = 空間的統一（データ構造）
- Loop = 時間的統一（制御構造）
- 両者の組み合わせで完全な統一を実現

## 📁 ディレクトリ構造

```
paper-e-loopsignal-ir/
├── README.md           # このファイル
├── abstract.md         # 論文概要（150-200語）
├── main-paper-jp.md    # 日本語版メイン論文
├── main-paper-en.md    # 英語版メイン論文（後日）
├── RESEARCH.md         # 詳細な研究ノート
├── figures/            # 図表
│   ├── loop-unification.png
│   ├── lowering-example.png
│   └── performance-metrics.png
└── evaluation/         # 評価データ
    ├── metrics.md
    └── benchmarks/
```

## 🚀 現在の状態

- [x] 基本概念の整理
- [x] ChatGPT5との議論による深化
- [ ] RESEARCH.mdの詳細化
- [ ] 実装計画の具体化
- [ ] 評価指標の設定

## 📝 次のステップ

1. RESEARCH.mdに詳細な仕様を記述
2. 論文A（MIR13）の将来展望に1段落追加
3. 実装は論文A/B投稿後に着手

## 🔗 関連ドキュメント

- [論文A: MIR13命令とIR設計](../paper-a-mir13-ir-design/)
- [論文B: Nyash実行モデル](../paper-b-nyash-execution-model/)
- [CURRENT_TASK.md](../../../../CURRENT_TASK.md)