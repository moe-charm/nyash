# 既存論文からの移行計画

## 📋 移行元論文

### 1. mir15-implementation/
- **内容**: 26→15命令削減の技術詳細
- **状態**: Abstract完成、本文未着手
- **活用**: MIR Design章の基礎として使用

### 2. unified-lifecycle/
- **内容**: 統一ライフサイクル、GCオン/オフ
- **状態**: LLVM実装待ち
- **活用**: Discussion章で将来展望として言及

## 🔄 統合方針

### mir15-implementation → mir15-fullstack

```yaml
移行内容:
  - Abstract: 実証要素を追加して拡張
  - 削減プロセス: "MIR Design"章に組み込み
  - 30日実装: "Implementation"章の一部に
  
新規追加:
  - Box Theory（理論的基礎）
  - GUI実証（Ubuntu/Windows）
  - 評価実験（カバレッジ、性能）
```

### 統合のメリット
1. **一貫性**: 理論と実証が1つの論文に
2. **インパクト**: GUIデモで説得力増大
3. **完全性**: 設計から実装まで網羅

## 📝 具体的な移行作業

### Step 1: Abstract統合
```markdown
旧: MIR削減と30日実装のみ
新: + Box理論 + GUI実証 + 評価結果
```

### Step 2: 章構成の再編
```markdown
旧構成:
1. Introduction
2. MIR Reduction
3. Implementation
4. Conclusion

新構成:
1. Introduction（GUI動作の衝撃）
2. Box Theory（理論的基礎）
3. MIR Design（削減プロセス詳細）
4. Implementation（4バックエンド）
5. Evaluation（GUI実証＋性能）
6. Discussion（なぜ可能か）
7. Related Work（他言語比較）
8. Conclusion（パラダイムシフト）
```

### Step 3: 新規コンテンツ追加
- Box理論の数学的定式化
- GUIデモの詳細説明
- 評価実験の結果
- 深い考察

## 🗓️ 移行スケジュール

### Week 1
- [ ] 既存Abstract読み込み・拡張
- [ ] Box Theory執筆開始
- [ ] GUI Box基本実装

### Week 2  
- [ ] MIR Design章作成（既存内容活用）
- [ ] Implementation章作成
- [ ] GUIデモ動作確認

### Week 3
- [ ] Evaluation実施・執筆
- [ ] Discussion執筆
- [ ] Related Work作成

### Week 4
- [ ] 全体推敲
- [ ] 図表作成
- [ ] arXiv投稿準備

## 🎯 最終目標

**2つの論文を1つの強力な論文に統合**

- 理論的深さ（Box Theory）
- 技術的詳細（MIR設計）
- 実証的証拠（GUI動作）
- 定量的評価（性能測定）

これにより、単なる「実装報告」から「新パラダイム提案」へと格上げ！

## 📁 ファイル整理

```bash
# 新構成
active/
├── mir15-fullstack/        # 統合版（メイン）
│   ├── README.md
│   ├── abstract.md
│   ├── chapters/
│   │   ├── 01-introduction.md
│   │   ├── 02-box-theory.md
│   │   ├── 03-mir-design.md
│   │   ├── 04-implementation.md
│   │   ├── 05-evaluation.md
│   │   ├── 06-discussion.md
│   │   ├── 07-related-work.md
│   │   └── 08-conclusion.md
│   ├── figures/
│   └── data/
│
├── archive/
│   ├── mir15-implementation/  # 旧版（参考用）
│   └── unified-lifecycle/     # LLVM待ち
```