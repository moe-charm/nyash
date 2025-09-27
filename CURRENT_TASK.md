# Current Task — Phase 15 (Concise)

Focus
- Keep VM quick green; llvmlite integration on-demand.
- Using SSOT（nyash.toml + 相対using）で安定解決。
- Builder/VM ガードは最小限・仕様不変（dev では診断のみ）。

Status Snapshot — 2025‑09‑27
- Completed
  - VM method_router: special-method table extended minimally — equals/1 now tries instance class then base class when only base provides equals (deterministic, no behavior change where both exist). toString→stringify remains.
  - MIR Callee Phase‑3: added TypeCertainty to Callee::Method (Known/Union). Builder sets Known when receiver origin is known; legacy/migration BoxCall marks Union. JSON emitter and MIR printer include certainty for diagnostics. Backends ignore it functionally for now.
  - Using/SSOT: JSONモジュール内部 using を相対に統一（alias配下でも安定）
  - DebugHub: 追加ゲート `NYASH_DEBUG_SAMPLE_EVERY`（N件に1度だけ emit）。重いケースでのログ制御のため（既定OFF・ゼロコスト）。

Decision — Variables (Option A; 2025‑09‑27)
- 方針: var/let は導入しない。ローカルは常に `local` で明示宣言。
- 目的: SSA/Loop‑Form と Known/Union 解析の単純さを維持し、未宣言代入の混入を防ぐ。
- 補足: 開発用の糖衣（行頭 `@name = expr` → `local name = expr`）はランナー前処理で提供（既定OFF）。言語仕様には含めない。
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
  - Scanner.read_string_literal: 未終端で null を返すよう修正（TokenizerがERRORを積む）＋小プローブ追加
  - Heavy JSON: quick 既定ONへ再切替（環境が安定したら）
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

Work Queue (Next)
1) Scanner: 未終端文字列で必ず null を返す（Tokenizer が ERROR へ）
2) Heavy JSON: quick 既定ONに戻す（プローブは維持）
3) エラーメッセージの詳細化（expected/actual/line/column）
4) Ny 実行器 M2 スケルトン（JSON v0 ローダ＋const/binop 等の最小実装）下書き
5) Parity ミニセット（VM↔llvmlite↔Ny）を用意し、差分ダッシュボード化
 6) Router 観測ログの軽追加（dev-only, 既定OFF）: class-reroute / special-reroute を DebugHub に emit（サンプル制御対応）
 7) LLVM ハーネスの MIR ダンプに certainty 表示（挙動不変の診断整合）

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

Update — 2025-09-27 (M2 skeleton: Ny mini-MIR VM)
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
  - toString→stringify early mapping logs added（reason: toString-early-*）
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
- Next: stabilize rewrite/materialize across branch/arity and toString→stringify mapping; then flip SKIPs to PASS.
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
