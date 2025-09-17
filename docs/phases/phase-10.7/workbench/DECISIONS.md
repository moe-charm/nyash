# DECISIONS (Phase 10.7)

## 2025-08-30 — 二本立て運用（決定）
- 決定: 現行の実行系（PyRuntimeBox, Plugin-First）は維持し、トランスパイル系（Python→Nyash）は All-or-Nothing で併走。
- 代替案: トランスパイルの部分フォールバック（実行時にPyRuntimeへ落とす）。
- 理由: 実行時の不一致/隠れ分岐を避ける。デプロイ時の挙動を単純に保つ。
- 影響: 生成Nyashの品質責任はトランスパイラ側。利用者は明示的に系を選択。

## 2025-08-30 — Parser/CompilerもプラグインBox（決定）
- 決定: PythonParserBox/PythonCompilerBox としてプラグイン化し、CLIから呼び出す。
- 代替案: コア組込み。
- 理由: Plugin-First原則、配布容易性、差し替え性、隔離テスト。
- 影響: plugins/ 以下に新規プラグインを追加。SDKの最小拡張が必要になる場合あり。
