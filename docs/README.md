# 📚 Nyash Documentation

## 🚀 はじめに
- **現在のタスク**: [development/current/CURRENT_TASK.md](development/current/CURRENT_TASK.md)
- **コア概念の速習**: [reference/architecture/nyash_core_concepts.md](reference/architecture/nyash_core_concepts.md)

---

## 📂 新しいドキュメント構造（2025年8月20日再編成）

### 📖 [reference/](reference/) - 正式な技術仕様
- **language/** - 言語仕様（構文、型システム、Box仕様）
- **architecture/** - システムアーキテクチャ（MIR、VM、実行バックエンド）
- **api/** - ビルトインBoxのAPI仕様
- **plugin-system/** - プラグインシステム、BID-FFI仕様
  - 🆕 **[TypeBox ABI統合](../development/roadmap/phases/phase-12/)** - C ABI + Nyash ABI統一設計
  - まずはこちら: `reference/boxes-system/plugin_lifecycle.md`（PluginBoxV2のライフサイクル、singleton、nyash.tomlの要点）

### 📚 [guides/](guides/) - 利用者向けガイド
- **getting-started.md** - はじめに（統一版）
- **tutorials/** - ステップバイステップのチュートリアル
- **examples/** - 実践的なサンプルコード
- **wasm-guide/** - WebAssemblyビルドガイド

### 🔧 [development/](development/) - 開発者向け
- **current/** - 現在進行中のタスク（CURRENT_TASK.md等）
- **roadmap/** - 開発計画
  - phases/ - Phase 8～12の詳細計画
  - phase-12/ - 🆕 TypeBox統合ABI設計（革命的プラグイン統一）
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
- [Getting Started](guides/getting-started.md)
- [Language Guide](guides/language-guide.md)
- [P2P Guide](guides/p2p-guide.md)

### 技術リファレンス
- [言語リファレンス](reference/language/LANGUAGE_REFERENCE_2025.md)
- [アーキテクチャ概要](reference/architecture/TECHNICAL_ARCHITECTURE_2025.md)
- [実行バックエンド](reference/architecture/execution-backends.md)
- [プラグインシステム](reference/plugin-system/)
 - [CLIオプション早見表](tools/cli-options.md)

### 開発状況
- [現在のタスク](development/current/CURRENT_TASK.md)
- [開発ロードマップ](development/roadmap/)
- [Phase別計画](development/roadmap/phases/)
  - 🔥 **[Phase 12: TypeBox統合ABI](development/roadmap/phases/phase-12/)** - プラグイン革命！

---

## 📋 再編成について
ドキュメントは2025年8月20日に再編成されました。詳細は[REORGANIZATION_REPORT.md](REORGANIZATION_REPORT.md)を参照してください。

旧パスから新パスへの主な変更：
- `説明書/` → `guides/` と `reference/` に分割
- `予定/` → `development/roadmap/`
- 散在していたファイル → 適切なカテゴリに整理

---

Nyash は「Everything is Box」哲学に基づく言語です。詳細はコア概念とガイドを参照してください。
