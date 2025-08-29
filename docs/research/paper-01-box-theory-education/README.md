# 📖 Box-Oriented Programming: A Language Design That Makes Bad Code Impossible

## 📑 論文概要

**タイトル**: Box-Oriented Programming: A Language Design That Makes Bad Code Impossible

**対象会議**: SIGCSE 2026 / ICER 2025

**著者**: [TBD]

**概要**: プログラミング初学者が陥りやすい設計ミスを言語レベルで防ぐ「箱理論」を提案。Nyash言語での実装と教育効果を実証。

## 🎯 研究課題

1. **なぜ初学者は悪いコードを書くのか？**
   - グローバル変数の乱用
   - 不適切な責任分離
   - メモリ管理の混乱

2. **既存の教育アプローチの限界**
   - 「良い設計」の説教
   - 後付けのリファクタリング
   - 抽象的な原則論

3. **提案：言語設計による解決**
   - 悪い設計を文法的に不可能に
   - 箱による自然な責任分離
   - 明示的な境界管理

## 📊 実験計画

### 対照実験
- **グループA**: 従来言語（Python/Java）で学習
- **グループB**: Nyashで学習
- **測定項目**:
  - コード品質メトリクス
  - デバッグ時間
  - 設計パターンの理解度

### 予想される結果
- Nyashグループは自然に良い設計に
- デバッグ時間の大幅削減
- 「なぜ良い設計か」の理解促進

## 📝 論文構成案

```
1. Introduction
   - プログラミング教育の課題
   - 言語設計の教育的影響

2. Related Work
   - 教育用言語（Scratch, Alice）
   - 設計制約言語（Elm, Rust）

3. Box Theory
   - 箱の定義と性質
   - 言語設計への適用
   
4. Nyash Language Design
   - Everything is Box
   - 明示的デリゲーション
   - スコープと生命管理

5. Educational Experiment
   - 実験設計
   - 結果と分析
   
6. Discussion
   - 教育的示唆
   - 産業界への影響
   
7. Conclusion
```

## 🚀 進捗状況

- [ ] 理論的枠組みの整理
- [ ] 実験プロトコルの設計
- [ ] IRB（倫理審査）申請
- [ ] パイロット実験
- [ ] 本実験
- [ ] 論文執筆

## 📚 参考文献候補

- Guzdial, M. (2015). Learner-Centered Design of Computing Education
- Stefik, A., & Siebert, S. (2013). An Empirical Investigation into Programming Language Syntax
- Ko, A. J., & Myers, B. A. (2005). A framework and methodology for studying the causes of software errors