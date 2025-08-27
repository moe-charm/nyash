# Phase 10.1 - PythonParserBox実装計画

このフォルダには、NyashとPythonの相互運用を実現するPythonParserBoxの実装計画が含まれています。

## 📁 フォルダ構造

### メイン実装ドキュメント（最新版）
- **`pythonparser_integrated_plan_summary.txt`** - 統合実装計画サマリー（最重要）
- **`python_implementation_roadmap.txt`** - 実装ロードマップ（エキスパート統合版）
- **`python_parser_box_implementation_plan.txt`** - 技術的実装計画（統合版）
- **`builtin_box_implementation_flow.txt`** - ビルトインBox実装フロー（統合版）
- **`python_to_nyash_transpiler.txt`** - Python→Nyashトランスパイラー機能

### エキスパート評価
- **`expert_feedback_gemini_codex.txt`** - Gemini先生とCodex先生のフィードバック全文

### アーカイブ（参考資料）
- **`archive/`** フォルダ
  - `python_parser_box_design.txt` - 初期設計案
  - `chatgpt5_original_idea.txt` - ChatGPT5の元アイデア
  - `summary_2025_08_27.txt` - 当日の議論まとめ

## 🎯 Phase 10.1の目標

1. **Pythonエコシステムの即座活用** - 既存Pythonライブラリの利用
2. **Differential Testing** - Nyashパーサーのバグ自動検証
3. **言語成熟度の向上** - 実用的なPythonコードでのストレステスト

## 🔑 核心戦略（エキスパート推奨）

1. **関数単位フォールバック** - ファイル全体でなく関数レベルで切り替え
2. **Python 3.11固定** - AST安定性の確保
3. **意味論の正確な実装** - 最適化より互換性優先
4. **GIL管理の最小化** - Python側でJSON生成、Rust側で処理
5. **テレメトリー重視** - 継続的な改善のための計測

## 📋 実装フェーズ

- **Phase 1**: Core Subset（基本構文）- 2週間
- **Phase 2**: Data Model（特殊メソッド）- 3週間
- **Phase 3**: Advanced Features（高度な機能）- 1ヶ月
- **Phase 4**: Modern Python（最新機能）- 将来

## 🚀 期待される成果

- 純Pythonループの2-10倍高速化
- Nyashパーサーのバグ発見と改善
- PythonからNyashへの段階的移行パス
- Python→Nyashスクリプト変換機能

---
作成日: 2025-08-27