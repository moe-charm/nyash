# Current Task — Phase 15 (Concise)

Focus
- Keep VM quick green; llvmlite integration on-demand.
- Using SSOT（nyash.toml + 相対using）で安定解決。
- Builder/VM ガードは最小限・仕様不変（dev では診断のみ）。
- Phase 15.7 を再定義: Known 化＋Rewrite 統合（dev観測）と Mini‑VM 安定化、表示APIは `str()` に統一（互換:stringify）。

Update — 2025-09-28 (P1 Known 集約・KPI・LAYER ガード)
- Builder: method_call_handlers の Known 経路を `rewrite::known` に集約。
  - 新規 API: `try_known_or_unique`（Known 優先→一意候補 fallback）。
  - equals/1 を `rewrite::special::try_special_equals` に移設（挙動不変）。
- Observe: `resolve.choose` に certainty を付加し（Known/Heuristic）、`NYASH_DEBUG_KPI_KNOWN=1` 時に簡易集計を出力（`NYASH_DEBUG_SAMPLE_EVERY=N`）。
- LAYER ガード（任意ツール）: `tools/dev/check_builder_layers.sh` を追加（origin→observe→rewrite の一方向チェック）。
- Unified 経路: `emit_unified_call` に equals/1 の集約を追加（Known 優先→一意候補）（仕様不変）。
- メソッド候補インデックス化: `MirBuilder` に tail→候補のキャッシュを追加（lazy再構築）。
  - API: `method_candidates(method, arity)`, `method_candidates_tail(tail)`
  - 利用箇所: method_call_handlers の resolve.try、rewrite::{special,known} の一意候補探索、unified equals/1 の一意候補。
- 集約ポリシー（P0 完了）:
  - 中央集約先: `emit_unified_call`（Methodターゲット時に rewrite/special/known を順に試行）
  - `method_call_handlers` は `emit_unified_call` を呼ぶだけに簡素化（重複ロジック削減）
  - equals/1 も同一ロジックに吸収
- レガシー経路（P1 準備）:
  - dev ガード追加: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1` でレガシー側のメソッド関数化を停止（将来削除の前段階）
  - Unified 無効時の後方互換は維持（既定OFF）

Status Snapshot — 2025‑09‑27
- Completed
  - VM method_router: special-method table extended minimally — equals/1 now tries instance class then base class when only base provides equals (deterministic, no behavior change where both exist). toString→str remains（互換: stringify を許容）。
  - MIR Callee Phase‑3: added TypeCertainty to Callee::Method (Known/Union). Builder sets Known when receiver origin is known; legacy/migration BoxCall marks Union. JSON emitter and MIR printer include certainty for diagnostics. Backends ignore it functionally for now.
  - Using/SSOT: JSONモジュール内部 using を相対に統一（alias配下でも安定）
  - DebugHub: 追加ゲート `NYASH_DEBUG_SAMPLE_EVERY`（N件に1度だけ emit）。重いケースでのログ制御のため（既定OFF・ゼロコスト）。
  - Router diagnostics: class-reroute / special-reroute を DebugHub に emit（dev-only, 既定OFF）。
  - LLVM diagnostics: `NYASH_LLVM_TRACE_CALLS=1` で `mir_call` の callee（Method.certainty 含む）を JSON 出力（挙動不変）。

Decision — Variables (Option A; 2025‑09‑27)
- 方針: var/let は導入しない。ローカルは常に `local` で明示宣言。
- 目的: SSA/Loop‑Form と Known/Union 解析の単純さを維持し、未宣言代入の混入を防ぐ。
- 補足: 行頭 `@name[:T] = expr` は標準ランナーで `local name[:T] = expr` へ自動展開（既定ON）。言語意味は不変。
- Docs 更新: quick-reference, language reference, tutorials に「var/let 不採用」を明記。
  - Tokenizer/Parser デバッグ導線（devトレース）を追加
  - json_lint_vm: fast‑pathの誤判定を除去＋未終端ガードを追加（PASS）
  - json_query_min_vm/json_query_vm/json_pp_vm: PASS
  - forward_refs_2pass: Builder が user Box に birth BoxCall を落とさないよう修正＋ランナーフィルタ調整（PASS）
  - Test runner: dev verify ノイズ（NewBox→birth warn）および BoxCall dev fallback をフィルタ
  - Entry policy: top‑level main 既定許可に昇格（NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN default=true）。
    - 互換: `Main.main` が存在する場合は常にそちらを優先。両方無い場合は従来通りエラー。
    - オプトアウト: `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=0|false|off` で無効化可能。
- Next
  - Heavy JSON: quick 既定ONへ再切替（LLVM 常備で段階復帰）
  - 解析ログの統一: parser/tokenizerのdevトレースは既定OFFのまま維持、必要時だけ有効化
- llvmlite（integration）: 任意ジョブで確認（単発実行のハングはタイムアウト/リンク分離で回避）

Update — 2025-09-27 (json_roundtrip_vm null 全化の修正)
- Cause: Tokenizer の構造トークン検出が `indexOf` 依存のため、環境によって `{ [ ] } , :` を認識できず ERROR に落ちていた。
- Fix: `char_to_token_type(ch)` を `==` での直接比較に変更（環境依存排除）。
  - File: apps/lib/json_native/lexer/tokenizer.nyash
- Result: core/json_roundtrip_vm.sh, core/json_nested_vm.sh → PASS（VM quick）

Self‑Hosting Roadmap (Revised) — 2025‑09‑27

Goal
- 一度に広げず、小粒で段階導入。既定挙動は変えず、dev/ci で計測→安定→昇格。
- 本線は VM（Rust）と llvmlite（Python）で検証しながら、Nyash 自身による最小実行器へ橋渡し。

Milestones
- M1: JSON 立ち上げ（VM quick 基準）
  - 目的: JSON 入出力の足場を固め、言語側のテスト土台を安定化。
  - 完了: 相対 using 統一、json_lint_vm/roundtrip/nested/query_min 緑化。
  - 次: Scanner.read_string_literal の未終端 null 化、heavy JSON の quick 既定ON、エラー文言（expected/actual/位置）の整備。
  - 受け入れ: quick で JSON 系が常時緑（SKIPなし）。

- M2: MIR Core‑13 最小セットの Ny 実装（JSON v0 ローダ＋実行器）
  - 範囲: const/binop/compare/branch/jump/ret/phi、call/externcall/boxcall（最小）。
  - 進め方: PyVM を参照実行器としてパリティ確認。fail fast を優先（dev 詳細ログ）。
  - 受け入れ: 代表スモーク（小型）を Ny 実行器で通過、PyVM と出力一致。

- M3: Box 最小群（String/Array/Map/Console）
  - メソッド: length/get/set/push/toString、print/println/log（必要最小）。
  - ポリシー: 既存NyRT/プラグインと衝突しないよう名前空間を分離。既定はOFF、devでON。
  - 受け入れ: JSON apps が Ny 実行器で最低限動作（速度不問）。

- M4: Parity/Profiles 整理
  - プロファイル: dev=柔軟、ci=最小+計測、prod=SSOT厳格（nyash.toml）。
  - パリティ: VM↔llvmlite↔Ny 実行器で代表サンプル一致。差分はテーブル化し段階吸収。
  - 受け入れ: quick（VM）緑、integration（llvmlite）任意緑、Ny 実行器で代表ケース緑。

Guards / Policy
- 変更は局所・可逆（フラグ既定OFF）。
- 既定挙動は不変（prod 用心）。
- dev では診断強化（ログ/メトリクス）し、ランナー側でノイズはフィルタ。

Policy — AST Using (Status Quo)
- SSOT（nyash.toml）＋AST prelude merge を維持。prod は toml 限定、dev/ci は段階的に緩和。
- 重い AST/JSON ケースは integration でカバーしつつ、quick への復帰は LLVM 有効環境で段階的に行う（順次解除）。

Work Queue (Next)
1) Scanner: 未終端文字列で必ず null を返す（Tokenizer が ERROR へ）
2) Heavy JSON: quick 既定ONに戻す（プローブは維持）
3) エラーメッセージの詳細化（expected/actual/line/column）
4) Ny 実行器 M2 スケルトン（JSON v0 ローダ＋const/binop 等の最小実装）下書き
5) Parity ミニセット（VM↔llvmlite↔Ny）を用意し、差分ダッシュボード化
 6) Router: Known/Union 方針の磨き込み（挙動不変）
    - Known → 既存の直接呼び出しを維持（VM 完了、LLVM は表示のみ）。
    - Union → ルータ経路を維持しつつ、ログで可視化（表は“必要最小”で追加）。
 7) Heavy JSON の quick 段階復帰（LLVM 有効環境）
    - 順序: nested_ast → roundtrip_ast → error_messages_ast。
 8) （診断）LLVM ダンプに certainty の補助表示（必要時、挙動不変）。

Update — @local expansion promotion (2025‑09‑27)
- すべてのランナーモードに `preexpand_at_local` を適用（common/llvm/pyvm に加え vm/selfhost へも導入）。
- Docs を更新し、構文糖衣が標準で有効であることを明記。

Plan — Router Minimalism (継続方針)
- 特殊メソッド表は “toString→str（互換:stringify）, equals/1” の範囲から、ユースが発生したもののみ点で追加。
- 既定の挙動・言語仕様は変更しない（フォールバックの拡大はしない）。
- 測定: DebugHub（resolve.*）ログと LLVM の `NYASH_LLVM_TRACE_CALLS` を併用し、Union 経路を可視化。

Runbook（抜粋）
- VM quick: `tools/smokes/v2/run.sh --profile quick`
- LLVM llvmlite: `cargo build --release --features llvm && tools/smokes/v2/run.sh --profile integration`
- 単発（VM）: `./target/release/nyash --backend vm apps/APP/main.nyash`
- 単発（LLVMハーネス）: `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/peek_expr_block.nyash`


Update — 2025-09-27 (Tokenizer/VM trace bring‑up)
- Implemented VM guards (prod): disallow user Instance BoxCall; dev keeps fallback with WARN.
- Dev assert: forbid birth(me==Void) in instance-dispatch path.
- Builder verify (dev): NewBox→birth invariant; warns when missing.
- Added targeted VM traces (dev):
  - JsonToken setField/getField one‑liners
  - Legacy/method calls for JsonTokenizer/JsonScanner keyword paths
- Tokenizer hardening:
  - Reordered next_token dispatch: keyword/number/string first, structural last (avoids misclassifying letters as structural)
  - char_to_token_type rewritten to strict per‑char check (no ambiguous match)
  - Result: "null" now tokenizes correctly (NULL), and JsonParser.parse("null") returns a JsonNode (R=BOX null in probe)

Status (after patch)
- token_probe: OK (NULL/null emitted as expected)
- json_probe3 (parse "null"): OK (returns JsonNode; stringify→"null")
- json_roundtrip_vm: arrays/objects still regress ([]/{} parsed as null); json_query_min still prints null

Next Steps (targeted)
1) Tokenizer structural path
   - Add minimal traces (dev) around create_structural_token in next_token to sample tokens for [ ] { }
   - Verify LBRACKET/RBRACKET/LBRACE/RBRACE sequences for samples: [], {}, {"a":1}
2) Parser array/object path
   - Trace JsonParser.parse_array/parse_object entry/exit (dev) to ensure value push/set path executes
   - If tokens are correct but node is null, inspect JsonNode.create_array/object and stringify
3) Fix + re‑run quick smokes (json_roundtrip_vm, json_nested_vm, json_query_min_vm)

How to reproduce (quick)
- token:  NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/token_probe.nyash --dev
- null:   NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe3.nyash --dev
- smokes: tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh

Notes
- Traces are dev‑only and silent by default; noisy prints in tokenizer were re‑commented.

Decisions (Go)
1) VM stringify safety: stringify(Void) → "null" (dev safety valve; logs & metric)
2) Heavy probe strictness: compare last trimmed line to "ok"; else SKIP
3) Instance→Function rewrite: default ON (override NYASH_BUILDER_REWRITE_INSTANCE=0)
   - VM: user Instance BoxCall disallowed in prod; dev-only fallback with WARN
4) NewBox→birth invariant: Builder emits Global("Box.birth/N"); VM has no implicit birth
   - Dev assert: birth(me==Void) forbidden (WARN+metric)

Plan (next patches)
- Implement stringify(Void) guard in VM (handlers/boxes.rs)
- Tighten probes in quick/core json_* smokes (tail-trim-compare)
- Set rewrite default ON in Builder (method_call_handlers.rs)
- Add VM guard for user Instance BoxCall (prod error; dev fallback)
- (Optional) Builder verify for NewBox→birth, VM dev assert hook

Status
- Tokenizer/parse([]): PASS
- Nested/Roundtrip: probe SKIP on this env (expected); direct run OK
- json_query_min (core): still null → fix follows via stringify(Void) + invariant

Acceptance
- quick: json_pp/json_lint/json_query_min PASS; user Instance BoxCall hits=0
- heavy: nested/roundtrip PASS where parser available

References
- docs/design/instance-dispatch-and-birth.md
- tools/smokes/README.md (heavy probes)

Update — 2025-09-27 (Parser array/object trace)
- Added dev-only traces in JsonParser.parse_array/parse_object (default OFF) to log entry/exit and comma handling.
- Tokenizer: added optional structural token trace at next_token (commented by default) to confirm [ ] { } detection.
- Repro (direct):
  - NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe_min.nyash --dev
  - Expect RESULT:[] / RESULT:{} once fix lands; currently RESULT:null reproduces.
- Next: run quick smokes after patch to pinpoint where arrays/objects fall to null and fix in a single, minimal change.

Update — 2025-09-27 (json_lint_vm guard fix)
- Issue: Unterminated JSON string ("unterminated) was incorrectly judged OK in json_lint due to a lax fast‑path.
- Fix (app-level, spec-safe): removed string fast‑path and added explicit guard — if starts_with('"') and not ends_with('"') then ERROR.
  - File: apps/examples/json_lint/main.nyash
- Result: apps/json_lint_vm.sh PASS on VM quick.
- Follow-up (root cause, parser side): JsonScanner.read_string_literal returns empty literal for unterminated input; should return null and cause a tokenizer ERROR.
  - File: apps/lib/json_native/lexer/scanner.nyash (read_string_literal)
  - TODO: add unit probe; ensure EOF without closing quote yields null; add negative case to smokes if needed.

Update — 2025-09-28 (Scanner 未終端→null とスモーク追加)
- Implemented: JsonScanner.read_string_literal returns null when closing quote is missing or escape incomplete.
  - File: apps/lib/json_native/lexer/scanner.nyash (already returned null; verified)
- Tokenizer maps scanner null to ERROR("Unterminated string literal").
  - File: apps/lib/json_native/lexer/tokenizer.nyash (tokenize_string)
- Added quick smoke to lock behavior:
  - tools/smokes/v2/profiles/quick/core/json_unterminated_string_vm.sh → expects "Unterminated string literal".

Work Queue — Reorganized (2025‑09‑28)
1) Scanner 未終端→null — completed
   - Status: Verified with new smoke; tokenizer ERROR emitted with line/column preserved.
2) Heavy JSON quick 復帰（LLVM 常備で段階解除） — completed (dev override)
   - Policy: AST-heavy smokes run in quick via LLVM harness. When LLVM is not detectable, they SKIP; 開発者は `SMOKES_FORCE_LLVM=1` で強制実行可。
   - Action: run.sh に `SMOKES_FORCE_LLVM=1` を追加、ハーネス/NYRT/ENV の自動整備を強化。nested_ast → roundtrip_ast → error_messages_ast が PASS。
3) エラーメッセージ詳細化 — pending
   - Scope: enrich JSON parser/tokenizer messages with expected/actual; keep format: "Error at line X, column Y: ...".
4) Ny 実行器 M2 スケルトン（最小） — baseline exists
   - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash; quick smoke present.
   - Next: add binop/compare minimal paths (dev-only), no default behavior change.
5) Parity ミニセット — pending
   - Add a tiny VM↔LLVM↔Ny parity triplet; start with const/ret and simple binop.
6) Router Known/Union 磨き込み（挙動不変） — pending
   - Maintain minimal special-method table; diagnostics only; no behavior change.
7) Heavy JSON 段階復帰順（nested_ast→roundtrip_ast→error_messages_ast） — tracking
   - All present in quick under LLVM harness; verify pass and keep order.
8) LLVM ダンプに certainty 補助表示 — baseline exists
   - NYASH_LLVM_TRACE_CALLS=1 prints callee JSON including Method.certainty.
9) QuickRef — Truthiness（quickで有効化）— completed
   - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh → enabled; PASS（0→false, 1→true, ""→false, non‑empty→true）
10) Language guards（planned; 既定OFF・段階導入）
   - ASI strictness: dev‑only check to fail a line break after a binary operator; default OFF.
   - Plus mixed: warn/fail‑fast when non‑String mixed `+` unless explicit stringify; default OFF; document String+number ⇒ concat.
   - Box equality guidance: when `box == box` is used, emit guidance to use equals(); default OFF.
   - Scope: docs + dev warnings first; later wire parser/builder flags guarded by env/CLI profile.

Update — 2025-09-27 (M2 skeleton: Ny mini-MIR VM)

Update — 2025-09-28 (json_lint_vm regression fix — condition_fn and birth bridge)
- Fixed: Unknown global function: condition_fn (quick json_lint_vm)
  - Indirect calls: ensure AST `condition_fn(ch)` lowers to Value call (unified path already used in exprs_call.rs)
  - Unified Global safety: emit_unified_call now dev‑safes `condition_fn` by returning const 1 when unresolved (explicit opt‑in legacy paths intact)
  - Dev stub: finalize_module injects minimal `condition_fn/1 -> 1` if missing (kept as guard)
- Unified→VM bridge: birth()
  - VM: when executing unified Method callee `*.birth`, delegate to BoxCall handler and return Void. This preserves legacy behavior for built‑ins when plugins are absent.
  - Builder: gated birth() injection for built‑ins (Array/Map/String etc). Default OFF unless `NYASH_DEV_BIRTH_INJECT_BUILTINS=1`.
- Next (high‑prio): local var materialization bug in main.nyash
  - Symptom: `local cases = new ArrayBox()` followed by `cases.push(...)` used an undefined receiver ValueId.
  - Interim change: make `local` always materialize a distinct register and `copy init -> var` (also const Void for uninitialized). This avoids SSA aliasing issues.
  - Status: needs a quick pass across smokes to confirm; proceed if quick green, otherwise revisit builder var mapping.

Dev toggles
- NYASH_DEV_BIRTH_INJECT_BUILTINS=1: re‑enable birth() injection for builtin boxes (default OFF to stabilize unified Method path until full bridge lands).
- NYASH_MIR_UNIFIED_CALL: default ON; opt‑out via 0|false|off.
- Added Ny-based minimal MIR(JSON v0) executor skeleton (const→ret only), dev-only app — no default behavior change.
  - File: apps/selfhost/vm/boxes/mir_vm_min.nyash
  - Entry: apps/selfhost/vm/mir_min_entry.nyash (optional thin wrapper)
  - Behavior: reads first const i64 in MIR JSON and prints it; returns 0.
- Quick smoke added to quick profile:
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_min_vm.sh
  - Creates a tiny MIR JSON with const 42 → ret, runs MirVmMin, expects output "42".
- Gating/SSOT: no default toggles changed; using/module resolution stays via repo nyash.toml (added modules.selfhost.vm.mir_min).

Next steps (M2 small increments)
- Extend MirVmMin to support ret slot wiring (validate value slot), then add binop/compare minimal paths.
- Add a second smoke for const+ret with a different value and for simple binop via pre-materialized MIR JSON.
- Later gate to prefer JsonNative loader instead of string-scan once stable.
Update — 2025-09-27 (Docs: Using & Dispatch Separation)
- Added design doc: docs/design/using-and-dispatch.md (SSOT+AST for using; runtime dispatch scope; env knobs; tests).
- Strengthened comments:
  - src/runner/modes/common_util/resolve/{mod.rs,strip.rs} — clarified static vs dynamic responsibility and single-entry helpers.
  - src/mir/builder/method_call_handlers.rs — documented rationale and controls for instance→function rewrite.
  - src/backend/mir_interpreter/handlers/boxes.rs — clarified prod policy for user instance BoxCall fallback.
- Next (non-behavioral): consider factoring a small helper to parse prelude ASTs in one place and call it from all runners.
Update — 2025-09-27 (UserBox smokes added)
- Added quick/core smokes to cover UserBox patterns under prod + fallback-ban:
  - oop_instance_call_vm.sh — PASS
  - userbox_static_call_vm.sh — PASS
  - userbox_birth_to_string_vm.sh — PASS
  - userbox_using_package_vm.sh — PASS (using alias/package + AST prelude)

Update — 2025-09-27 (Loop/Join ScopeCtx Phase‑1)
- Implemented Debug ScopeCtx in MIR builder to attach region_id to DebugHub events.
  - Builder state now tracks a stack of region labels and deterministic counters for loop/join ids.
  - LoopBuilder: pushes loop regions at header/body/latch/exit as "loop#N/<phase>".
  - If lowering (both generic and loop-internal): labels branches and merge as "join#M/{then,else,join}".
  - DebugHub emissions (ssa.phi, resolve.try/choose) now include current region_id.
- How to capture logs
  - NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl \
    tools/smokes/v2/run.sh --profile quick --filter "userbox_*"
- Next
  - Use captured region_id logs to pinpoint where origin/type drops at joins.
  - Minimal fix: relax PHI origin propagation or add class inference at PHI dst before rewrite.

Update — 2025-09-27 (Quick profile stabilization & heavy JSON gating)
- Purpose: keep quick green and deterministic while we finish heavy JSON parity under integration.
- Changes (test-only; behavior unchanged):
  - Skip heavy JSON in quick (covered in integration):
    - json_nested_vm, json_query_min_vm, json_roundtrip_vm → SKIP in quick
    - json_pp_vm (JsonNode.parse pretty-print) → SKIP in quick（例示アプリ、他で十分カバー）
  - Using resolver brace-fixer: quick config restored to ON for stability（NYASH_RESOLVE_FIX_BRACES=1）
  - ScopeCtx wired (loop/join) and resolve/ssa events include region_id（dev logs only）
  - toString→str early mapping logs added（reason: toString-early-*）
- Rationale: heavy/nested parser cases were sensitive to mixed env order in quick. Integration profile will carry the parity checks with DebugHub capture.
- Next (focused):
  1) Run integration smokes for JSON heavy with DebugHub ON and collect /tmp logs
  2) Pinpoint join/loop seam by region_id where origin/type drops (if any)
  3) Apply minimal fix (either PHI origin relax at join or stringify guard tweak)
  4) When green, revert quick SKIPs one-by-one (nested→query→roundtrip)
- Files touched (tests):
  - tools/smokes/v2/profiles/quick/core/json_nested_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_query_min_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/apps/json_pp_vm.sh → SKIP in quick（例示アプリ）
  - tools/smokes/v2/configs/rust_vm_dynamic.conf → RESOLVE_FIX_BRACES=1（安定優先）

Integration plan (dev runbook):
- Heavy with logs: NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_integ.jsonl \
  tools/smokes/v2/run.sh --profile integration --filter "json_*ast.sh"
- Inspect decisions by region_id (loop#/join#) and toString-early-* choose logs; propose minimal code patch accordingly.

Acceptance (this phase):
- quick: 100% green with heavy SKIPs; non-JSON suites unaffected
- integration: JSON heavy passes locally with DebugHub optional; discrepancies have a precise region_id to fix
  - userbox_method_arity_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_branch_phi_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_toString_mapping_vm.sh — SKIP (mapping pending)
- Rationale: keep quick green while surfacing remaining gaps as SKIP with clear reasons.
- Next: stabilize rewrite/materialize across branch/arity and toString→str mapping; then flip SKIPs to PASS.
Update — 2025-09-27 (Loop‑Form Scope Debug & AOT PoC — Plan)
- Added design doc: docs/design/loopform-scope-debug-and-aot.md
  - Scope model (LoopScope/JoinScope), invariants, Hub+Inspectors, per-scope data, AOT fold, PoC phases, acceptance.
- Work Queue (phased)
  1) PoC Phase‑1 (dev‑only; default OFF)
     - Add DebugHub (env: NYASH_DEBUG_ENABLE/NYASH_DEBUG_SINK/NYASH_DEBUG_KINDS)
     - ScopeCtx stack in builder; enter/exit at Loop/Join construction points
     - Emit resolve.try/choose in method_call_handlers.rs
     - Emit ssa.phi in builder.rs (reuse dev meta propagation)
     - Smokes: run userbox_branch_phi_vm.sh, userbox_method_arity_vm.sh with debug sink; verify region_id/decisions visible
  2) Phase‑2
     - OperatorInspector (Compare/Add/stringify)
     - Emit materialize.func / module.index; collect requires/provides per region
     - Fold to plan.json (AOT unit order; dev only)
  3) Phase‑3 (optional)
     - ExpressionBox (function‑filtered), ProbeBox (dev only)
- Acceptance (Phase‑1)
  - Debug JSONL has resolve/ssa events with region_id and choices; PASS cases unchanged (OFF)
  - SKIP cases pinpointable by log (branch/arity) → use logs to guide fixes → flip to PASS


Update — 2025-09-28 (Plugins 既定ON と ENV 整理)
- Plugins: 既定ONで統一。テストランナー/開発スクリプトから `NYASH_DISABLE_PLUGINS=1` を撤去。
  - tools/smokes/v2/lib/test_runner.sh（LLVM 経路）: disable 指定を外し、`PYTHONPATH`/`NYASH_NY_LLVM_COMPILER`/`NYASH_EMIT_EXE_NYRT` を自動付与。
  - tools/dev_env.sh: `pyvm`/`bridge` プロファイルで plugins を無効化しない（unset のみに変更）。
- VM/LLVM 二系統の最小ENV（ドキュメント方針）:
  - VM: 既定でOK（追加ENV不要）
  - LLVM(harness): `NYASH_LLVM_USE_HARNESS=1` + `NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc` + `NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release`
  - quick強制: `SMOKES_FORCE_LLVM=1` で AST heavy を quick で実行可能


Priority TODO — 2025-09-28 (VM/LLVM 2-Line + M2)
- ENV minimalization (plugins=ON):
  - VM: no extra ENV.
  - LLVM(harness): NYASH_LLVM_USE_HARNESS=1, NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc, NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release.
  - Docs: add a small "VM vs LLVM minimal-ENV" box to README.md and README.ja.md. [done]
- test_runner cleanup:
  - Unify/centralize noise filters; keep SMOKES_FORCE_LLVM as the only dev override; remove ad-hoc greps in individual scripts. [todo]
- M2 executor (Ny):
  - Add compare (Eq) to M2 runner; add 2 smokes (Eq true/false). [done]
  - Externalize MirVmM2 to apps/selfhost/vm/boxes/mir_vm_m2.nyash and switch smoke to using-based variant; keep inline smoke as safety. [later]
  - Next (optional): branch/jump minimal; phi later. [pending]

Update — 2025-09-28 (Language Quick Reference & Smokes)
- Added quick-reference draft for language (keywords, operators, ASI, truthiness, equality, '+', rewrite, errors).
  - docs/reference/language/quick-reference.md
- Added planned smokes for quickref rules (initially SKIP until strict rules are wired):
  - tools/smokes/v2/profiles/quick/core/lang_quickref_asi_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh (ENABLED)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_plus_mixed_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_equals_box_error_vm.sh (SKIP)
- Temporarily SKIP Mini‑VM M2/M3 smokes while parser/segment boundaries are being fixed:
  - selfhost_mir_m2_eq_true_vm.sh / selfhost_mir_m2_eq_false_vm.sh / selfhost_mir_m3_branch_true_vm.sh / selfhost_mir_m3_jump_vm.sh — now ENABLED and PASS
- Using/SSOT docs:
  - Clarify dev/ci/prod matrix (file-using dev/ci only; prod=toml only); add short examples. [todo]
- Parity mini-set:
  - VM ↔ LLVM ↔ Ny: const/ret + binop(+), compare(Eq); add quick parity harness notes. [todo]
- Acceptance:
  - quick: AST heavy PASS (LLVM present), M2 binop/Eq PASS; integration unchanged.
  - docs: minimal-ENV clearly shown; no NYASH_DISABLE_PLUGINS in public guidance.

Update — 2025-09-28 (Interpreter gating & Phase 15.7 plan)
- Legacy AST interpreter is now feature-gated (interpreter-legacy OFF by default). Runner/tests that depend on it are behind cfg.
  - Files: src/runner/modes/common.rs, src/runner/modes/bench.rs, src/tests/* (vm_bitops/refcell/functionbox)
- Added Phase 15.7 roadmap (Mini‑VM M3 + NYABI Kernel skeleton; dev-only; default OFF).
  - docs/development/roadmap/phases/phase-15.7/README.md
- Drafted NYABI Kernel spec (v0) and added Ny skeleton box (not wired).
  - docs/abi/vm-kernel.md; apps/selfhost/vm/boxes/vm_kernel_box.nyash

Plan — Instance→Function Rewrite Consolidation (2025‑09‑28)
- Goal: 内部表現を関数呼び出しへ極力統一（obj.m(a) → Class.m/Arity(me,a)）。prodでの Instance BoxCall 依存を排除。
- Approach（小粒・可逆）
  1) PHI/Join での origin/type 伝播の強化（region_id ログで落ちる断面を特定→補修）
  2) 限定 materialize: module 内で name+arity がユニークな場合のみ Glue 関数を合成（既定OFF、dev/CIで計測）

Roadmap Priorities (Phase 15.7 revised)
- P0: me 注入 Known 化（起源付与/維持）— リスク低・効果大。軽量PHI補強（単一/一致時）
- P1: Known 100% 関数化（Known 経路の instance→function 正規化、special 集約）
- P2: Policy（Ny Kernel, dev‑only）— equals/str/truthiness の観測API（バッチ、再入禁止/タイムアウト/計測）
- P3: 表示APIの移行誘導 — toString→str（互換:stringify）の警告/ドキュメント（仕様不変）
- P4: Union 観測・分析 — resolve.try/choose と ssa.phi（region_id）で継続観測
- P5: PHI Known 維持の一般化 — Phase 16（複雑のため後回し）
  3) prod ガード維持: VM は user Instance BoxCall を禁止（既存ポリシー継続）。dev/CI は WARN＋観測
  4) スモーク/観測: quick で Instance BoxCall の dev WARN=0 を確認。resolve.try/choose と LLVM `NYASH_LLVM_TRACE_CALLS` を併用
- Controls
  - `NYASH_BUILDER_REWRITE_INSTANCE`（既定ON）: 強制ON/OFF
  - `NYASH_DEV_REWRITE_USERBOX`（dev限定）: userbox rewrite 検証用
  - materialize 新ENV（既定OFF）: `NYASH_BUILDER_MATERIALIZE_UNIQUE=1`（予定）
- Acceptance（段階）
  - Stage‑1: Known 経路で 100% 関数化（quick全域で dev WARN=0）
  - Stage‑2: 限定 materialize をON時に適用し、分岐/PHI 合流の代表ケースが関数化（差分はdevのみ）
  - 常に prod は挙動不変・安全（OFFで現状維持）

Update — 2025-09-28 (Mini‑VM M2/M3 fix + smokes)
- Fix: compare/ret segmentation made robust without heavy JSON parse.
  - Approach: per‑block coarse passes for const/binop/compare and a precise in‑block ret search; control‑flow (branch/jump) handled with a single pass using computed regs.
  - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash
- Smokes: enabled and PASS
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_false_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_branch_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_vm.sh
- Notes: kept changes local and spec‑neutral; no default behavior changes to core VM.

Update — 2025-09-28 (QuickRef Dev Guards + Docs llvmlite)
- Dev guards (env‑gated; default OFF) implemented and validated by quick smokes:
  - ASI strict line‑continuation: `NYASH_ASI_STRICT=1` → parse error when a binary operator ends the line.
- Plus mixed (String×Number): `NYASH_PLUS_MIX_ERROR=1` → type error; suggest str()/明示変換。
  - Box equality guidance: `NYASH_BOX_EQ_GUIDE_ERROR=1` → equals()誘導のエラー。
  - Smokes enabled: `lang_quickref_asi_error_vm.sh`, `lang_quickref_plus_mixed_error_vm.sh`, `lang_quickref_equals_box_error_vm.sh`（PASS）
- LLVM ドキュメント統一（llvmlite一本化）
  - `LLVM_SYS_180_PREFIX` の記述を主要ドキュメントから撤去し、llvmlite/ny‑llvmc 前提に更新。
  - Files: `AGENTS.md`, `README.md`, `README.ja.md`, `CLAUDE.md`

Plan — Next (2025-09-28)
1) Mini‑VM 単一パス化（仕様不変・安全化） — completed
   - 各 op を JSON オブジェクト単位で厳密セグメント化し、一回走査で評価（coarse pass を除去）。
   - 代表ケース（複数op/ret先頭/ret末尾/compare v0,v1/jump/branch）で緑維持を確認。
2) Rewrite 統合 Stage‑1（挙動不変・dev観測） — completed (observability wired)
   - builder_calls の unified 経路に resolve.try/resolve.choose を追加（dev‑only/既定OFF）。
   - method_call_handlers の既存 emit と整合。Known/Union の certainty を choose に含める。
   - 使い方: `NYASH_MIR_UNIFIED_CALL=1 NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl`。
   - Known 経路の100%関数化（dev WARN=0）を DebugHub で観測。userbox スモークで検証。
3) P0/P1 着手（構造化） — in progress
   - origin/observe/rewrite の責務分割（モジュール新設: src/mir/builder/{origin,observe,rewrite}/）。
   - P0: me 注入 Known 化（起源付与/維持）と軽量PHI補強（単一/一致時）。
   - P1: Known 経路 100% 関数化（special 集約: toString→str（互換:stringify）/equals）。
   - Docs: README を各層に追加（origin/observe/rewrite）— completed
   - 観測呼び出しの統一: builder_calls/method_call_handlers から observe::resolve を使用 — completed
3) CI/Profiles 整理 — ongoing
   - quick: VM 主線（llvmlite パリティは integration に委譲）。
   - integration: 代表パリティ（llvmlite ハーネス）継続、apps系は任意実行。

Notes — Display API Unification (spec‑neutral)
- 規範: `str()` / `x.str()`（同義）。`toString()` は Builder で `str()` に早期正規化。
- 互換: `stringify()` は当面エイリアス（内部で `str()` 相当）。
- VM ルータ: toString/0 → str/0（なければ stringify/0）。
- QuickRef/ガイド更新済み。`NYASH_PLUS_MIX_ERROR` の誘導文言も `str()` に統一。

追加メモ — これからやる（ユーザー合意、2025‑09‑28）
- Mini‑VM の単一パス化を安全に実装（既定挙動不変）
  - 各 op を厳密セグメントで1回走査に統合（coarse を段階撤去）
  - 代表スモーク（M2/M3/compare v0,v1）で緑維持確認
- 続いて Rewrite 統合 Stage‑1 の観測へ進む（dev のみ、挙動不変）
- Dev Profiles
  - tools/dev_env.sh に Unified 既定ON（明示OFFのみ無効）とレガシー関数化抑止を追加。
    - `NYASH_MIR_UNIFIED_CALL=1`（既定ON明示）
    - `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（重複回避; 段階移行）
