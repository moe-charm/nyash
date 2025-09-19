# Current Task — Phase Freeze (Macro Complete)

Updated: 2025‑09‑19

## Status
マクロ基盤は完成・既定ONで安定（AST JSON v0、PyVMサンドボックス、strict/timeout、dump/golden、JSONL trace、プロファイル）。ここで機能を一旦凍結し、自己ホスト/実アプリ開発に注力して磨き込むフェーズに入る。

## Delivered (Macro Platform)
- Built‑in macros (Rust): derive(Equals/ToString) minimal, public‑only + hygiene.
- User macros (Nyash/PyVM): Proxy → child process (NYASH_VM_USE_PY=1, plugins disabled, timeout). Strict=1 by default (fail on error/timeout; strict=0 → identity fallback).
- Dump/Trace: `--dump-expanded-ast-json`, `NYASH_MACRO_TRACE_JSONL=…`.
- Runner routes:
  - Child mode (PoC): `--macro-expand-child <file>` (stdin JSON → stdout JSON).
  - PyVM runner (recommended): dynamic runner includes macro and calls `MacroBoxSpec.expand(json)` once per pass.
- Templates & Tests:
  - Templates: echo_macro (identity), upper_string_macro ("UPPER:" prefix → uppercase suffix).
  - Array/Map examples: array_prepend_zero_macro (prepend 0 to any Array), map_insert_tag_macro (insert {"__macro":"on"} into any Map).
  - Goldens (user macros): identity, upper_string, array_prepend_zero, array_empty, array_nested, array_mixed, map_insert_tag, map_multi, map_esc。
  - Negative smokes: user macro timeout (strict fail), invalid JSON strict fail, invalid JSON non‑strict → identity fallback。
- Capabilities (実効化): マクロNyashのAST静的走査で IO/NET をゲート（strict=fail / 非厳格は未登録=identity）。
- MacroCtx (MVP): gensym/report/getEnv の最小スキャフォールドを提供。
- Self‑host 前展開（PyVM限定）: `NYASH_USE_NY_COMPILER=1` + `NYASH_MACRO_SELFHOST_PRE_EXPAND=1|auto` → AST前展開→MIR→PyVM 実行。
- CI: min‑gate に macro‑golden ジョブと selfhost‑preexpand‑smoke を追加。
- Docs: user‑macros.md / ast‑json‑v0.md / capabilities.md (io/net/env).

## Focus — Freeze & Polish
1) 実アプリの作成と運用（マクロ前展開/PyVM/LLVMラインの動作確認）
2) バグ修正・ドキュメント整備・スモーク/ゴールデン/CI強化（仕様不変）
3) 自己ホスト前展開の観測強化（ログ/スモーク）と安定運用
4) ランタイムcapabilities（io/net/env）のPyVM側実効化は必要になった時点で最小修正

## Next Milestones
- DONE: Self‑host 前展開 既定化（auto）
  - 変更多: `NYASH_MACRO_SELFHOST_PRE_EXPAND` 未設定時に、マクロ有効かつ `NYASH_VM_USE_PY=1` で自動ON（安全策付き）。
  - 追加: `--macro-preexpand-auto` フラグ。
- DONE: 環境変数の整理（CLI中心の導線）
  - 追加: `--macro-top-level-allow`, `--macro-profile {dev|ci-fast|strict}`（非破壊の簡易マッピング）。
- DONE: Top‑level allow 既定値を OFF に変更 + AST検出へ統一（ファイル名ヒューリスティック撤廃）。
- DONE: MacroCtx 契約 PoC — ランナーが `expand(json, ctx)` を優先、失敗時に `expand(json)` へフォールバック。

Next (short)
- Self‑host 前展開: array/map ケースの追加スモーク（auto 動作の追認）。
- `macro-profile` の最終化（ドキュメント化 + CIプロファイルの導入検討）。
- MacroCtx: ctx JSON スキーマの明記（limits/nonce/gensym 提示）と `ctx.gensym` の採用例を追加。

Action Items (next 48h)
- [x] Add selfhost pre-expand smokes for array/map
  - Scripts: `tools/test/smoke/macro/selfhost_preexpand_array_auto.sh`, `selfhost_preexpand_map_auto.sh`
  - Accept: logs contain "selfhost macro pre-expand: engaging" and program exits 0
- [x] Finalize macro profiles
  - Docs: `docs/guides/macro-profiles.md`（dev/lite/ci → envの対応）
  - CLI: `--profile {lite|dev|ci|strict}` 実装済
- [x] MacroCtx example
  - Example macro: `apps/macros/examples/gensym_example_macro.nyash`
- [x] ENV縮約の警告
  - 旧ENV使用時の非推奨ログを追加（TOPLEVEL_ALLOW/BOX_CHILD_RUNNER/BOX_NY_*）

## Phase‑16 Outlook
- MacroCtx (gensym/report/getEnv) and capabilities mapped to `nyash.toml`.
- Attribute‑style (@derive/@lint) macros consolidated via MacroBox.
- span/source map for better diagnostics.

## Final Goal (Replacement Strategy)
- Ultimately replace both the Rust front (beyond minimal bootstrap/backends) and PyVM with Nyash implementations—fully self‑hosted pipeline.
- Steps: front macros → resolver helpers → MIR lowering helpers → Nyash VM → phase out PyVM → reduce Rust to boot/backends only.

## Quick Reference
- Profiles: `--profile {lite|dev|ci|strict}`（dev相当が既定運用）
- Register macros: `NYASH_MACRO_PATHS=apps/macros/examples/echo_macro.nyash`
- Strict/Timeout: `NYASH_MACRO_STRICT=1`（既定）, `NYASH_NY_COMPILER_TIMEOUT_MS=2000`
- Dump expanded AST: `nyash --dump-expanded-ast-json <file.nyash>`
- Self‑host 前展開: 既定auto（PyVM限定）
- Docs: `docs/guides/user-macros.md`, `docs/guides/macro-profiles.md`, `docs/reference/ir/ast-json-v0.md`, `docs/reference/macro/capabilities.md`
