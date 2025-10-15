# Papers Active - 執筆中の論文

**最終更新**: 2025年10月9日

---

## 📝 執筆中の論文（最優先アクセス）

現在執筆中の論文。完成したら `papers-archive/` に移動。

### 1. **ai-design-intent-communication/** 🆕 ⭐
**AI協働開発における設計意図の伝達**

- **内容**: Implementation State Bias の発見と分析
- **キーワード**: AI pair programming, Design intent, Incremental migration
- **完成度**: 100%（初稿完成）
- **発見**: AIは実装を見て設計意図を誤解する（段階的移行中）
- **事例**: Hakorune の "Everything is Plugin" 誤解
- **価値**: 新しい概念提示 + 実践的解決策 + アーキテクチャ検証
- **作成日**: 2025年10月9日

**読み始める**: [ai-design-intent-communication/paper.md](ai-design-intent-communication/paper.md)

---

### 2. **hakorune-complete-story/** ⭐
**58日間の完全記録（統合版）**

- **内容**: タイムライン、面白事件簿、AI協働洞察、技術詳細を統合
- **構成**: 全6章（README + Part 1-5 + Appendix）
- **総行数**: 約13,800行
- **目標**: OOPSLA 2026投稿（AI協働開発記録として）
- **特徴**:
  - Part 1: タイムライン（58日間の軌跡）
  - Part 2: 面白事件簿（爆速開発の裏側）
  - Part 3: ChatGPT却下と雪辱（3,990行の大作へのリンク）
  - Part 4: 危険センサー（破綻しなかった理由）
  - Part 5: 技術詳細（Everything is Box実装）
  - Appendix: 統計と資料

**読み始める**: [hakorune-complete-story/README.md](hakorune-complete-story/README.md)

---

### 3. **mir14-universal-execution/**
**MIR14設計哲学**

- **内容**: たった14命令で万能実行系を実現する中間表現
- **キーワード**: Everything is Box哲学の証明
- **完成度**: 70%

---

### 4. **box-first-convergent-design/**
**57日間のAI協働開発革命**

- **内容**: Convergent Design Pattern（収束型設計パターン）の実証
- **キーワード**: AIが却下した設計に57日後に収束した軌跡
- **行数**: 777行
- **完成度**: 80%

---

### 5. **nyash-box-first-language/**
**Nyash言語設計・認知負荷分析**

- **内容**:
  - Everything is Box完全実装
  - 認知負荷論文（15時間労働の暗黒面分析）
  - 設計洞察4本（階層設計進化、toplevel main判断、メソッド解決、4原則）
- **完成度**: 75%

---

### 6. **box-oriented-programming/**
**Box指向プログラミング**

- **内容**: Everything is Box の理論的基盤
- **完成度**: 60%

---

## 📊 執筆進捗状況

```yaml
ai-design-intent-communication:
  進捗: 100%（初稿完成！）
  状態: 完成（2025年10月9日）
  行数: 1,100行
  特記: Implementation State Bias の概念提示

hakorune-complete-story:
  進捗: 100%（完成！）
  状態: 完成（2025年9月30日）
  行数: 13,800行
  章数: 6章

mir14-universal-execution:
  進捗: 70%
  状態: 執筆中

box-first-convergent-design:
  進捗: 80%
  状態: 執筆中

nyash-box-first-language:
  進捗: 75%
  状態: 執筆中

box-oriented-programming:
  進捗: 60%
  状態: 執筆中
```

---

## 🎯 投稿予定

### OOPSLA 2026

**投稿予定日**: 2026年2月

**メイン論文**:
- Everything is Box: A Unified Programming Language Design
- 基礎: hakorune-complete-story + nyash-box-first-language

**サブ論文**:
- AI-Collaborative Development: A 58-Day Case Study
- 基礎: box-first-convergent-design

**補足論文**:
- MIR14: Minimal Instruction Set for Universal Execution
- 基礎: mir14-universal-execution

---

## 📚 関連資料

### papers-archive/（完成済み論文 - 41本）

完成した論文群。詳細は [`../papers-archive/README.md`](../papers-archive/README.md) を参照。

主要論文:
- paper-02-box-theory-jit: Box理論とJIT（23ファイル）
- paper-07-nyash-one-month: 1ヶ月開発記録（14ファイル）
- paper-14-ai-collaborative-abstraction: AI協働開発（11ファイル）
  - **chatgpt-rejection-and-redemption.md（3,990行）** - Part 3でリンク

### research/（進行中の調査）

- `ai-dual-mode-development/`: AI協働開発研究
- `timeline/`: 開発タイムライン記録

---

## 📝 執筆ガイドライン

### 新規論文の作成

1. `papers-active/` にフォルダ作成
2. `README.md` を作成（概要・目的・構成）
3. このファイルに追加
4. 執筆開始

### 完成時の処理

1. 論文の最終チェック
2. `papers-archive/` に移動
3. このファイルから削除
4. `papers-archive/README.md` に追加

---

## 🎊 最近の成果

### 2025年10月9日
- ✅ **ai-design-intent-communication 完成！**
  - 初稿完成（1,100行）
  - Implementation State Bias の新概念提示
  - AI協働開発の落とし穴と解決策
  - "Everything is Plugin" アーキテクチャの検証

### 2025年9月30日
- ✅ **hakorune-complete-story 完成！**
  - 全6章、13,800行
  - Claude Sonnet 4.5 の30時間連続作業能力を実証
  - AI協働開発の新記録

### 2025年9月28日
- ✅ box-first-convergent-design 80%完成
- ✅ Everything is Box 100%達成の記録

### 2025年9月15日
- ✅ nyash-box-first-language 75%完成
- ✅ 認知負荷分析完了

---

## 🔗 ナビゲーション

```
docs/private/
├── papers-active/           ← 今ここ
│   ├── ai-design-intent-communication/  🆕 完成！
│   ├── hakorune-complete-story/  ⭐ 完成！
│   ├── mir14-universal-execution/
│   ├── box-first-convergent-design/
│   ├── nyash-box-first-language/
│   └── box-oriented-programming/
├── papers-archive/          完成論文（41本）
├── research/                進行中の調査
└── ideas/                   アイデアメモ
```

---

**さあ、どの論文を読む？** 📖✨

**最新**: [ai-design-intent-communication](ai-design-intent-communication/paper.md) - Implementation State Bias の発見！
**人気**: [hakorune-complete-story](hakorune-complete-story/README.md) - 58日間の完全記録！