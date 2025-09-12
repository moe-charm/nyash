# AI-Assisted Compiler Development: The Nyash LLVM Journey

## 📚 論文概要
本論文は、ChatGPT/Claude/Geminiを活用してゼロからコンパイラ（Nyash）をLLVM層まで構築した、世界初？のAI主導コンパイラ開発の完全記録です。

## 🎯 論文の切り口（ハイブリッドアプローチ）

### 1. **技術的貢献**
- MIR14: 27→13→14命令への進化
- Box理論: Everything is Boxの設計哲学
- LoopForm: PHI集中化による制御フロー正規化
- Sealed SSA: 支配関係の構造的保証

### 2. **方法論的貢献**
- AI支援開発プロセスの実態記録
- 人間×AIの役割分担と共進化
- 失敗と解決の繰り返しから学ぶ知見
- Python llvmliteハーネスへの方針転換

## 📅 開発タイムライン（1週間以上の激闘）

```
Day 1: 「PHI簡単だにゃ！」→ 現実の厳しさを知る
Day 2-3: PHI欠落、支配関係違反との戦い
Day 4: ChatGPT 8分調査「Investigating the builder issue」
Day 5: Python llvmlite導入決断
Day 6: LoopForm最終手段投入
Day 7+: Resolver API統一、設計のブレを修正
```

## 🔍 主要な転換点

1. **PHI配線問題の発覚**
   - 「PHINode should have one entry for each predecessor」
   - emit側での配線 vs to側での需要駆動

2. **支配関係違反との長い戦い**
   - 「Instruction does not dominate all uses!」
   - 値解決の分散が根本原因と判明

3. **LoopForm導入**
   - 既存問題を顕在化させた構造
   - 「すべてをループのスコープにする」シンプルな理念

4. **Resolver API設計**
   - すべての値解決を統一的に扱う
   - 局所化→型正規化→キャッシュの自動化

## 📝 論文構成（案）

1. **序論**: AI時代のコンパイラ開発
2. **背景**: 既存コンパイラの複雑性とNyashの設計思想
3. **技術編**: MIR14, Box理論, LoopForm, Sealed SSA
4. **開発編**: AI対話プロセスと実装の実態
5. **評価編**: LLVM層完成の証明と性能評価
6. **考察編**: AI×人間の協調開発の可能性と限界
7. **結論**: 新しいコンパイラ開発手法の提案

## 🎉 成果物
- 動作するLLVMバックエンド
- 27→14命令の極限的IR削減
- AI支援開発の実践的知見
- 「簡単最高」哲学の実証

## 📌 注記
- 完全な再現は困難（「もう覚えてないにゃー」）
- しかし、プロセスの記録として価値がある
- 混沌とした開発過程自体が研究データ