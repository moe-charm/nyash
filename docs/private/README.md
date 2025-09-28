# Nyash Private Research & Papers

**最終整理**: 2025-09-28 - フラットで直感的な構造に再編成完了 🎉

---

## 📁 ディレクトリ構造

### 📝 papers-active/ - 執筆中の論文（最優先アクセス）

現在執筆中の論文。完成したら `papers-archive/` に移動。

- **`mir14-universal-execution/`** - MIR14設計哲学
  - たった14命令で万能実行系を実現する中間表現
  - Everything is Box哲学の証明

- **`box-first-convergent-design/`** - 57日間のAI協働開発革命
  - Convergent Design Pattern（収束型設計パターン）の実証
  - AIが却下した設計に57日後に収束した軌跡（777行）

- **`nyash-box-first-language/`** - Nyash言語設計・認知負荷分析
  - Everything is Box完全実装
  - 認知負荷論文（15時間労働の暗黒面分析）
  - 設計洞察4本（階層設計進化、toplevel main判断、メソッド解決、4原則）

### 📦 papers-archive/ - 完成・保管済み論文（41フォルダ、117+ MDファイル）

アーカイブ済みの研究論文。貴重な開発記録。

- `paper-01-box-theory-education/` - Box理論教育
- `paper-02-box-theory-jit/` - Box理論とJIT
- `paper-07-nyash-one-month/` - Nyash 1ヶ月開発記録
- `paper-08-tmux-emergence/` - tmux創発対話
- `paper-k-explosive-incidents/` - 爆速開発事件簿
- ... 他36本

### 🔬 research/ - 進行中の調査・研究ノート

- **`ai-dual-mode-development/`** - AI協働開発研究
  - デュアルモードAI開発パターン
  - PHI bug consultation記録
  - 危険センサー事例研究

- **`paper-planning/`** - 論文戦略ドキュメント
  - 論文分割戦略、優先順位、ロードマップ

- **`timeline/`** - 開発タイムライン記録

### 💡 ideas/ - アイデアメモ（80/20ルールの残り20%）

実装アイデア・改善案・新機能提案。

- `improvements/` - 既存機能の改善案
- `new-features/` - 新機能提案
- `other/` - その他アイデア・調査メモ

### 📋 proposals/ - 設計提案書

言語機能・アーキテクチャの正式提案。

### 📄 _templates/ - テンプレート

論文・ドキュメントの雛形。

### 📤 out/ - 出力ファイル（1.6MB）

PDF生成物・ビルド成果物。`pandoc/` ビルド設定含む。

### 📝 paper-ideas.md - 論文アイデアバックログ

将来書きたい論文のアイデア一覧。

---

## 🗺️ 論文間の関係図

```
Box-First Architecture (設計哲学)
       ↓
MIR14 (中間表現実装)
       ↓
Convergent Design (開発プロセス・AI協働)
       ↓
Cognitive Load (持続可能性分析)
```

---

## 🎯 運用ルール

1. **執筆中の論文**: `papers-active/` に配置
2. **完成した論文**: `papers-archive/` に移動
3. **アイデア段階**: `ideas/` または `paper-ideas.md` に記録
4. **設計提案**: `proposals/` に配置
5. **研究ノート**: `research/` に配置

---

## 📅 整理履歴

- **2025-09-28**: 大整理実施
  - 重複階層 `research/docs/private/research/` を削除
  - `papers-archive/` を救出（117+ MDファイル）
  - `papers-active/` を最上位に移動
  - Box-First大論文（777行）を `papers-active/` に統合
  - 古い `papers/` ディレクトリを解体・整理
  - フラットで直感的な構造に再編成

---

## 🐱 にゃーん

この構造なら迷子にならないにゃ！
