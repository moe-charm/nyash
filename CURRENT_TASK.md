# Current Task — Macro Normalize + Freeze (Save Point)

Updated: 2025‑09‑20

## Today (Done)
- PeekExpr → If 連鎖の正変換を安定化
  - マクロ側（IfMatchNormalize）での検出を has_kind("PeekExpr") に統一。
  - Local/Assignment/Return/Print の4経路で PeekExpr を If に置換できるよう整備。
  - 子ランナー（PyVM）経路の不安定さを踏まえ、既定実行を「internal‑child（内蔵変換）」に切替。
    - 内蔵変換（Rust）で Literal‑only の match を If 連鎖へ正規化する安全パスを実装。
  - Golden 緑化: tools/test/golden/macro/match_literal_basic_user_macro_golden.sh
- ランナー診断の強化
  - 子プロセス stderr を親に透過。失敗時にエラー内容を必ず表示（EOF隠れを防止）。
- JsonBuilder の安定化
  - キーワード衝突を回避（local → local_decl）。呼び出し側を追従。
- LLVM PHI 健全性スモーク拡張（If/Matchを追加）
  - 実行には LLVM ビルドが必要。スクリプトは tools/test/smoke/llvm/ir_phi_hygiene_ifcases.sh。
- Scope ヒント設計（no‑op）
  - docs/guides/scope-hints.md を追加（今は観測用のみ）。

重要な運用変更
- ユーザーマクロは継続使用。ただし既定実行は internal‑child（内蔵変換）。
  - NYASH_MACRO_BOX_CHILD_RUNNER の既定は OFF（必要時のみON）。
  - 子環境では NYASH_MACRO_ENABLE=0 などを明示して再帰初期化を抑止。

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

## Delivered (LoopForm – MVP‑1 Safe)
- LoopNormalize macro (Nyash runner) scaffolded and active.
- Canonicalize Loop node key order (condition → body).
- Safe body reorder: move Assignment nodes to tail only when original order already has non‑assign → assign; otherwise keep order (no semantic change).
- JsonBuilder utility added for AST JSON v0 fragments (string‑based minimal helpers).
- Golden comparison made key‑order insensitive (JSON normalized via python) for macro tests.
- LLVM pre‑expand run verified on loop_simple (PyVM → MIR → LLVM).

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
- Match（ガード含む）の正規化を内蔵変換にも拡張（If 連鎖）+ golden/smoke 追加
- DONE: LoopForm MVP‑2 — while → carrier normalization（break/continueなし、最大2変数）
  - 内蔵変換（Rust, internal‑child）で安全ガード付きの末尾整列を実装（非代入→代入）。
  - two‑vars の出力一致スモークを追加（tools/test/smoke/macro/loop_two_vars_output_smoke.sh）。
- DONE: LoopForm MVP‑3 — break/continue 最小対応（セグメント整列）
  - 本体を control 文で分割、各セグメント内のみ安全に「非代入→代入」。
  - スモーク: tools/test/smoke/macro/loopform_continue_break_output_smoke.sh
- DONE: for/foreach 正規化（コア正規化パスへ昇格）
  - 形: `for(fn(){init}, cond, fn(){step}, fn(){body})`, `foreach(arr, "x", fn(){body})`
  - 出力スモーク: tools/test/smoke/macro/for_foreach_output_smoke.sh（for: 0,1,2 / foreach: 1,2,3）
- LoopForm MVP‑3: break/continue minimal handling (single‑level)
- for/foreach pre‑desugaring → LoopForm normalization (limited)
- LLVM IR hygiene for LoopForm / If / Match — PHI at block head, no empty PHIs (smoke)
- Docs: enrich `docs/guides/loopform.md` with carrier examples and JSON builder snippets.
- If/Match normalization pass: canonical If join with single PHI group and Match→If‑chain (scrutinee once, guard fused), expression results via join var.
- ScopeBox (compile-time meta): design + docs; no-op macro scaffold; MIR hint names (no-op) and plan for zero-cost stripping.
- ControlFlowBuilder/PatternBuilder docs and scaffolding: author APIs for If/Match normalization and pattern conditions; migrate macros to use them.

Action Items (next 48h)
- [x] Enable sugar by default (array/map literals)
- [x] Golden normalizer (key‑order insensitive) for macro tests
- [x] Loop simple/two‑vars goldens with normalization
- [x] Match guard: smoke（PeekExpr なし）
- [x] Match guard: golden（literal OR 最小形）
- [ ] Match guard: 追加golden（type最小形、Boxなし構成）
- [x] Smoke for guard/type match normalization（no PeekExpr; If present）
- [x] LoopForm MVP‑2: two‑vars carrier safe normalization + tests/smokes
- [x] LLVM PHI hygiene smoke on LoopForm cases
- [x] LLVM PHI hygiene smoke on If cases
- [ ] ScopeBox docs + macro scaffold (no-op) + MIR hint type sketch
- [ ] ControlFlowBuilder/PatternBuilder docs（本commitで追加）→ スキャフォールド実装 → If/Matchマクロ置換の最初の1本
 - [x] Reorganize macro tests under apps/tests/macro/* and update golden/smoke paths
 - [x] Add MIR hints module (no-op sink) and loop header/latch hint calls

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
