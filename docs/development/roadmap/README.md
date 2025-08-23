# 🗺️ Nyash 予定（ロードマップ・タスク）

## 📋 現在進行中

### 🎯 最重要タスク
- **現在のタスク**: [development/current/CURRENT_TASK.md](../current/CURRENT_TASK.md)
- **Phase 8.3**: Box操作WASM実装（Copilot担当）
- **Phase 8.4**: ネイティブコンパイル実装計画（AI大会議策定済み）

## 🚀 ネイティブコンパイル計画 (2025-08-14策定)

### 📊 AI大会議成果
- **[🤖 AI大会議記録](ai_conference_native_compilation_20250814.md)** - Gemini×codex×Claude技術検討
- **[🗺️ ネイティブコンパイルロードマップ](native-compilation-roadmap.md)** - 技術戦略詳細

### ⚡ 実装フェーズ
- **Phase A (2-3週間)**: AOT WASM → 500倍高速化目標
- **Phase B (2-3ヶ月)**: Cranelift Direct → 600倍高速化目標  
- **Phase C (6ヶ月+)**: LLVM Ultimate → 1000倍高速化目標

## 🤖 Copilot協調管理

### 📋 Copilot作業管理
- **[copilot_issues.txt](copilot_issues.txt)** - Copilot様への依頼・課題整理
- **協調戦略**: [docs/CURRENT_TASK.md](../CURRENT_TASK.md)内に詳細記載

### 🎯 フェーズ別課題
- **Phase 8課題**: [native-plan/issues/](native-plan/issues/)
- **統合管理**: Claude×Copilot マージ競合回避戦略

## 📊 実装状況追跡

### ✅ 完了済み (Phase 8.2)
- WASM: 0.17ms (280倍高速化) 
- VM: 16.97ms (2.9倍高速化)
- ベンチマークシステム完成
- 3バックエンド統合CLI

### 🚧 進行中 (Phase 8.3)
- Box操作WASM対応（Copilot実装中）
- RefNew/RefGet/RefSet命令
- メモリレイアウト最適化

### 🔜 次期予定 (Phase 8.4+)
- AOT WASMネイティブ化
- MIR最適化基盤
- エスケープ解析実装
- MIR/Builder/Optimizer簡略化計画（責務分離・効果正規化・可視化）
  - [Phase 8.x: MIRパイプライン簡略化計画](phases/phase-8/phase_8_x_mir_pipeline_simplification.md)

## 📚 関連ドキュメント

### 📖 技術資料
- **[実行バックエンドガイド](../../reference/architecture/execution-backends.md)** - 3バックエンド使い分け
- **[コアコンセプト](../nyash_core_concepts.md)** - Everything is Box哲学

### 🔄 進捗管理
- **定期更新**: 毎週金曜日に進捗反映
- **AI会議**: 重要決定事項は3AI大会議で検討
- **ベンチマーク**: 性能回帰チェック自動実行

---

**最終更新**: 2025-08-14 - AI大会議によるネイティブコンパイル戦略確定
