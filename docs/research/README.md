# 🎓 Nyash Research - 学術研究ドキュメント

このディレクトリはNyashプロジェクトの学術的な研究テーマ、論文提案、実験計画を管理します。

## 📚 ディレクトリ構成

```
research/
├── papers-wip/         # 作業中の論文（Git追跡除外）
├── papers-under-review/ # 査読中の論文（Git追跡除外）
├── papers-published/    # 公開済み論文（Git追跡対象）
├── drafts/             # 下書き・メモ（Git追跡除外）
├── notes/              # 研究ノート（Git追跡除外）
├── proposals/          # 研究提案
└── experiments/        # 実験データ・計画
```

### 📁 フォルダの使い分け

#### 🚧 papers-wip/ (Work In Progress)
**Git追跡除外** - 執筆中の論文
- 自由に編集・実験できる作業場所
- AI（ChatGPT/Claude/Gemini）との共同執筆
- 未完成でも安心して保存

#### 📝 papers-under-review/
**Git追跡除外** - 投稿・査読中の論文
- 学会投稿済みだが未公開の原稿
- 査読コメントと対応メモ
- リビジョン作業中の文書

#### ✅ papers-published/
**Git追跡対象** - 完成・公開可能な論文
- arXiv投稿済み
- 学会発表済み
- 一般公開OKの完成版

## 🔬 現在の研究テーマ

### 作業中（papers-wip/）

#### 1. 箱理論論文シリーズ
- **01-教育論文**: "Programming Language Design that Makes Bad Code Impossible"
- **02-JIT論文**: "Box-Oriented JIT: A Fault-Tolerant Architecture" ⭐進行中
- **03-全体理論**: "Everything is Box: A Unified Model"

#### 2. AI協調開発論文
- **tmux事件研究**: "Emergent AI Dialogue through Terminal Multiplexing"
- **協調パターン**: "Multi-AI Collaboration Patterns in Software Development"

#### 3. Debug-Only GC: GCをデバッグツールとして再定義
- **概要**: GCを実行時メモリ管理ではなく開発時品質保証ツールとして使用
- **キーワード**: GC切り替え、所有権森、意味論的等価性

## 📝 論文執筆ガイドライン

### 構成テンプレート
各論文プロジェクトは以下の構成を推奨：
- `README.md` - 論文概要と進捗
- `abstract.md` - アブストラクト（日英両方）
- `introduction.md` - はじめに
- `design.md` - 設計・アーキテクチャ
- `experiments.md` - 実験計画と結果
- `evaluation.md` - 評価
- `related-work.md` - 関連研究
- `references.md` - 参考文献

## 🔄 論文執筆ワークフロー

### ステージ移動
1. **アイデア** → `drafts/` or `notes/`
2. **執筆開始** → `papers-wip/`
3. **完成・投稿** → `papers-under-review/`
4. **採択・公開** → `papers-published/` ✅

### Git管理の境界
```bash
# 作業中はGitに上げない
papers-wip/my-paper.md         # ❌ Git追跡されない
papers-under-review/my-paper.md # ❌ Git追跡されない

# 公開後はGitで管理
papers-published/my-paper.md    # ✅ Git追跡される
```

## 🚀 研究の進め方

1. **アイデア段階**: `drafts/`に初期アイデアを記録
2. **提案段階**: `research/proposals/`に研究提案を作成
3. **実験段階**: `research/experiments/`に実験計画・データ
4. **論文執筆**: `papers-wip/`で執筆作業
5. **査読対応**: `papers-under-review/`で管理
6. **公開**: `papers-published/`に移動してGit管理

## 🤝 共同研究

Nyashプロジェクトは学術的な貢献を歓迎します。研究提案やコラボレーションについてはプロジェクトチームまでご連絡ください。

---

*Everything is Box, Everything is Research*