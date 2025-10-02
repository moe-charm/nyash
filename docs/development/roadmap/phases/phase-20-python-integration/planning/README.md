# Planning - Python-Hakorune統合計画

## 📋 概要

Python-Hakorune統合の計画・設計ドキュメント集です。

## 📁 ファイル一覧

### 🌟 最新計画（2025-10-02追加）
- **[milestones.md](milestones.md)** ⭐必読 - M0〜M6段階的実装計画（ChatGPT Pro UltraThink）

### 主要計画書
- **[integrated-plan.md](integrated-plan.md)** - ChatGPT5による統合計画（旧Phase 10.5全体計画）
- **[python-parser-plan-summary.md](python-parser-plan-summary.md)** - Pythonパーサー統合計画サマリー

### 設計ドキュメント
- **[python-parser-box-design.md](python-parser-box-design.md)** - PythonパーサーBox設計
- **[expert-feedback.md](expert-feedback.md)** - GeminiとCodexによるAI専門家フィードバック

## 🎯 計画の核心

### Phase 10.5の目的（旧計画）

1. **ネイティブ基盤固め**
   - VM/JIT分離（VM=実行、JIT=コンパイル）
   - AOT/EXEパイプライン確立
   - クロスプラットフォーム対応

2. **Python統合**
   - PyRuntimeBox: Python実行環境
   - PyObjectBox: Pythonオブジェクト管理
   - Hakorune ⇄ Python 双方向呼び出し

### 設計方針

#### Embedding vs Extending
- **Embedding**: HakoruneプロセスにCPythonを埋め込み
- **Extending**: Python拡張モジュールとして提供

#### ABI設計
- ハンドル: TLV tag=8（type_id+instance_id）
- Pythonオブジェクト: `PyObjectBox` として格納
- 変換: Bool/I64/String/Bytes/Handle の相互変換
- GIL: birth/invoke/decRef中はGIL確保

## 📊 実装フェーズ（旧10.5計画）

| フェーズ | 期間 | 内容 |
|---------|------|------|
| 10.5a | 1-2日 | Python設計・ABI整合 |
| 10.5b | 2-4日 | ネイティブビルド基盤 |
| 10.5c | 3-5日 | PyRuntimeBox/PyObjectBox実装 |
| 10.5d | 3-5日 | JIT/AOT統合 |
| 10.5e | 1週間 | サンプル/テスト/ドキュメント |

## ⚠️ 現在のステータス

**保留中** - Phase 15（Hakoruneセルフホスティング）完了後に再開予定

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Parser Integration](../parser-integration/)
- [Core Implementation](../core-implementation/)
- [Design Documents](../design/)
