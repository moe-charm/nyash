# Seam‑Aware JSON Unification: AI 前処理 × C‑ABI Box 正規化（開発メモ／論文草案）

目的
- 生成・結合・実行系で発生する JSON の“揺れ”を、AI 前処理で吸収しつつ、最終は C‑ABI な JSON Box で決定論的に正規化する設計をまとめる。Nyash の using 結合（seam）問題や自前スキャナの脆さを根本解決して、開発生産性と信頼性を両立する。

背景（問題設定）
- JSON IR は人手編集を前提としない一方、生成器や結合工程（using inline）由来の“揺れ（欠落/重複/順序/境界ミス）”で壊れやすい。
- 既存は Nyash スクリプトによる最小スキャナ（MiniJson/MiniVmScan）で運用。高速に試作できるが、seam 断面やブレースずれに脆弱。
- PyVM は意味論参照器として有用だが、結合や健全性監視の導線が弱いとデバッグコストが増す。

着想（本提案）
- AI‑in‑the‑loop 正規化: 生成 JSON の揺れを AI で前整形（危険な崩れを JSON 仕様準拠に寄せる）。
- Seam‑aware 結合: using 結合時に断面ログ（prelude_tail/body_head）・brace 差分・重複（dup_box/dup_fn）を観測・補修。
- 決定論的 Box: C‑ABI で呼べる JSON エンジン（候補: yyjson）を薄い Box 化し、最終抽出・参照はそこで確定。

想定構成（段階導入）
1) Python combiner（現行）
   - tools/using_combine.py で seam ログ・brace 補完・最小 dedup を実施（NYASH_USING_COMBINER=1）。
   - みんなが読むソースは“結合後1ファイル（[pyvm-code]）”に統一。
2) C‑ABI JSON Box
   - ライブラリ: yyjson（C, MIT）。parse/close + JSON Pointer get_* + statements イテレータ + esc_json。
   - Nyash 側から externcall 経由で呼び出し（Box 抽象で隠蔽）。
3) 自前スキャナの段階置換
   - MiniJson/MiniVmScan 呼び出し箇所を C‑ABI Box に差し替え。seam のブレでも壊れない土台へ。

評価軸（実験計画）
- 正常化率: 代表“揺れ”ケース（欠落/重複/境界）で AI 前処理あり/なしを比較。
- 信頼性: JSON Box 抽出の精度（ゴールデン比較）・クラッシュ率・未定義挙動の減少。
- 生産性: デバッグ時間/回帰検出時間/レビュー時間（seam インスペクタの数値 prelude_brace_delta, dup_box, dup_fn）。
- 性能: 追加前処理のオーバーヘッド、C‑ABI 化による抽出速度（ms/MB）。

関連実装（現状資産）
- Seam ログ/補修: src/runner/modes/common_util/resolve.rs, tools/using_combine.py（--seam-debug / --fix-braces / --dedup-*）。
- インスペクタ: apps/selfhost-vm/boxes/seam_inspector.nyash（prelude_brace_delta/dup_box/dup_fn を出力）。
- PyVM 参照器: src/runner/modes/pyvm.rs, tools/pyvm_runner.py（[pyvm-code] ダンプと MIR 実行）。

研究貢献（書き出し案）
- AI 前処理＋決定論的 Box のハイブリッドで、JSON IR の“運用上の揺れ”に強い開発フローを提案・実装・定量化。
- Seam‑aware（断面意識）での結合監視手法（ログ/メトリクス）と、即応の補修（brace/dedup）。
- Nyash の Box 哲学に沿った C‑ABI JSON Box による抽象境界の設計と運用事例。

想定リスクと対策
- AI 前処理の再現性: 前処理は“提案/補助”に限定、最終整形は Box で決定論に。
- エンジン依存: yyjson を既定、巨大入力向けには YAJL を第二候補（将来のストリーミング向け）。

短期ロードマップ（半日粒度）
- [ ] C‑ABI JSON ラッパ（parse/get_str/get_int/esc_json）モック追加
- [ ] MiniVmScan の get_digits/quoted の呼び出しを 1 箇所置換（スモーク緑確認）
- [ ] seam インスペクタの出力を CI に収集（prelude_brace_delta==0 をゲート）
- [ ] 代表“揺れ”合成セットで正常化率を測定（before/after）

メモ（用語）
- prelude_brace_delta: 結合前置部のブレース差分（0 で正常）。
- dup_box/dup_fn: 結合により重複生成された Box/関数の検出ログ。

参考タイトル案
- AI‑in‑the‑Loop Normalization of Evolving JSON IRs with Deterministic C‑ABI Boxes
- Seam‑Aware JSON Unification: From AI Pre‑Fixing to Box‑Level Normalization

ドラフトの所在
- 本ディレクトリ配下に章立ての追加ファイルを随時作成（outline.md, experiments.md など）。

