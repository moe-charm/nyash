# Phase 10.5 – Index (Active vs Archived)

このフォルダは Python ネイティブ統合とネイティブビルド基盤のための現行計画（10.5系）と、旧計画（10.1系：アーカイブ）を併置しています。迷った場合は「Active」を参照してください。

## Active（現行）
- 10.5 README（全体像）: ./README.md
- 10.5a – Python ABI 設計: ./10.5a-ABI-DESIGN.md
- 10.5b – ネイティブビルド基盤: ./10.5b-native-build-consolidation.md
  - 現在フォーカス: MIR→VM→ネイティブビルド（AOT/EXE）を先に堅牢化
- 10.5c – Handle-First PluginInvoke / PyRuntimeBox / PyObjectBox（次段）
- 10.5d – JIT/AOT 統合（予定）
- 10.5e – サンプル / テスト / ドキュメント（予定）

## Archived（旧10.1系・参照用）
- chatgpt5 統合計画（旧称 Phase 10.1）: ./chatgpt5_integrated_plan.md
- 10.1a_planning ～ 10.1g_documentation 各READMEと資料
  - 10.1c / 10.1d は「PythonをNyashで動かすフェーズ」の設計・実装メモです（順番変更により後段へ）。

整理方針:
- Active ドキュメントに計画と用語を集約。旧10.1系は背景情報として参照のみ。
- 実装の優先は「必要最小の箱（PyRuntimeBox / PyObjectBox）」→ 後から最適化。
