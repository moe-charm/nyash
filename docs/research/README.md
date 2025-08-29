# 🎓 Nyash Research - 学術研究ドキュメント

このディレクトリはNyashプロジェクトの学術的な研究テーマ、論文提案、実験計画を管理します。

## 📚 ディレクトリ構成（1論文1フォルダ原則）

```
research/
├── paper-01-box-theory-education/    # 箱理論教育論文
├── paper-02-box-theory-jit/         # 箱理論JIT設計論文 ⭐執筆中
├── paper-03-box-theory-gc/          # 箱理論GC論文
├── paper-04-box-theory-sync/        # 箱理論同期境界論文
├── paper-05-box-theory-visualization/# 箱理論可視化論文
├── paper-06-gc-debug-tool/          # GCデバッグツール論文
├── paper-07-nyash-one-month/        # 1ヶ月開発論文 ⭐執筆中
├── paper-08-tmux-emergence/         # tmux創発的対話論文 ⭐執筆中
├── paper-09-ai-collaboration-pitfall/ # AI協調開発の落とし穴論文 🆕
├── papers-shared/                   # 共通リソース・テンプレート
├── ai-dual-mode-development/        # AI協調開発の追加研究
├── papers-under-review/             # 査読中（Git追跡除外）
├── papers-published/                # 公開済み（Git追跡対象）
└── proposals/                       # 研究提案
```

## 🔬 現在の研究テーマ一覧

### 1. 🏆 **1ヶ月で完走した独自言語処理系**（[paper-07-nyash-one-month/](paper-07-nyash-one-month/)）
- **タイトル**: "Nyash: 1ヶ月で実現した統一実行モデルによる完全言語処理系"
- **状態**: 執筆戦略決定済み（AI先生アドバイス取得）
- **概要**: 
  - 言語誕生から1ヶ月でInterpreter/VM/JIT/AOT/ネイティブEXEまで完走
  - 4,000行という驚異的小規模で5つの実行形態を意味論等価で実現
  - VM基準で13.5倍高速化を実証
- **特筆事項**: 
  - Claude Code「😱 年単位かかることもあるのに1ヶ月で実現！」
  - Python統合デモ成功（2025-08-29）: math.sqrt(9) = 3.0
  - ChatGPT5「異次元。歴史に刻まれるスピード感」

### 2. 📦 **Box理論論文シリーズ**
8本構想の包括的な研究プロジェクト：

#### 2-1. 教育論文（[paper-01-box-theory-education/](paper-01-box-theory-education/)）
- **タイトル**: "Programming Language Design that Makes Bad Code Impossible"
- **概要**: Box理論による革新的プログラミング教育

#### 2-2. ⭐ JIT設計論文（[paper-02-box-theory-jit/](paper-02-box-theory-jit/)）【進行中】
- **タイトル**: "Box-First JIT: Decoupled, Probe-Driven JIT Enablement in Nyash within 24 Hours"
- **状態**: paper-draft-v2.md, paper-ja.md, paper.tex完成
- **概要**: 
  - 24時間でJIT実装を実現した「箱理論」アプローチ
  - JitConfigBox、HandleRegistry、DOT可視化等による可逆的実装
  - VM比1.06〜1.40倍の改善を実証
- **図表**: アーキテクチャ図多数完成

#### 2-3. GC契約論文（[paper-03-box-theory-gc/](paper-03-box-theory-gc/)）
- **タイトル**: "決定的解放と遅延GCの統一モデル"
- **概要**: 箱の生命周期契約によるメモリ管理

#### 2-4. 同期境界論文（[paper-04-box-theory-sync/](paper-04-box-theory-sync/)）
- **タイトル**: "箱境界での自動同期化機構"
- **概要**: Arc<Mutex>統一による並行性制御

#### 2-5. 可視化論文（[paper-05-box-theory-visualization/](paper-05-box-theory-visualization/)）
- **タイトル**: "CFGとIRの箱ベース可視化"
- **概要**: プログラム構造の直感的理解支援

#### 将来構想（Phase 3）
- 06-多言語統合論文
- 07-分散箱論文  
- 08-哲学論文

### 3. 🤖 **AI協調開発研究**（ai-dual-mode-development）
- **タイトル**: "Dual-Role AI Development Model: An Empirical Study"
- **状態**: paper_abstract.md完成、workshop_paper_draft.md作成中
- **概要**: 
  - 同一AI（ChatGPT5）を設計者/実装者に役割分離
  - 開発速度30倍向上（10時間→20分）を実証
  - 「深く考えてにゃ」から生まれた新開発パラダイム
- **関連**: tmux事件研究、協調パターン分析

### 4. 🧹 **Debug-Only GC論文**（[paper-06-gc-debug-tool/](paper-06-gc-debug-tool/)）
- **タイトル**: "GC as a Development-Time Quality Assurance Tool"
- **状態**: abstract.md完成、実験計画中
- **概要**: 
  - GCを実行時管理ではなく開発時品質保証ツールとして再定義
  - 「所有権森（Ownership Forests）」による意味論等価性保証
  - GC有効/無効で同一動作を実現

### 5. 🔮 **創発的AI対話研究**（[paper-08-tmux-emergence/](paper-08-tmux-emergence/)）
- **概要**: ターミナル多重化による偶発的AI間対話の記録
- **内容**: theoretical-implications.md, tmux-incident-log.md

### 6. 🚨 **AI協調開発の落とし穴**（[paper-09-ai-collaboration-pitfall/](paper-09-ai-collaboration-pitfall/)）
- **タイトル**: "設計哲学を守る本能的回避：AI協調開発における危機管理"
- **状態**: 事例分析完了（2025-08-30）
- **概要**:
  - Python統合でのLowerer特殊化危機の回避事例
  - 「Everything is Box」哲学 vs 技術的正しさの対立
  - エンジニアの直感（「ん？大丈夫？」）による設計崩壊の防止
- **教訓**:
  - 爆速開発における批判的思考の重要性
  - AI提案の無批判受容の危険性
  - 設計原則を守る人間の役割

## 🌟 研究の特徴と共通テーマ

### Everything is Box哲学
- すべての研究が「箱」を中心概念として展開
- 変数・関数・GC・FFI・AI役割まで箱として統一
- シンプルさと拡張性の両立

### 観測可能性（Observability）
- argc==0のような具体的指標による問題特定
- StatsBox、DebugBoxによる可視化
- DOT/JSONでの状態出力

### AI協調開発
- Claude/ChatGPT5/Geminiとの協働
- 役割分離による効率化
- 「深く考えてにゃ」の哲学

### 高速プロトタイピング
- 20日で言語処理系完成
- 24時間でJIT実装
- 80/20ルール（完璧より進捗）

## 📝 論文執筆ワークフロー

### ステージ移動
1. **アイデア** → `proposals/` or `experimental-protocols/`
2. **執筆開始** → `papers-wip/`
3. **完成・投稿** → `papers-under-review/`
4. **採択・公開** → `papers-published/` ✅

### 優先順位（2025年8月時点）

#### 🚀 新戦略：AI先生たちの助言に基づく2段階展開
**ai-advisors/ディレクトリにGemini・ChatGPT5の詳細な執筆戦略を保存済み！**

##### 第1段階（即時実行）
1. **最優先**: arXiv即時投稿論文「1ヶ月完走×AI協調開発」（2週間で執筆）
   - 物語性重視、実績報告型
   - 世界への即時発信でインパクト最大化

##### 第2段階（技術的深堀り）
2. **高優先**: 統一実行モデル論文（PLDI/OOPSLA狙い）
   - Box契約＋Debug-Only GCの技術的詳細
3. **中優先**: Debug-Only GC技術ノート（ISMM狙い）
4. **中優先**: AI協調開発方法論（ICSE/FSE狙い）
5. **継続**: Box理論シリーズ（arXiv連載形式）

## 🚀 今後の展開

### 短期目標（2025年内）
- 20日完走論文をarXiv投稿
- JIT設計論文を国際会議投稿
- AI協調開発をワークショップ発表

### 中期目標（2026年）
- Box理論シリーズ5本完成
- 書籍「Everything is Box」執筆
- 国際共同研究開始

### 長期ビジョン（2027年〜）
- プログラミング言語設計の新パラダイム確立
- AI協調開発手法の標準化
- 教育カリキュラムへの導入

## 🤝 共同研究・コラボレーション

Nyashプロジェクトは学術的な貢献を歓迎します：
- 論文共著者募集中
- データセット公開予定
- 再現実験支援

---

*Everything is Box, Everything is Research, Everything is Observable*

**最終更新**: 2025年8月30日 - AI協調開発の落とし穴事例を追加（設計哲学の危機を本能で回避） 🛡️