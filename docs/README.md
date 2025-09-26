# 📚 Nyash Documentation

## 🚀 はじめに（導線）
- 現在のタスクと進行状況: ../CURRENT_TASK.md
- コア概念の速習: reference/architecture/nyash_core_concepts.md
- 設計ブループリント（文字列/文字コード）: development/design/blueprints/strings-utf8-byte.md

---

## 📂 ドキュメント構造（指針）

### 📖 [reference/](reference/) - 正式な技術仕様
- **language/** - 言語仕様（構文、型システム、Box仕様）
- **architecture/** - システムアーキテクチャ（MIR、VM、実行バックエンド）
- **api/** - ビルトインBoxのAPI仕様
- **plugin-system/** - プラグインシステム、BID-FFI仕様
  - 🆕🔥 **[TypeBox ABI統合 + セルフホスティング](../development/roadmap/phases/phase-12/)** - ABIすらBoxとして扱う革命的設計！
  - まずはこちら: `reference/boxes-system/plugin_lifecycle.md`（PluginBoxV2のライフサイクル、singleton、nyash.tomlの要点）

### 📚 [guides/](guides/) - 利用者向けガイド
- **getting-started.md** - はじめに（統一版）
- **tutorials/** - ステップバイステップのチュートリアル
- **examples/** - 実践的なサンプルコード
- **wasm-guide/** - WebAssemblyビルドガイド

### 🧩 [how-to/](how-to/) - 目的別ハウツー
- 手順重視の短いガイド（前提→コマンド→検証）

### 🔧 [development/](development/) - 開発者向け
- **current/** - 現在進行中のタスク（CURRENT_TASK.md等）
- **roadmap/** - 開発計画
  - phases/ - Phase 8～12の詳細計画
  - phase-12/ - 🆕🔥 TypeBox統合ABI + Nyash ABI C実装（セルフホスティング実現！）
  - native-plan/ - ネイティブビルド計画
- **proposals/** - RFC、新機能提案

### 🔌 Net Plugin（HTTP/TCP）
- 使い方と仕様: `reference/plugin-system/net-plugin.md`

### 🗄️ [archive/](archive/) - アーカイブ
- **consultations/** - AI相談記録（gemini/chatgpt/codex）
- **decisions/** - 過去の設計決定
- **build-logs/** - ビルドログ、ベンチマーク結果
- **old-versions/** - 古いドキュメント

---

## 🎯 クイックアクセス

### すぐ始める
- guides/getting-started.md
- guides/language-guide.md
- guides/p2p-guide.md

### 技術リファレンス
- reference/language/LANGUAGE_REFERENCE_2025.md
- reference/language/EBNF.md（演算子: ! 採用 / do-while 非採用）
- reference/language/strings.md（UTF‑8/Byte 二本柱）
- reference/architecture/TECHNICAL_ARCHITECTURE_2025.md
- reference/architecture/execution-backends.md
- reference/runtime/gc.md
- reference/plugin-system/
- tools/cli-options.md（CLI早見表）
 
### デザイン/ガイド
- guides/language-core-and-sugar.md（コア最小＋糖衣）
- guides/loopform.md（ループ正規化）
- guides/scopebox.md（開発時の可視化）
- guides/dev-local-alias.md（開発向け: 行頭 @name = expr → local 宣言糖衣）
 - guides/box-patterns.md（Boxパターン集：Ownership/Lease/Cancel/Capability/Affinity/Observable）
 - guides/box-design-checklist.md（Box 設計チェックリスト）
 - proposals/concurrency/boxes.md（並行モデルのBox設計：Routine/Channel/Select/Scope）
 - reference/concurrency/semantics.md（ブロッキング/close/select/観測の規約）
- design/（設計ノート入口）
  - development/design/legacy/flow-blocks.md（矢印フロー／匿名ブロック・設計草案）
  - development/proposals/scope-reuse.md（スコープ再利用ブロック・MVP提案）
  - reference/language/match-guards.md（ガード連鎖／Range・CharClass設計）
  - guides/core-principles.md（最小構文・ゼロランタイム・可視化の原則）

### 開発状況
- [現在のタスク](../CURRENT_TASK.md)
 - [開発ロードマップ](development/roadmap/)
 - [Phase別計画](development/roadmap/phases/)
   - 🔥 **[Phase 12: TypeBox統合ABI](development/roadmap/phases/phase-12/)**
   - 🔥 **[Phase 16: マクロ革命](development/roadmap/phases/phase-16-macro-revolution/)**
   - 🧪 **[Phase 17: LoopForm Self‑Hosting](development/roadmap/phases/phase-17-loopform-selfhost/)**
   - 💡 **[Rust所有権統合（候補）](private/ideas/new-features/2025-09-22-rust-ownership-fusion.md)** - Phase 17+候補
- 🧩 **[Mini‑VM 構築ロードマップ](development/roadmap/phases/phase-17-loopform-selfhost/MINI_VM_ROADMAP.md)**
- 🧭 **Using→Loader 統合（最小設計）**: development/design/legacy/using-loader-integration.md
 - 🗂️ **Docsの書き方（小さく・リンク駆動）**: guides/contributing-docs.md

---

## 📋 再編成について / フォルダの見分け方
ドキュメントは2025年8月20日に再編成されました。詳細は[REORGANIZATION_REPORT.md](REORGANIZATION_REPORT.md)を参照してください。

旧パスから新パスへの主な変更：
- `説明書/` → `guides/` と `reference/` に分割
- `予定/` → `development/roadmap/`
- 散在していたファイル → 適切なカテゴリに整理

---

補足:
- `reference/` は正本（仕様）。
- `guides/` は読み物、`how-to/` は手順書。
- `design/` は公開できる設計ノート。
- `private/` は下書き保管庫（将来 `reference/`/`design/` に昇格）。

Nyash は「Everything is Box」哲学に基づく言語です。詳細はコア概念とガイドを参照してください。
