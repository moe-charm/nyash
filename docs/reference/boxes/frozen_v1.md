# Frozen v1 Box Set (Phase 15.76)

Purpose
- 定義: 凍結（Frozen）ラインに同梱する最小のコア Box 群。
- 方針: 初期は静的リンク（単一バイナリ）を優先。拡張は動的プラグインで段階導入。

Included (static, v1)
- String, Array, Map
- Console (print)
- Time (now_ms)
- JSON (stringify/min)
- File[min]（必要最小限の読み書き）

Excluded (defer to dynamic add‑ons)
- Regex（重い依存・差分が大きい）
- Crypto（アルゴリズム・依存の選定が必要）
- OS/Path の拡張（環境差が大きい）
- Network（配布・セキュリティポリシーを要検討）

Packaging rationale
- Single EXE による再現性と配布容易性を最優先。
- HostHandle/Plugin 経由の拡張は VM/LLVM と共通導線で後付け可能。

Controls (features)
- `crates/hako_kernel` の features で構成（例: `core-collections`, `core-io`）。
- フル静的→コア静的＋拡張動的への段階解凍を想定（Release ノートで告知）。

Notes
- 将来: `C ABI 出力` などのビルド補助機能は “プラグイン”相当の拡張として後置き（既定は同梱しない）。
- 既存の VM プロファイルはそのまま使用可能。凍結ラインは“配布/日常開発の既定”として整備する。
