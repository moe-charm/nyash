# Tooling: MIR snapshot expansion & printer stabilization

Summary:
- MirPrinter の出力順/表記を安定させ、スナップショットテストを拡張。

Scope/Tasks:
- [ ] MirPrinter の順序・表記の固定化（deterministic）
- [ ] tests/mir_snapshots/ を追加整備（loop/try/arith/compare/concat など）
- [ ] スナップショット差分で後退検知できるよう整備

Acceptance Criteria:
- 代表ケースのスナップショットが緑で安定

