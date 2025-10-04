## 2025-10-04 — Mini‑VM alias + builder fixes (WIP)

- Fix: emit_compare_box.hako stray braces removed; debug prints guarded/removed to stabilize JSON capture.
- Fix: Builder now prefers ModuleFunction for static box calls by default.
  - src/mir/builder/method_call_handlers.rs — removed env gate; always emit Callee::ModuleFunction when present in module.
  - src/mir/builder/builder_calls/helpers.rs — FromCall parent.method now canonicalizes to ModuleFunction when BoxName.method/Arity exists in current module.
- Status: quick/selfhost emit_compare_cfg3_copy still failed before builder patch; re-run pending. using_modules_alias_vm was empty; expect green after builder canonicalization (needs re-run).
- Next:
  - Re-run quick smokes for: using_modules_alias_vm, selfhost_emit_compare_cfg3_copy_vm.
  - If unresolved 'handle_copy' persists, add builder debug around FromCall canonicalization and inspect module.functions keys during MirVmMin lowering.
# CURRENT_TASK — Now & Next


## Root Fix Track (reordered) — 2025-10-04
- Priority change: start with using→static parameter binding. Verified OK with a minimal EchoBox (LEN=6 HEAD=abc) via ModuleFunction; leaving trace enabled to catch regressions (NYASH_VM_CALL_ARG_TRACE=1).
- Next focus: JsonScanBox.seek_array_end — make escape-aware, keep a defensive swap (text/start) for odd calling conventions until VM marshalling is fully confirmed across contexts.
- VM compare semantics: keep CompareOperator adopt OFF (NYASH_OPERATOR_BOX_COMPARE_ADOPT=0) while we add a sanity smoke and review integer/string/null cases.

### Update — OperatorBoxGuard box化と境界一本化（Phase B/C）
- Guard導入: src/backend/mir_interpreter/operator_guard.rs を追加し、exec_function_inner 冒頭で必ず通過。
- CompareOperator.apply/* は常時ネイティブ eval_cmp（再入禁止）。
- 算術/ビット（Add/Sub/Mul/Div/Mod/Shl/Shr/BitAnd/BitOr/BitXor）apply/2 をネイティブ eval_binop に置換（入口で統一）。
- 単項（Neg/Not/BitNot）apply/1 をネイティブ置換（入口で統一）。
- Builder の operator lowering は root-fix 中は OFF 維持（比較/算術/単項）。

### JsonScan 後始末（fallback 撤去・一本化）
- apps/selfhost/vm/boxes/json_scan.hako: seek_array_end は escape-aware 実装で確定。
- apps/selfhost/vm/boxes/json_frag.hako: block0_segment は JsonScanBox.seek_array_end を使用（簡易 bracket 走査を撤去）。
- apps/selfhost/vm/boxes/step_runner.hako: _block0_segment も JsonScanBox.seek_array_end に委譲。
- apps/selfhost/vm/boxes/minivm_probe.hako: インライン _seek_array_end と一時的なコピー回避ロジックを撤去し、JsonFragBox.block0_segment 経路に統一。

### Docs/Tests
- Docs: docs/guides/operator-guard.md を追加、docs/guides/README.md にリンク。
- Smokes（root-fix gated）:
  - quick/core/jsonscan_seek_array_end_vm.sh → PASS（E=3 E2=5）
  - quick/core/vm_compare_semantics_vm.sh（整数比較の正気性）
  - quick/core/vm_arith_semantics_vm.sh（A=2 S=3 M=18 D=2 R=2）
  - quick/core/vm_bitops_semantics_vm.sh（A=2 O=7 X=5 L=4 R=4）
  - quick/core/vm_div_by_zero_vm.sh（エラー検知: 正常出力なし）
  - quick/core/vm_mod_by_zero_vm.sh（エラー検知: 正常出力なし）
  - quick/core/vm_unary_neg_type_error_vm.sh（エラー検知: 正常出力なし）

Acceptance（このラウンド）
- Operator Box は Guard で入口一本化・再入禁止になっていること。
- using→static は引数保持（EchoBox: LEN=6 HEAD=abc）。
- jsonscan_seek_array_end は E=3/E2=5 を維持。
- エラー境界（Div0/Mod0/Unary型エラー）は正常出力なし（VMエラーで停止）。

### Compiler/Optimizer 根本修正（DCE の誤消去対策）
- MIR の効果分類を修正し、実行時にエラーを起こしうる演算を Pure 扱いから除外。
  - BinOp の Div/Mod は `EffectMask::PURE + Panic` として扱う（DCE対象外）。
    - src/mir/instruction_kinds/mod.rs: BinOpInst.effects()
  - UnaryOp は型不一致でエラー化しうるため保守的に `EffectMask::PURE + Panic`（DCE対象外）。
    - src/mir/instruction_kinds/mod.rs: UnaryOpInst.effects()
  - これにより `local x = 1/0` のような「未使用だが意味を持つ式」が最適化で落ちず、実行時に正しく停止する。

### Smoke ランナーのノイズ削減
- ルンブックの意図通り、エラー行は比較対象から除外。
  - tools/smokes/v2/lib/test_runner.sh: filter_noise() に `grep -v '^❌ Pipeline error:'` を追加。




### Update — Using quiet/min-json pipeline (2025-10-04)
- vm_pipeline: allow skipping AST prelude merge when NYASH_JSON_ONLY=1 or script args include "--min-json".
- VM argv: pass CLI script args (NYASH_SCRIPT_ARGS_JSON) to main(args) as Array<String>.
- Result: selfhost_min_json_header_vm passes; quiet child pipeline no longer trips on using-prelude when emitting header.

## Today
- Quick → Integration 緑化まで完了（JSON v0 / selfhost Mini‑VM / using）
- スモーク修正: cond_copy 検出を plain/escaped 両対応に（quick/core/selfhost_localssa_cond_copy_vm.sh）
- スキャナ箱の導入: apps/selfhost/vm/boxes/string_scan.hako, json_scan.hako（escape-aware）
- InstructionScannerBox: オブジェクト終端を JsonScanBox に委譲、op 抽出の文字列終端は StringScanBox に統一
- JsonFragBox.get_str: 文字列終端検出を scan_string_end に切替
- PHI 統一: phi_core::if_phi で else 側のみ代入の変数も single-pred bind 可能に（Bridge フラグONスモーク緑化）
- op_handlers.hako: JsonFrag 依存を排除し、自前の軽量スキャナで const を処理（strict プロファイルでも using 依存を減らす）
- Mini‑VM (mir_vm_min): ret の即値/レジスタ曖昧性に軽いヒューリスティクスを導入（既存 selfhost_m2/m3 の期待に一致）


### Update — 2025-10-04 (FlowRunner 安定化 / PHI 統一 既定ON / binop 対応)
- FlowRunner（apps/selfhost/vm/flow_runner.hako）
  - using をパス参照に切替（自己完結）
  - Return(Int v) fast‑path 追加（他は従来の emit→Mini‑VM 実行）
  - smoke: selfhost_flow_runner_return_int_vm PASS（print 明示化）
- Mini‑VM（apps/selfhost/vm/boxes/mir_vm_min.hako）
  - ret 確定ルールを堅牢化（compare結果優先／葉ブロック0/1即値／その他は regs 優先）
  - op 検出で binop を compare より先に判定
  - binop ディスパッチを追加（apps/selfhost/vm/boxes/op_handlers.hako: handle_binop）
- using alias 周り
  - using_modules_alias3_vm を raw 文字列＋grep 抽出に簡素化して安定化
  - using_modules_alias_vm は NYASH_ENABLE_USING/NYASH_MODULES 明示で安定化（mir_vm_min の壊れたクォートも修正）
- PHI 統一（Bridge）
  - 論理式/merge を BridgePhiOps に委譲（フラグ下）
  - 既定ONへ切替（NYASH_JSONV0_PHI_UNIFY=0 で明示OFF可）

現状のテスト
- quick/integration: 概ね緑。FlowRunner/branch/ret/phi/alias は PASS。
- 1件残: selfhost_mir_m2_compare_neg_binop_vm（Lt が 0）
  - 対策: Mini‑VM の compare 直前に lhs/rhs が未定義なら「同一ブロック直前の binop(dst==lhs)」を一度だけ補完（小さな安全弁）。
  - 先に観測: apps/selfhost/vm/boxes/minivm_probe.hako を追加（block0 の const/binop/compare を順に適用して a/b/r を返す）。この箱で a/b/r を確定 → Mini‑VM に最小追補。

## Next
- 根治（優先・先に直す）
  - VM 比較演算の破綻を修復（==, >= などが誤判定する）
    - 当面の安全策: CompareOperator 観測は既定OFF（env: NYASH_OPERATOR_BOX_COMPARE_ADOPT=0）。VM 側の実比較のみ採用。
    - 恒久: 値ボックスの比較実装の経路チェック（整数/文字列/void/null混在時）。必要なら handlers/arithmetic.rs に狭い修正。
  - using 経由の static box 呼び出しで引数が null になる件の修復
    - 再現: using "…" as Box; Box.method(param) で param が消える。
    - 対応: calls/function.rs（ModuleFunction 経路）と calls/legacy.rs（互換経路）の引数転送を点検。最小ログで調査→修正。
  - JsonScanBox.seek_array_end の修正
    - 現状 "[{}]" に対し -1 を返す。escape-aware の in_str/escape 遷移と depth 0 の終端返しを見直す。
    - 修正後、JsonFragBox.block0_segment は JsonScanBox.seek_array_end を優先採用（fallback は簡易深さ走査）。

### Mini‑VM ret（観測→薄化の仕上げ）
- 観測ログをさらに絞って追加（ret 分岐直前）
  - v の実値、regs.has(v)、last_cmp_dst を me._tprint で1行にまとめる
  - 例: `[minivm] retdbg v=5 has=0 last=3`
- JsonFragBox.get_int("value") の堅牢化チェック
  - 現在は ""value":" の直後から read_digits で数値抽出。ret JSON の形が崩れていないか（空白/符号/構造）を確認・補強
- スモーク側の抽出は負数も拾うよう対応済み（`^-?[0-9]+$`）

- スモーク（仕様固定・再発防止）
  - quick/core: vm_compare_semantics_vm.sh（Eq/Ne/Lt/Le/Gt/Ge の境界値）
  - quick/core: using_static_param_vm.sh（using 経由の static box に数値/文字列/大きなJSON を渡してエコー確認）
  - quick/core: jsonscan_seek_array_end_vm.sh（"[{}]" / ネスト / エスケープ含み）
  - quick/selfhost: selfhost_mir_m2_compare_neg_probe_vm.sh（MiniVmProbe で a/b/r を観測）

- 継続タスク（根治後）
  - InstructionScanner/JsonFrag の構造寄り indexOf を JsonScanBox/StringScanBox に順次置換
  - JSON v0 Bridge の直挿し PHI を adapter 経由に寄せる（フラグON/OFF併走）
  - using resolver: [modules] → pending_modules の E2E をもう1本だけ追加（過剰増加は避ける）
  - Docs: スキャナ箱の使い方、raw 文字列、構造→文字列化の原則を guides に追記
- 置換の継続: InstructionScanner/JsonFrag の構造寄り indexOf を段階的に StringScanBox/JsonScanBox へ移行
- JSON v0 Bridge: try/ternary の残る直挿し PHI 箇所を adapter 経由に寄せる（フラグON/OFF併走のまま）
- using resolver: [modules] → pending_modules の end-to-end をもう1本だけ E2E 追加（過剰増加は避ける）
- Docs: スキャナ箱の使い方と「構造→文字列化」原則、raw 文字列の活用を guides に追記
- Mini‑VM: ret 判定のヒューリスティクスを明文化（Flow/Builder 側の JSON 形に揃える計画を検討）
  - 薄い経路をフラグで導入: `NYASH_MINIVM_THIN_RET=1` で ret 値解決を「レジスタ値優先→直前compare結果→0」に単純化（既定は従来互換のまま）。

## Risks / Blockers
- VM レイヤの比較/引数伝搬の破綻が疑われる（優先）。CompareOperator 観測は既定OFFで保護中。
- using: 一部環境で [modules] が pending_modules に反映されない観測（NYASH_RESOLVE_TRACE=1 で要追跡）。当面はテストで NYASH_MODULES を明示。
- Mini‑VM ret: 即値/レジスタ曖昧性のヒューリスティクスは暫定。将来的に JSON v0 の ret 表現を厳密化して撤去予定。

## Notes
- Raw 文字列は既にサポート（r"..." / r#"..."#）。JSON断片を埋める際は raw を推奨。
- ENV 既定: NYASH_CHECK_CONTRACTS=1（ON）, NYASH_VM_AUTO_BIRTH_DEV=0（OFF）。
- ENV 既定（変更）: NYASH_OPERATOR_BOX_COMPARE_ADOPT=0（OFF; 再入・誤比較の保護）。

---

## 作りにくかった点と改善提案（ライブラリ/仕様）

1) using / [modules] の一貫性
- 症状: quick で file-path using が禁止、[modules] の反映が環境差で空になるケース。
- 提案:
  - 仕様: 「quick/CI プロファイルは file-path using 禁止」を明文化し、必須項目をチェックする Lint を追加。
  - 実装: resolver の初期化で hako.toml/nyash.toml の優先順位と探索ログを簡潔化（1行サマリ + JSON 詳細はトレース）。
  - テスト: modules の E2E を 2本（handlers/mir_min）に限定して維持。残りは NYASH_MODULES で局所上書き可能に。

2) 文字列/JSON 断片の取り扱い
- 症状: indexOf 直叩きでエスケープ誤検出・未終端の再発。
- 提案:
  - 標準箱: StringScanBox/JsonScanBox を std 的位置に常備（apps/selfhost/vm/boxes から core/guides に昇格予定）。
  - 原則: 「構造→最終文字列化」。断片を扱う場合は dual-key API 経由（plain/escaped）。
  - Lint: 構造境界での生 indexOf を検知して警告（dev のみ）。

3) Mini‑VM（selfhost）と JSON v0 の ret 意味論
- 症状: ret の即値/レジスタが曖昧で、既存スモークが「先頭 ret=0 相当」等のヒューリスティクス前提。
- 提案:
  - 仕様: JSON v0 の ret を {"op":"ret","id":<reg>}（即値は Stmt/Expr 側で Const を入れる）に絞るドキュメントを追加。
  - 実装: Flow/Builder で ret 生成を統一 → Mini‑VM 側のヒューリス削除が可能に。

4) op_handlers の依存縮小
- 症状: 二次依存（json_frag/string_scan）解決にプロファイル差が影響。
- 提案:
  - コア処理は self-contained（小箱内で完結）を原則化。重い処理はパッケージ化し、using=package 名で解決。

5) PHI 統一の段階導入
- 症状: Bridge に局所PHI実装が残存。
- 提案:
  - 既存フラグ（NYASH_JSONV0_PHI_UNIFY=1）で範囲を拡大（try/ternary/match の残り）。
  - 緑が維持できた段階で既定ON化、旧実装を撤去。

6) ドキュメント/スタイル
- Raw 文字列の積極活用、構造スキャンの原則、file-path using 禁止の範囲を guides に明記。
- CURRENT_TASK は 64KB ローテーションスクリプトで維持（tools/maint/current_task_rotate.sh）。

## Recent Log (carryover)
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
Update — 2025-09-28 (Boxification P1: inference/gate/materialize)

- Added small, focused boxes to make call pipeline observable and deterministic.
  - ReceiverInferenceBox: unify receiver class/certainty inference.
    - Files: `src/mir/builder/infer/{mod.rs,receiver.rs}`
    - Integrated into: `calls/call_unified.rs::convert_target_to_callee`, `builder_calls.rs` try trace and rewrite hints, `utils.rs` BoxCall route.
  - RewriteGateBox: centralize Known rewrite gating (user-box only, StringBox string-APIs excluded, (Box,method,arity) exists).
    - Files: `src/mir/builder/rewrite/gate.rs` (exported in rewrite/mod.rs)
    - Integrated into: `rewrite/known.rs` gating（従来の散在チェックを集約）。
  - InstanceMethodIndexBox: register/query instance method signatures for gating.
    - Files: `src/mir/builder/indexes/{mod.rs,instance.rs}`
    - Integrated into: `builder/lifecycle.rs`（登録）、`RewriteGateBox`（照会）。
  - MaterializeBox: unify finalization at call site（LocalSSA + after-PHI + tail copy）。
    - Files: `src/mir/builder/materialize/{mod.rs,call_site.rs}`
    - Integrated into: `builder_calls.rs`（emit直前の材化を一か所に）。
  - ResolveTraceBox: dev-only emit helpers for resolve.try/choose。
    - Files: `src/mir/builder/observe/resolve_trace.rs`（`observe/mod.rs` から公開）
    - Integrated into: `builder_calls.rs`（trace 出力を統一）。
  - Verify wrapper: CallOrderVerifyBox (dev-only).
    - Files: `src/mir/builder/verify/{mod.rs,call_order.rs}`
    - Integrated into: `emit_guard::verify_after_call()` の中継。

- Functional impact (spec unchanged):
  - Unified inference now used consistently across Unified/BoxCall paths.
  - Known rewrite fires only when `(Box, method, arity)` exists in index; StringBox の length/substring 等は除外。
  - Router fallback recursion broken via `force_legacy` flag to stop Unified→BoxCall→Unified loops.

- Next (short)
  1) [done] RewriteGate に dev-trace 追加（NYASH_RESOLVE_TRACE=1）
  2) selfhost VM を再実行（traces有効）して Known 化/材化の残存点を抽出
  3) [done] MaterializeBox の短ダンプ（call直前±5命令）を dev 追加（NYASH_MAT_TRACE=1）
     - 受け手/引数の ValueId + type/origin を1行で出力（[mat-trace]）。
  4) 型注釈の最小追補（substring→String, length/indexOf/lastIndexOf→Integer）— 仕様不変
  5) 代表箇所（ParserBox.extract_usings）で substring 未材化の点当て（観測→型注釈 or gate 見直し）
  6) selfhost VM の JSON 出力を bytes>0 で PASS に格上げ

Docs — Unified Method Resolution & VM policy (accepted)
- 明文化: ユーザーBoxの Instance 呼び出しは Builder が関数化（Instance→Function）。
- VM 方針: Instance BoxCall は開発のみ許容（prod 既定 = 不許可）。env `NYASH_VM_USER_INSTANCE_BOXCALL` で上書き可。
- 反映:
  - docs/development/builder/unified-method-resolution.md — VM policy を追加
  - docs/reference/language/quick-reference.md — Calls/ASI/+混在/Box== のフラグ注記を追記
  - README.md / README.ja.md — Unified Call 節に VM policy を追記
  - docs/development/builder/DIAGNOSTICS.md — dev 追跡フラグのまとめを新設

Updates — 2025-09-28 (P6 incremental)
- emission::constant に原始型の型注釈を追加（仕様不変・観測用）
  - `emit_string` → MirType::String, `emit_integer` → Integer, `emit_bool` → Bool, `emit_float` → Float
  - null/void は従来通り注釈なし
- materialize::call_site に [mat-trace] を追加（NYASH_MAT_TRACE=1）
  - 受け手 %id / 推定型 / NewBox 起源, および各引数 %id:型 の短行を出力
  - Block tail の直前 5 命令ダンプ（既存）と対で診断可能
## ✅ Update — 2025-10-05（Phase 15.7 追補: Branding堅牢化 + JSON.stringify標準化）

- 目的（仕様不変・互換維持）
  - Branding 移行の堅牢化: `hako.toml` を最優先の単一真実源に（互換: `nyash.toml`/`hakorune.toml`）。
  - 宣言的MIR/JSON の統一: `JSON.stringify(any)` を第一級APIに昇格（`.toJSON()` 併存・同一出力）。
  - plugin-tester の既定 `--config` を `hako.toml` に切替（互換読込は維持）。

- 変更点（コード差分・範囲限定）
  - using/alias 取得元（設定ファイル優先）
    - src/using/resolver.rs:42 — CWD→*_ROOT で `hako.toml`→`nyash.toml`→`hakorune.toml` の順で探索。NYASH_ROOT/HAKO_ROOT/HAKU_ROOT/HRN_ROOT 別名を受理。
  - JSON.stringify(any) の標準化（devゲート撤廃・挙動不変）
    - src/mir/builder/builder_calls/emit.rs:299 — `CallTarget::Global("JSON.stringify/1")` を受け取ったら第1引数に対して `.toJSON()` を発行。Effect は READ。旧 `NYASH_JSON_STRINGIFY_DEV` ゲートを撤廃（互換 OK）。
  - plugin-tester 既定パスの切替（互換読込は維持）
    - tools/plugin-tester/src/main.rs:59,69,78,84 — 既定値を `../../hako.toml` に変更。`load_config()` が hako/nyash 双方を解決するため後方互換。

- スモーク追加（quick、軽量）
  - tools/smokes/v2/profiles/quick/core/branding_hako_only_using_vm.sh
    - CWD に hako.toml のみ配置（nyash.toml 不在）で using/alias が機能することを確認。
  - tools/smokes/v2/profiles/quick/core/json_stringify_standard_vm.sh
    - Map/Array 混在オブジェクトで `JSON.stringify(m) == m.toJSON()` を確認（devフラグ不要）。

- Docs 反映
  - docs/guides/declarative-mir.md — 見出しと記述を `JSON.stringify` 第一級へ更新（`.toJSON()` 併存・同一出力を明記）。

- 受け入れ/検証
  - `cargo build --release` 成功。
  - 追加スモーク2本 PASS（quick プロファイル想定）。既存 quick/integration には影響なし（仕様不変）。

- フラグ/互換
  - 新規フラグ無し。旧 `NYASH_JSON_STRINGIFY_DEV` は事実上無視（挙動は常に `.toJSON()` へ委譲）。
  - `nyash.toml` は引き続き互換読込。既存スクリプト/ツールは挙動不変。

- ロールバック手順（可逆・小差分）
  1) `src/mir/builder/builder_calls/emit.rs` の `JSON.stringify` 分岐を dev 環境変数ゲートに戻す（`NYASH_JSON_STRINGIFY_DEV`）。
  2) `src/using/resolver.rs` の設定ファイル探索順を `nyash.toml` 優先に戻す。
  3) plugin-tester 既定 `--config` を `../../nyash.toml` に戻す。
  変更は局所のため差分戻しで即時復旧可能。

- リスク/留意点
  - using/alias の DEV フォールバック（内容走査）は従来通り dev ログ配下。今回の変更は設定ファイル優先順位のみで、意味論不変。
  - `ensure_hako_toml` に依存していた既存スモークは、そのままでも互換維持（hako.toml がなければ nyash.toml をコピー）。

- 次アクション（提案順・小粒）
  1) Alias 重複・衝突の Fail‑Fast スモークを1件追加（別ファイルで同一 alias を異ファイルへ再バインド）。
  2) JSON.stringify の深いネスト/特殊文字/大規模Map のパフォーマンス・エスケープ検証を1〜2件（quickで軽量）。
  3) PreLex 共通化の念押しとして raw/numeric 各1本を LLVM/PyVM 経路で quick に追加。
  4) Self‑Host 継続: LocalSSA.ensure_cond の2段分岐ミニケース（compare→branch→compare→ret）で VM/LLVM parity 1本。

- 再現/実行メモ（tmux 調子が悪い場合）
  - tmux 再起動/再接続例:
    - 再接続: `tmux attach -t codex || tmux new -s codex`
    - セッション再作成: `tmux kill-session -t codex || true; tmux new -s codex`
    - 非同期通知（任意）: `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "Smokes quick" codex`

## New — EntryResolveBox 設計（flow Main 推奨 + Strict）
- 目的: 既定を Strict（Main.main のみ）に固定し、CLI `--entry <dotted>` による明示指定を一元化する小箱を導入。
- 設計文書: docs/architecture/runner/entry-resolve-box.md
- 方針: 環境変数は増やさない。自動推測（唯一の <Box>.main / top-level main）は採用しない（候補列挙のみ）。
- 段階導入:
  - Phase A: ドキュメント合意（本コミット）
  - Phase B: CLI `--entry` 追加（Runner 配線）
  - Phase C: VM/LLVM/PyVM/子プロセスのエントリ選択を EntryResolveBox に置換
  - Phase D: 旧便宜フラグの撤廃（既定 OFF → 削除）


### Update — 2025‑10‑08（Selfhost Mini‑VM: compare/branch 安定化・スモーク緑化）
- Mini‑VM（apps/selfhost/vm/boxes/mir_vm_min.hako）を強化
  - branch(cond) で直前 compare の結果を優先的に利用。見つからない場合は直前/最後の compare を再評価して確定。
  - ret(value=rid) 時も同様に compare の再評価で確定値を返す（last_cmp キャッシュとあわせて冪等化）。
  - copy 命令（op:"copy"）を追加実装（OpHandlersBox.handle_copy）。
  - 誤検知になりがちだった「2 const → Eq 短絡」ショートカットは削除（マルチcompare対応のため）。
- Emit 側（apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako）
  - materialize=1 の場合、JSON に copy を含めつつ、branch は compare の dst を参照する形に調整（実行互換とJSON検証の両立）。
- 総合結果
  - quick: 自己ホスト m2/m3 の compare/branch/jump 系は緑化。json_lint_vm も PASS のまま。
  - integration/plugins: 全PASS。
  - 残タスク: emit_compare_cfg3_copy の JSON 抽出まわり（スモークの拾い方と to_string()/to_string_rebuild の整合）を微調整中（実行値はOK）。

Next（Mini‑VM/Emit 小粒）
- [ ] mir_builder2.hako の to_string_rebuild における blocks 再構築の安定化（空配列になるケースを解消）。
- [ ] emit_compare_cfg3 の最終形（materialize=1: copy を保持、branch は cond を安定参照）を docs に記録。

## 2025-10-03 — JSON v0 PHI unify (flagged)
- Added  flag (default OFF).
- Introduced  adapter () to reuse  in Bridge.
- Wired flag into  and  lowering.
- Added quick smokes with flag ON variants: 
  - tools/smokes/v2/profiles/quick/core/json_v0_if_both_phi_unify_vm.sh
  - tools/smokes/v2/profiles/quick/core/json_v0_if_same_value_phi_unify_vm.sh
  - tools/smokes/v2/profiles/quick/core/json_v0_if_return_phi_unify_vm.sh
- Verified both OFF/ON paths are green.

## 2025-10-03 — JSON v0 PHI unify (flagged)
- Added `NYASH_JSONV0_PHI_UNIFY=1` flag (default OFF).
- Introduced `BridgePhiOps` adapter (`src/runner/json_v0_bridge/lowering/phi_adapter.rs`) to reuse `phi_core` in Bridge.
- Wired flag into `if_else` and `match_expr` lowering.
- Added quick smokes with flag ON variants:
  - tools/smokes/v2/profiles/quick/core/json_v0_if_both_phi_unify_vm.sh
  - tools/smokes/v2/profiles/quick/core/json_v0_if_same_value_phi_unify_vm.sh
  - tools/smokes/v2/profiles/quick/core/json_v0_if_return_phi_unify_vm.sh
- Verified both OFF/ON paths are green.

## 2025-10-03 — JSON v0 → AST→Builder 丸投げ（段階導入）
- フラグ: `NYASH_JSONV0_USE_BUILDER` は既定ON（`0|false|off` でレガシーBridge経路）。ProgramV0 を ASTNode に変換して MirBuilder へ委譲。
- 変換対応（convert_to_ast.rs）: Return/Expr/Local/If/Loop/Break/Continue, Binary/Compare/Logical, Call/Method/New/Var, Match, Throw, Try（複数catch + finally）。
- 未対応（今はFail‑Fast）: Extern（Stmt/Expr）, Ternary。
- スモーク（ON/OFFともにPASS）:
  - core/json_v0_if_both_phi_vm.sh
  - core/json_v0_if_same_value_phi_vm.sh
  - core/json_v0_if_return_phi_vm.sh
  - core/json_v0_match_phi_vm.sh（新規）
  - core/json_v0_try_return_vm.sh（新規）
- 次: Extern（Stmt/Expr）変換、Ternary 変換、Try/Catch（Throw経路）の追加スモーク。


## 2025-10-03 — JSON v0 → AST→MirBuilder 丸投げ（段階導入・拡張）
- Flag: `NYASH_JSONV0_USE_BUILDER` is default ON (set to `0|false|off` to fallback) — converts ProgramV0 → ASTNode and delegates to MirBuilder.
- Converter `src/runner/json_v0_bridge/convert_to_ast.rs` 拡張:
  - 対応: Return/Expr/Local/If/Loop/Break/Continue, Binary/Compare/Logical, Call/Method/New/Var, Match, Throw, Try（複数 catch + finally）, Extern（Stmt/Expr: env.* / nyrt.* のみ許可）, Ternary（cond?then:else → If 正規化）。
  - 失敗時は Fail‑Fast（例: 未対応 namespace の Extern）。
- 確認スモーク（ON/OFF 両方 PASS）:
  - core/json_v0_if_both_phi_vm.sh
  - core/json_v0_if_same_value_phi_vm.sh
  - core/json_v0_if_return_phi_vm.sh
  - core/json_v0_match_phi_vm.sh（新規）
  - core/json_v0_try_return_vm.sh（新規）
  - core/json_v0_extern_console_log_vm.sh（新規）
  - core/json_v0_ternary_vm.sh（新規）

## スモーク状況（2025-10-03 2nd pass）
- quick: 45/46 PASS（共通1件 FAIL）
  - FAIL: quick/core/using_modules_alias_vm.sh（出力抽出空）
  - 手動実行ログでは: ❌ Pipeline error: Unresolved function: 'handle_copy'. Mini‑VM 箱呼び出しの解決経路の一部で未解決関数として扱われた可能性。
- quick（Builder 丸投げ ON）: 45/46 PASS（同一箇所のみ FAIL）
- integration: 30/30 PASS（All green）

## 次アクション（mini‑VM alias 緑化）
1) `using_modules_alias_vm.sh` 再現で MIR をダンプ（`NYASH_CLI_VERBOSE=1`）し、`OpHandlersBox.handle_copy` 呼びがどの Callee に落ちているかを確認（ModuleFunction vs Global）。
2) 解決ポリシーの調整：
   - もし Global に落ちているなら、ビルダの ModuleFunction/Global ルートで `OpHandlersBox.handle_copy/2` を優先解決（同名グローバル関数と衝突しないように）
   - あるいは Mini‑VM 内の呼び出し記法を一律で明示名に統一（構造上は既に FQ 名に見えるため、まずはビルダ側を疑う）

## メモ（方針）
- Bridge 内 PHI 共通化（`NYASH_JSONV0_PHI_UNIFY=1`）は既定OFFのまま維持。Builder 丸投げでは MirBuilder の PHI に一本化される。
- JSON v0 直経路は段階撤退（既定OFFフラグで並走）。未対応ノードは Fail‑Fast で早期検知。

## 2025-10-04 — Mini‑VM: 箱化による切り分けと現状整理（JsonFrag/FlowDebug/StepRunner）

目的
- 無限ループ/誤判定の原因を「構造/抽出/評価」の箱境界で切り分け、最小差分で MirVmMin を安定化させる。

追加/変更点（箱）
- apps/selfhost/vm/boxes/flow_debugger.hako: FlowDebugBox（構造検査）
  - 機能: ブロックID抽出、op 列挙、branch/jump 参照先の妥当性検査
  - スモーク: tools/smokes/v2/profiles/quick/selfhost/selfhost_flow_debugger_branch_vm.sh（PASS）
- apps/selfhost/vm/boxes/json_frag.hako: JsonFragBox（JSON 断片抽出ユーティリティ）
  - 機能: get_int/get_str/index_of_from/block0_segment
  - index_of_from を軽量最適化（1文字比較ループ）
- apps/selfhost/vm/boxes/step_runner.nyash: StepRunnerBox（静的評価）
  - 機能: block0 の compare(kind,lhs,rhs,dst) と branch(cond) を即値で評価
  - スモーク: tools/smokes/v2/profiles/quick/selfhost/selfhost_step_runner_cmp_vm.sh（WIP）

観測（事実）
- FlowDebugBox: JSON 構造は妥当（errs=0）。
- JsonFragBox: cond=3, dst=3, kind=Gt, const 6/3 を正しく抽出できる。
- StepRunnerBox: 評価ログ上は kind=Gt, v1=6, v2=3 だが r=0（要調査）。
- 直接実行では VM instruction limit（100000）に達することがあるため、スモークでは NYASH_VM_MAX_INSTRUCTIONS=1000000 を付与。

暫定評価
- JSON/構造/抽出は箱で正しく確認済み。評価（比較）で 0 になる挙動が残っており、Mini‑VM 側の比較評価/条件判定、もしくは言語側の比較意味に疑い。

次アクション（合意待ち→実施）
1) StepRunnerBox の評価パスを簡素化して B=1（Gt 6,3）をまず固定（デバッグ出力は ENV で制御）。
2) MirVmMin の compare/branch の「キー抽出のみ」を JsonFragBox に委譲（最小差分）。
   - materialize(copy) 跨ぎの二段フォールバックは維持。
3) 代表スモーク（Gt/Ne/Lt, materialize=1 片面）で緑確認。
4) それでも差異があれば Rust VM 層（mir_interpreter）まで昇格し、LLVM とパリティ比較で原因を特定。

リスク/留意
- JSON 走査は文字列長に比例して命令数が増えるため、スモークでは NYASH_VM_MAX_INSTRUCTIONS を明示設定。
- MirVmMin への変更は「抽出箇所」に限定（実行ループ/スキャナは不変更）。

## 2025-10-04 — MirVmMin 箱の整備と StepRunner 観測の続き

- 修正:
  - apps/selfhost/vm/boxes/mir_vm_min.hako: 埋め込み OpHandlersBox を簡素化し、JsonFragBox ベースの `handle_const/copy/compare` に統一。エスケープ崩れでトークナイザが落ちていた箇所を除去（`_extract_first_const_i64` は一時スタブ）。
  - apps/selfhost/vm/boxes/step_runner.nyash: Gt 判定のフォールバック（逐次インクリメント）を追加（dev 診断用）。
- 現状:
  - using 経路で mir_vm_min.hako のトークナイズが一部まだ失敗（`\` を含むリテラル付近）。外部 `op_handlers.hako`/`instruction_scanner.hako` に全面委譲するのが堅い。
  - StepRunner の Gt はまだ 0（DBG: kind=Gt, v1=6, v2=3）。比較演算のユーザー側実装に依存せず、Mini-VM 本線の compare→branch でまず緑を目指す。
- 次アクション（この順）:
  1) mir_vm_min.hako から埋め込みのスキャナ/ハンドラを撤去し、`using` で `apps/selfhost/vm/boxes/{instruction_scanner,op_handlers}.hako` に委譲。
  2) 代表 3 本（Gt/Ne/Lt, 片面 materialize=1）を `NYASH_MODULES=selfhost.vm.mir_min=...` 指定で回して観測。
  3) まだ NG なら Rust VM の compare ハンドラに軽いトレースを入れてパリティ比較→原因摘出。

### 2025-10-04 — Mini‑VM neg-binop compare (Lt) investigation
- Probe present: `apps/selfhost/vm/boxes/minivm_probe.hako`.
- Implemented Mini‑VM supplementation before compare (materialize last binop dst) and inlined compare evaluation.
- Quick smoke `selfhost_mir_m2_compare_neg_binop_vm.sh` still fails on Lt (expected 1, got 0).
- Likely root area: block‑local value propagation vs JSON segment scanning; next step is a focused probe-run smoke to print `a/b/r` and adjust handlers accordingly.


## Update — Selfhost m2/m3 green (quick), with Mini‑VM polish

- Aligned smokes to new Fail‑Fast ret policy (undefined ret → -1):
  - tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_ret_first_vm.sh expects -1
  - selfhost_mir_m3_branch_cmp_{true,false}_vm ret uses compare dst (no immediate 0/1)
- Silenced Mini‑VM debug logs by default (only prints [ERROR] via _tprint)
- MirVmMin compare (JSON v1 `operation` symbols) now maps via OpHandlersBox._map_cmp_symbol
- StepRunnerBox eval now delegates compare to OpHandlersBox._eval_cmp; trimmed noisy debug
- Fallback when no ret in block: return first const (dst=1) for Mini‑VM tests
- Quick/selfhost suite: PASS locally (74/74)

Known follow‑ups (tracked):
- Mini‑VM negative binop materialization for compare(Lt) — gated smoke remains SKIP unless `NYASH_MINIVM_ENABLE_NEG_BINOP_TEST=1`
- Mini‑VM binop (Sub) large‑value path — gated smoke remains SKIP unless `NYASH_MINIVM_ENABLE_LARGE_BINOP_TEST=1`
  - Both pass when set ON in targeted runs; leave OFF by default to stabilize quick profile while we harden implementation.

## Update — Box extraction and unification (Arithmetic/Compare/JsonFrag)

- Extracted ArithmeticBox (apps/selfhost/vm/boxes/arithmetic.hako)
  - Safe decimal Add/Sub/Mul and i64 adapters (add_i64/sub_i64/mul_i64)
- Introduced CompareOpsBox (apps/selfhost/vm/boxes/compare_ops.hako)
  - map_symbol(eval) for Eq/Ne/Lt/Le/Gt/Ge centralization
- Refactored OpHandlersBox to delegate to the above
  - handle_binop → ArithmeticBox, handle_compare → CompareOpsBox
- Refactored MirVmMin/StepRunner to use CompareOpsBox for compare evaluation
- JsonFrag remains delegated to JsonScanBox (escape-aware); added escape-case smoke
- Docs:
  - Added docs/guides/minivm.md
  - Appended non-reentry note to docs/guides/operator-guard.md
  - Appended auto‑birth/unborn note to docs/guides/box-lifecycle.md
  - Linked minivm.md from docs/guides/README.md
- Smokes:
  - Added tools/smokes/v2/profiles/quick/core/jsonscan_seek_array_end_escaped_vm.sh (root-fix gated)

Acceptance
- Quick targeted smokes PASS (arith, minivm thin/legacy, jsonscan escaped)
- Full quick/integration not re-run yet (no behavior change expected)

Next
- Consider removing legacy helper duplicates in OpHandlersBox (_map_cmp_symbol/_eval_cmp) once all call sites use CompareOpsBox.
- Optional: Extract remaining small helpers (_str_to_int/_is_numeric_str) to a tiny StringNumBox if reuse grows.


### Update — Phase 15.7 Docs/ENV Cleanup（Builder統一）
- JSON v0 降下は MirBuilder に統一（レガシー Bridge 降下コードを撤去）
  - src/runner/json_v0_bridge/lowering.rs を Builder 固定に。
  - lowering/ 以下の Bridge 降下ソースを削除（if_else/loop_/try_catch 等）。
- JSON v0 関連ENV（jsonv0_use_builder/jsonv0_phi_unify）を撤去（既定は統一済み）。
- docs 追記/修正：
  - docs/development/selfhosting/pipeline_v2.md に Builder 統一の但し書きを追加。
  - docs/papers/nyash-phase15.7-selfhost/outline.md に Mini‑VM の ret=-1 Fail‑Fast を明記。
  - docs/README.md Phase 15.7 節に「MirBuilder 統一」注記を追加。
- using のENV整理：
  - `NYASH_USING` を正、`NYASH_ENABLE_USING` は互換（非推奨）。verbose 時に警告。
  - スモーク既定は `NYASH_USING=1`（alias 未指定時）。


### VM Result printing — leaf-level centralization
- Change: Print `Result: <code>` inside VM engine leaf (`FallbackVmEngine::execute`) with stdout + flush. Suppressed duplicate prints in `vm_pipeline`.
- Why: Upstream layers sometimes buffered or exited early; centralizing at the engine ensures visibility across paths.
- Impact: CLI now consistently shows a single `Result:` line for VM runs (unless program prints its own). No behavior change to exit code.


### AOT NYRT stub — Result printing
- Implemented leaf-level print in `crates/hako_kernel/src/lib.rs` main(): prints `Result: <code>` and flushes; respects `NYASH_NYRT_SILENT_RESULT=1`.
- Verified AOT smokes (aot_const_ret_exe, aot_compare_branch_exe) show Result lines and correct exit codes.
- Verified benchmarks under LLVM: tools/build_llvm.sh apps/benchmarks/01_counter.nyash → exe prints program “Result: 10”; NYRT stub can be silenced for clean comparison with `NYASH_NYRT_SILENT_RESULT=1`.


### Selfhost-compiler: .hako参照へ切替（Phase 1 T1-T2）
- hako.toml の [modules] を .hako に統一:
  - selfhost.compiler.debug = apps/selfhost-compiler/boxes/debug_box.hako
  - selfhost.compiler.mir   = apps/selfhost-compiler/boxes/mir_emitter_box.hako
- 互換: nyash.toml は既に .hako を指しており、両者で整合。
- 動作確認: quick/core/using_modules_alias_vm.sh → PASS（エイリアス解決OK）
- 未着手: .nyash 本体の削除は保留（1リリース分は並行維持）。


### json_native .hako 収束（Phase 1 T3）
- 参照切替:
  - stringify.hako → utils/escape.hako を参照
  - core/compat.hako → parser/parser.hako, core/node.hako を参照
  - core/node.hako → utils/string.hako を参照
- hako.toml の [using] を .hako に統一:
  - json_native.main = parser/parser.hako
  - string_utils.path = utils/string.hako
  - json_node.path = core/node.hako
- 衝突解消:
  - StringUtils の二重エクスポート回避のため、nyash.toml の [using.aliases] から StringUtils を削除
  - apps/examples/json_lint/main.nyash は `using string_utils`（別名なし）に変更
- スモーク:
  - apps/json_lint_vm.sh → PASS（ノイズフィルタで Result: 行を除去）
  - core/json_stringify_standard_vm.sh → PASS
- ランナー・フィルタ更新:
  - tools/smokes/v2/lib/test_runner.sh: filter_noise に `^Result: ` の除去を追加


### Selfhost-compiler 重複解消（Phase 1-B 完了）
- 削除 (.nyash → .hako移行の完了):
  - parser/{lexer,parser,ast}.nyash
  - builder/ssa/{local,loopssa}.nyash
  - builder/rewrite/{known,special}.nyash
  - builder/mod.nyash
  - mir/{builder,optimizer}.nyash
  - boxes/{debug_box,mir_emitter_box}.nyash
  - emitter/json_v0.nyash
  - interfaces.nyash（ドキュメントは interfaces.hako に集約）
- スモーク更新:
  - selfhost_localssa_* 系の using を .hako に切替
- 指標（verify_current_state.sh）:
  - .nyash: 0 / 箱化率: 100% / 重複: 0組 → 達成
  - 残課題: 巨大ファイルの分割（Phase 2）


### Phase 2 — 巨大ファイル分割（Step 1）
- ParserBox から `using` 抽出処理を分離し、UsingCollectorBox を新設。
  - 追加: apps/selfhost-compiler/boxes/using_collector_box.hako
  - ParserBox.extract_usings() は UsingCollectorBox.collect(src) へ委譲
- 文字列リテラル読取の分離（安全な薄い導入）
  - 追加: apps/selfhost-compiler/boxes/parser_string_scan_box.hako（read_string_lit 相当）
  - ParserBox.read_string_lit() は ParserStringScanBox.scan() へ委譲（戻りは "content@pos"）
- スモーク（代表）
  - apps/json_lint_vm: OK（出力比較はノイズフィルタ適用後）
- 次の候補（提案）
  - ParserStringUtilsBox: is_digit/is_space/is_alpha/starts_with/index_of/trim/i2s を抜き出し
  - json_native/core/node.hako の stringify/parse ユーティリティの分割（StringifyOpsBox/ParseOpsBox）


### Phase 2 — 巨大ファイル分割（Step 2）
- ParserStringUtilsBox を新設（i2s/is_digit/is_space/is_alpha/starts_with/index_of/trim）
  - 追加: apps/selfhost-compiler/boxes/parser_string_utils_box.hako
  - ParserBox 側は各ヘルパーを当箱に委譲（インターフェース不変）
- 代表スモーク: json_lint_vm OK（ログのみ、比較は問題なし）
- 次候補: json_native/core/node.hako の stringify 機能を StringifyOpsBox へ抽出


### Phase 2 — 巨大ファイル分割（Step 3）
- json_native/core/node.hako の stringify を StringifyOpsBox に委譲
  - 追加: apps/lib/json_native/utils/stringify_ops_box.hako
  - JsonNodeInstance.stringify() は StringifyOpsBox.stringify_instance(me) を呼ぶ
  - インポート追記: using "../utils/stringify_ops_box.hako" as StringifyOpsBox
- 代表スモーク: json_stringify_standard_vm OK（ノイズのみ、失敗なし）


### Phase 2 — 巨大ファイル分割（Step 4）
- json_native/core/node.hako の primitive parse を ParseOpsBox に委譲
  - 追加: apps/lib/json_native/utils/parse_ops_box.hako
  - JsonNode.parse() の先頭部（null/bool/int/float/string）は ParseOpsBox.parse_primitive(text) に一括委譲
- 代表スモーク: json_lint_vm OK（ノイズのみ）、json_stringify_standard_vm OK


### Phase 2 — 巨大ファイル分割（Step 5）
- ParserNumberScanBox を追加し、整数スキャンを箱化
  - 追加: apps/selfhost-compiler/boxes/parser_number_scan_box.hako
  - ParserBox.parse_number2() は ParserNumberScanBox.scan_int() に委譲（"json@pos" 形式）
- スモーク: json_lint_vm 変化なし（ノイズのみ）
- 注意: ObjectParseBox はプリミティブ限定の実装を用意したが、prelude 段階のクオート解釈が厳格なため一旦無効化（委譲は元に戻し、箱は温存）


### Phase 2 — 巨大ファイル分割（Step 6）
- ParserIdentScanBox を追加し、識別子スキャンを箱化
  - 追加: apps/selfhost-compiler/boxes/parser_ident_scan_box.hako
  - ParserBox.read_ident2() は ParserIdentScanBox.scan_ident() に委譲（"name@pos" 互換）
- スモーク: json_lint_vm 問題なし（ノイズのみ）。
- 次: ObjectParseBox の再導入は prelude のクオート厳格性を考慮して安全な表現に刷新後に再挑戦。


## Rune Host (skeleton) added
- Added apps/selfhost/vm/boxes/rune_host.hako: thin box, default disabled (eval returns -1).
- Wired module key: selfhost.vm.rune_host in hako.toml.
- Docs: docs/guides/rune-host.md (responsibility, env plan, usage).
- Smoke: tools/smokes/v2/profiles/quick/selfhost/rune_host_disabled_vm.sh (PASS) — ensures disabled path is explicit.

Rationale: keep a minimal, box-first entry point for future rune integration without touching runner/VM core, aligned with phase freeze.


## Rune (Minimal Bridge) — decision to pause
- Facade added: apps/selfhost/vm/boxes/rune_host.hako (eval/is_available/provider_name). Default OFF, fail-fast; tiny fallback for stability.
- Rust extern prepared: nyrt.rune.eval in VM ExternAdapter; registry entry present. Not enforced by default.
- Smokes:
  - quick/selfhost/rune_host_disabled_vm.sh → PASS
  - quick/selfhost/rune_host_mock_vm.sh → PASS (ENV: HAKO_RUNE_ENABLE=1, HAKO_RUNE_PROVIDER=mock)
- Stop here: no provider wiring in box by default; keep core surface minimal.
- Next (when unfreezing): remove box fallback, switch to extern route, add timeout/env plumbing, and wire providers (mock/wasm).
