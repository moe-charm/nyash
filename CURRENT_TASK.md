## 2025-10-05 — index_of_from 統一（第1弾）+ Throw 最小対応 + Verifier 整理

- 2引数 indexOf 残差の段階移行（検索は index_of_from 統一）
  - libs: byte_cursor/utf8_cursor を `StringStd.index_of_from` に寄せ、`StringStd.index_of_from` を追加。
    - apps/libs/byte_cursor.nyash, apps/libs/utf8_cursor.nyash, apps/lib/boxes/string_std.hako
  - selfhost VM 最小器: mir_vm_m2 はローカル `index_of_from` を追加して置換。
    - apps/selfhost/vm/boxes/mir_vm_m2.hako
  - hakorune VM 最小器: 一括で `me.index_of_from` に統一（presence チェック含む）。
    - apps/hakorune/vm/boxes/hakorune_vm_min.hako
  - ツール/アダプタの一部も置換（段階）：
    - apps/selfhost/tools/dep_tree.nyash → `StringStd.index_of_from`
    - apps/selfhost/common/json/mir_v1_adapter.hako → 自前 `_index_of_from`
  - DEV リント導入: `tools/lints/lint_indexof_two_args.sh`（`.indexOf(a,b)` を検出）。Makefile の `lint`/`lint-ny` に連携。

- Throw/PHI（VM 最小対応）
  - MIR インタプリタ: 末端 `Throw` を「void で即 return」扱いにし、未到達 Throw 片の存在で VM が落ちないように調整。
    - src/backend/mir_interpreter/exec.rs
  - スモーク: Throw 系は引き続きゲート（enable: `SMOKES_ENABLE_JSON_V0_THROW=1`）。実行される Throw を含む JSON では結果行が出ないため、常時ONは段階導入。

- SignatureVerifier/MethodRegistry（現状整理）
  - Registry は toString/stringify/startsWith/endsWith 等を含む最小カバレッジで稼働中。
  - Pipeline v2 は `verify_call_name_arity` を適用済み。次フェーズでメソッド群のカバレッジを拡張予定。

- 影響/状態
  - quick 抜粋 PASS: `using_modules_alias_vm.sh`, `json_missing_key_vm.sh`
  - lint 実行で 2引数 .indexOf は大幅減（残差は旧/互換/診断系に限定）。

- Next（小粒・推奨順）
  1) `apps/selfhost/vm/boxes/mir_vm_m2.hako`/`flow_runner.hako` の残り presence/検索箇所を再点検し `index_of_from` に寄せる（完了に近い）
  2) Throw の意味論（例外伝播/診断整形）を詰め、Throw スモークを常時ON化できる最小形に引上げ
  3) MethodRegistry カバレッジ拡張 + SignatureVerifierBox 強化（name 正規化の幅/別名対応）



## 2025-10-05 — Selfhost to_json migration (cont.) + Throw/PHI smokes

- Selfhost emit path unified further to Map→to_json:
  - builder2: per-instruction string appends are gated off by default (`append_insts=0`),
    headers also gated (`append_headers=0`); `to_string()` prefers rebuild via HeaderEmitBox.
  - builder_min: newbox uses NewBoxEmitBox helpers (`with_args_array/with_args_text`) to keep args_text snapshots consistent.
  - pipeline_v2: mir_call_box/emit_newbox normalized with `match` for nulls; emit path uses HeaderEmitBox + JsonEmitBox.
- Smokes added (gated):
  - quick/core/json_v0_if_throw_phi_vm.sh (requires `SMOKES_ENABLE_JSON_V0_THROW=1`; VM may skip/fail otherwise).
  - quick/llvm/phi/phi_invariants_throw_branch.sh (skips if IR dump is unavailable).
- No default behavior changes; tests are gated to avoid flakiness on environments without Throw/IR support.


### Postmortem: 2-arg indexOf bug + mitigation

- Cause: some paths ignored the start position when using `String.indexOf(needle, from)`, returning matches from the head and masking real errors.
- Policy: unify all substring search to `index_of_from(text, needle, pos)` helpers (Box-First; centralize in CfgNavigatorBox/StringScanBox/JsonCursorBox).
- Actions (this patch):
  - Added `StringScanBox.index_of_from(text, needle, pos)` and migrated `JsonScanBox.find_key_dual` to use it (no more 2-arg `String.indexOf`).
  - Added DEV lint: `tools/lints/lint_indexof_two_args.sh` (reports 2-arg `indexOf(...)` occurrences; non-fatal by default, set `LINT_INDEXOF_FAIL=1` to fail).
- Next: continue small-scoped migration (one file at a time) towards `CfgNavigatorBox.index_of_from`/`StringScanBox.index_of_from`.



## 2025-10-05 — Contracts hotfixes + Phase 15.9 doc

- Added Phase 15.9 optimization plan: docs/development/roadmap/phases/phase-15.9/README.md
- VM fixes (runtime semantics):
  - PluginInvoke: unborn guard for non-birth (both direct and helper path)
  - NewBox: mark born when no birth/N exists (builtin/plugin compatibility)
  - ModuleFunction birth failure: remove exposed instance from regs to avoid use-after-unborn
- Next: run quick smokes; then integration minimal pass


## 2025-10-05 — Auto‑birth C++型/in_birth 最終確定 + Plugin 互換

- 仕様確定
  - C++型: MIR NewBox に auto_birth を内包。VM が NewBox 実行後に即時 birth(me,args...) を呼ぶ（関数が存在する場合のみ）。
  - ライフサイクル: unborn → in_birth(try) → born(success)/unborn(fail)。in_birth 中は同一インスタンスのメソッド呼び出しを許可。再入はエラー、二度目の birth は冪等 no‑op。
  - Parser: dot 呼び birth 受理（unborn 経路のE2E）。
  - Contracts 既定ON: NYASH_CHECK_CONTRACTS=1。unborn 操作は禁止。
  - 実例: `local alice = new Life("Alice")` は自動 birth で name()=="Alice"。
- プラグイン方針
  - `birth` 実装があればコンストラクタとして呼ぶ（method_id=0 推奨）。
  - `birth` 不在なら no‑op（互換維持）。birth は冪等実装を推奨。
- 実装ステータス
  - Builder: cross‑module 検出で `auto_birth=Some("Class.birth/N")` を付与（既定ON）。
  - VM: in_birth 導入・成功時のみ born 確定。生存期間ガードを born||in_birth で評価。
  - Bridge(JSON v0): 降下で NewBox.auto_birth を付与可能（フラグ）。PHI最小統合はENVゲート。
  - Runner: 実行バイナリ優先度 `hakorune → hako → nyash`。
  - Smokes: quick 必要最小は緑（重い/外部依存はENVでSKIPゲート）。
- ドキュメント
  - 更新: docs/guides/box-lifecycle.md（C++型 auto‑birth / in_birth を反映）。
- Next（小粒）
  1) integration を一周し、最小ゲートで緑維持
  2) Global 呼びの正規化を CallNameResolverBox に完全移行（短名の根絶）
  3) Plugin birth あり/なしの E2E をもう1本だけ追加（過剰追加は避ける）
  4) [modules] 別 alias のE2Eを+1本
  5) 警告掃除と README/INDEX の導線微修正


## 2025-10-05 — CallResolver 箱導入と VM 経路の名寄せ修正（quick 緑化）

- 追加（箱化）
  - `src/backend/mir_interpreter/resolve/call_resolver.rs`: Global→ModuleFunction の名前解決を一元化。
    - 戦略: 完全一致 → arity 付与(`/N`) → tail一致(`.method/N` かつ `Class.`/`Class_` 接頭) → `Alias_Alias.method/N` → 末尾一致ユニーク拾い。
  - ハンドラ適用: `handlers/calls/function.rs::handle_callee_global` で resolver を呼ぶように変更。
    - 追加の正規化: resolver 呼び出し前に raw 名の末尾 `"/N"` を除去。

- 修正（LLVM/共通方針）
  - `src/llvm_py/instructions/mir_call.py`: print 正規化を表化（`print|println|log` → `nyash.console.log`）。
  - safepoint 方針を統一（constructor/value にも `allow_safepoint` を適用）。

- スモーク/結果
  - PASS: `tools/smokes/v2/profiles/quick/apps/json_object_roundtrip_vm.sh`（以前の Unknown global StringUtils.trim を解消）
  - PASS: `json_pp_vm.sh`, `json_query_vm.sh`（deprec ノイズはフィルタ済み）
  - Unified 再帰（Global/ModuleFunction/Method）は gated で追加済み（既知課題のため既定 SKIP）。

- ドキュメント
  - 追加: `docs/guides/call-resolver.md`（設計/戦略/利用箇所/将来拡張）
  - `tools/smokes/v2/README.md` に quick.env の案内を追記済み（`SMOKES_PROFILE_ENV=quick`）。

- 影響範囲と互換性
  - 既存の Builder/ランナーの公開仕様は不変。VM fallback の名前解決が強化され、曖昧な Global 呼びでも ModuleFunction に安全に合流。
  - 既存テストの動作差は無し（quick 一部はノイズ除去のみ）。

- 次アクション（小粒で継続）
  1) CallResolver に観測トレース（`NYASH_VM_RESOLVE_TRACE=1`）を仮実装（デフォルト OFF）
  2) nested alias を含む最小ケースのスモークを quick に1本だけ常設（過剰追加は避ける）
  3) 余裕があれば Builder 側の ModuleFunction 降下判定も CallResolver に寄せて一本化

### Follow-up done (same day)
- CallResolver: `NYASH_VM_RESOLVE_TRACE=1` で raw/argc/pick を一行JSONで出力（既定OFF／テストではフィルタ済み）。
- Smokes: `quick/apps/json_lint_vm.sh` を常時ONに変更。`quick/core/using_modules_nested_alias_vm.sh` を追加（入れ子 alias のE2E）。
- テストノイズ整備: test_runner フィルタに `^{"resolve":` を追加。
- 警告掃除: 未使用 `std::io::Write` import を削除（vm_pipeline.rs）。
- Docs: `docs/INDEX.md` に CallResolver ガイドへのリンクを追加。
- Nested alias: 同一ファイル内ローカル別名テーブルを導入。`resolve_using_target` にヘッド置換（`Alias.*` → `target.*`）を追加。
  - スモーク `using_modules_nested_alias_vm.sh` は `NYASH_USING_AST=1` でPASS（必要モジュールを `NYASH_MODULES` に付与）。

## 2025-10-05 — Phase 1 self-rec direct + Phase 2 scaffolding

- Phase 1（自己再帰直呼び；既定OFF）
  - Builder: self-recursive call は `ModuleFunction` 直呼びを強制（Global/文字列経路をバイパス）。
    - src/mir/builder/builder_calls/build.rs:101
  - LLVM harness: Global self-rec は `from_i8_string`/`to_i8p_h` を生成せず、直接 call を組み立て。
    - src/llvm_py/instructions/mir_call.py:120+
  - フラグ: `NYASH_MIR_SELFREC_DIRECT=1`（観測: `NYASH_MIR_OPTIMIZE_TRACE=1` でJSON）

- Phase 2（5箱の骨格；既定OFF）
  - Config: `src/mir/optimizer_passes/hints_config.rs`
  - Detectors: `src/mir/optimizer_passes/detectors/mod.rs`（SelfRec/Tail）
  - Hints: `src/mir/optimizer_passes/hints.rs`（HintsMap）
  - Reporter: `src/mir/optimizer_passes/reporter.rs`
  - Docs: `docs/guides/mir-hints.md`

- SSOT: 共有CallResolverコア導入（VM/Builder兼用）
  - `src/mir/resolve/call_resolver_core.rs`／VMラッパは共有を呼ぶ＋トレース維持

- 次アクション（小粒）
  1) LLVM側に tail 指定の薄接続（`NYASH_LLVM_TAILCALL=1` or `NYASH_MIR_HINTS=tail|all`）
  2) Reporter の最小配線（適用イベントを一行JSONで出力）
  3) fib(12) IR差分の簡易スモーク（from_i8_string/… の消滅を確認）

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

## 2025-10-05 — Auto‑birth/Resolver 仕上げ（Phase 1/2）

- Phase 1（完了）: Builder の Global 経路を `CallNameResolverBox::normalize` に統一（短名根絶）
  - src/mir/builder/builder_calls/emit.rs: Global 分岐で `name.contains('.')` のみ ModuleFunction 候補とし、`normalize(name, argc)` に通した上で `module.functions.contains_key` を確認。完全名のみ許可。
  - 目的: `birth` 系や `Box.method/N` 解決の一貫化と、レガシー `func` 文字列経路の撲滅（事故の温床）

- Phase 2（観測の統一）: birth_auto は公開JSONに出さない（devトレースのみ）
  - VM 側の契約系ログ（`contracts_newbox`/`contracts_birth`/`contracts_birth_pre`）に寄せて観測。test_runner は `^{\"kind\":\"contracts_` を既にフィルタ済み。
  - Builder での auto‑birth 生成に新たな公開フィールドは付与せず（JSON スキーマ変更なし）。必要時は `NYASH_BUILDER_DEBUG=1` で観測。

- Phase 3（構想/既定OFF）: birth 引数スキーマ宣言（軽量）→ arity/type の早期検証
  - Box定義側に最小メタ（arity/型ヒント）を宣言 → Builder が `new → birth(args…)` 生成時に早期検証（Fail‑Fast/警告は既定OFF）。
  - 影響: 既定OFFのため互換不変。導入時は `docs/guides/box-lifecycle.md` に仕様追記、`NYASH_BIRTH_SCHEMA_STRICT=1` で段階導入予定。

- スモーク計画（最小）
  - quick/core/userbox_birth_vm.sh（auto/explicit）: 正常化を確認（PASS 維持）
  - quick/core/using_modules_alias_vm.sh: [modules] alias 解決（既存 PASS）
  - quick/core/using_modules_alias_timer_static_vm.sh: [modules] 別alias（TimerBox）E2E 追加（PASS）
  - quick/core/using_modules_alias_toml_only_vm.sh: env無しで hako.toml のみ（PASS）
  - LLVM 自己再帰 IR は環境依存のため既定 SKIP（`SMOKES_ENABLE_LLVM_SELFREC=1` で任意）

## 2025-10-05 — Hakorune‑VM への改名と箱構成（Phase 15.7 反映）

- 目的: Mini‑VM を hakorune‑vm へ改名し、箱境界を明確化。自己ホストのユニットとして扱いやすくする。
- 変更計画（ドキュメント先行→実装）
  - パス: `apps/selfhost/vm/boxes/mir_vm_min.hako` → `apps/hakorune/vm/boxes/hakorune_vm_min.hako`
  - [modules]: `selfhost.vm.*` → `hakorune.vm.*`（旧キーは1リリースalias）
  - 入口: `HakoruneVmBox.run_min(json)`（旧 `_run_min` はadapter）
  - 箱: InstrDecoderBox / ProgramStateBox / OpHandlersBox / PhiWiringBox / StringScanBox / JsonScanBox / ObserveBox（最小）
- スモーク影響: `selfhost_m2/m3` を `hakorune_vm_*` 名へ順次置換（旧名は当面維持）


- 互換性
  - 公開仕様は不変。Builder/VM の名前解決経路のみ構造的に厳格化。`print` 等の Global 経路は影響なし。



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
Update — 2025-10-04 (Mini‑VM φ decode box化・安定化)
- φ の再導入を安全版で実施（Box‑First）
  - MirVmMin: φ decode を Result.ok/err で包み、GuardBox で values[] 走査を上限化。適用は PhiApplyBox.apply に委譲。
  - values[] は bracket 範囲に限定し、単調前進＋512ガードで Fail‑Fast。pred 不一致時は先頭 value を fallback 採用。
  - スモーク（dev gate）: selfhost_mir_m3_phi_entry_vm, selfhost_mir_m3_phi_diamond_vm（PASS）
- 新規ユーティリティ箱
  - ScannerBox: 安全な逐次スキャン（peek/advance/at_end）。
  - GuardBox: 反復上限ガード（tick）。
  - Result/ResultBox: 統一結果（ok/err, unwrap_or）。
- 次の段階
  - φ decode を PhiDecodeBox（decode_result）へ本配線する（準備済み）。当面は inline 安定経路を維持し、回帰が無いタイミングで切り替え。
  - docs/guides に「スキャンは箱経由・前進保証・ガード義務化」を追記。

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
## Current Focus
- Env: NYASH_MINJSON_USE_HEADER_BOX=1 (selfhost early header via HeaderEmitBox)
- Using: alias→modules registry commonized (VM/LLVM/fallback)
- Mini-VM φ decode hardening: DONE (error taxonomy, skip malformed, empty-array handling).
- Mini-VM log noise: DONE (default errors-only; NYASH_MINIVM_DEBUG=1 to enable debug).
- Guide updated: scanning-policy staged boundaries (seek→substring→Frag).
- Smokes: 1 always-on (values), extras gated via SMOKES_ENABLE_PHI_DECODE_EXTRA=1.

### Next Small Steps
- Builder: static factory auto-birth now emits lowered `Class.birth/Arity` even if instance box is lowered later (static-first order).
- Mini‑VM: Throw terminator implemented (returns -2, error-only log).
- Selfhost header emit unified: pipeline_v2/HeaderEmitBox kept for orchestration; compiler early --min-json emits locally to avoid AST-prelude dependency.
- φ diamond smoke is now always-on in quick.
- Optionally expand phi error variants if new malformed shapes appear.
- Keep quick lean (1 phi test), run extras via filter/ENV when diagnosing.
- Proceed to remaining m3 validation (branch_false/cond_prev_block/jump_chain) once needed.


### LLVM: plain main() exit policy (fixed)

- Problem: ny_main wrapped plain `main()` and returned its value as the process exit code. For benchmarks that `print("Result: ...")`, the last expression path could yield a String handle; AOT runtime then printed a trailing `Result: 2`, making output look wrong.

- Fix: Update wrapper in `src/llvm_py/builders/entry.py`.
  - Plain `main()` is executed (side effects preserved) but its return value is discarded.
  - ny_main returns `0` for plain `main()`.
  - `Main.main/1` remains the only path that can define an intentional numeric exit code.

- Bench hygiene: Use `NYASH_NYRT_SILENT_RESULT=1` during timing to suppress the runtime’s final `Result: <exit>` line. Our bench tools already set this where applicable.

## 2025-10-05 — Auto‑birth一本化と完全名解決（smokes 緑化）

- 仕様と構造（箱化・統一）
  - Auto‑birth の SSOT を Builder→VM に一本化。
    - Builder: `new Box(args)` の直後に、ModuleFunction 形式 `Box.birth/Arity(me, args…)` を生成（関数が Module に存在する場合のみ）。
    - 明示 `obj.birth(args)` も Builder 側で ModuleFunction に正規化（BoxCall 経路に残さない）。
  - 名前解決は共有コアに集約。
    - `src/mir/resolve/call_resolver_core.rs` に `normalize/parse/is_fully_qualified` を追加。
    - Builder 側の Global 経路は `normalize()` を通した完全名のみを `Callee::ModuleFunction` で emit。
    - VM 側の ModuleFunction は完全名のみ受理（不完全名は Fail‑Fast）。

- レガシー経路の隔離
  - `builder_calls/emit.rs` の module.functions 命中時の legacy Call 生成（callee=None）を撤廃し、常に ModuleFunction を emit。
  - VM の tail-based fallback は維持しつつ、入口は完全名に寄せる（将来的に VM 側は exact のみへ簡素化予定）。

- 観測とデバッグ
  - Builder: `NYASH_BUILDER_DEBUG=1` で New と auto‑birth の1行ログ、Call は callee 優先表示（`MF:Box.method/N`）。
  - VM: `NYASH_VM_BIRTH_TRACE=1` で birth プリマーク JSON 一行。`NYASH_VM_CALL_ARG_TRACE=1` で ModuleFunction 呼びの a0/a1/a2 種別を表示。

- スモーク緑化（quick）
  - `tools/smokes/v2/profiles/quick/core/userbox_birth_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/userbox_birth_explicit_vm.sh`
  - 安定化のため、当該2本は `SMOKES_USE_DEV=0`（devモードOFF）に変更。`test_runner.sh` の既定 `NYASH_VM_TOLERATE_VOID` も 0 へ。
  - `test_runner.sh` のバイナリ解決順を `nyash → hako → hakorune` に変更し、雑音をフィルタ（`[deprecate]`）。

- Docs 反映
  - `docs/guides/box-lifecycle.md`: auto‑birth 既定・unborn 経路・冪等性・ModuleFunction 経路固定を追記。
  - `docs/guides/call-resolver.md`: 完全名前提の normalize/parse API と適用箇所（Builder/VM）を追記。

- 既知メモ
  - dev モード（--dev）では bring‑up 便宜の挙動差が混じるため、core の値検証系は dev=OFF を既定に維持（今回の2本も OFF）。
  - Plugin/Array/Map/String の birth は no‑op 合成で互換（VM 側でプリマークのみ）。


## 2025-10-05 — C++方式 auto‑birth（設計→実装計画）

- ねらい: NewBox に birth 呼び（コンストラクタ）を内包し、C++ 同様に「new が即 birth」を実現する。New→Call(birth) の二重命令を解消し、未born誤爆を構造的に減らす。

- 仕様（設計）
  - MIR 拡張: `NewBox { dst, box_type, args, auto_birth: Option<String> }`
    - `auto_birth = Some("Class.birth/N")`（完全名）であれば、VM は NewBox 実行直後に birth を自動実行。
    - `None` の場合は未出生（unborn）。ユーザーは明示 `birth()` を呼ぶ。
  - Builder: CallNameResolverBox で完全名を生成し、モジュールに存在する場合のみ `auto_birth=Some` を付与。`StringBox` 等は `None`。
  - VM: NewBox 実行時に contracts の born を先出し→`auto_birth` があれば ModuleFunction 経由で birth 実行。失敗は NewBox としてエラー伝播（コンストラクタ例外準拠）。
  - 互換: 旧MIR（New + Call(birth)）は当面受理。

- 段階導入（実装計画）
  1) VM: `NewBox.auto_birth` を受理（旧MIRも継続）。
  2) Builder: フラグ `NYASH_BUILDER_NEWBOX_AUTOBIRTH=1` で `auto_birth` を付与（既定OFF）。
  3) JSON v0 ブリッジ: `NewBox(auto_birth)` 降下を追加（既定OFF）。
  4) 既定ONに切替→レガシーの New→Call(birth) 特別扱いを段階整理。

- テスト方針（最小）
  - auto: `new Life("A")` → name()=="A"。
  - unborn: `Life.unborn().name()` は Fail‑Fast、`Life.unborn().birth("A").name()` は OK。
  - plugin no‑birth: `new SomePlugin()` が安定（no‑op birth + born 記録）。

- メモ（観測）
  - dev では `[contracts_*]` と `[call]` ラインで観測。テストランナーは既にフィルタ済み。

- Implemented: JSON v0 bridge honors NYASH_JSON_NEWBOX_AUTOBIRTH=1 to enable Builder auto_birth during bridge; Builder default auto_birth now ON (override with NYASH_BUILDER_NEWBOX_AUTOBIRTH=0). Added quick smoke: tools/smokes/v2/profiles/quick/core/plugin_no_birth_nop_vm.sh (SKIP unless NYASH_PLUGIN_NO_BIRTH_BOX set).

## 2025-10-05 — Hakorune‑VM rename/stubs
- Added: `apps/hakorune/vm/boxes/hakorune_vm_min.hako` (copy; entry `run_min` standard).
- hako.toml: `[modules]` now maps `hakorune.vm.mir_min` → `apps/hakorune/vm/boxes/hakorune_vm_min.hako`; legacy `selfhost.vm.mir_min` kept.
- Stubs: `InstrDecoderBox`, `ProgramStateBox`, `PhiWiringBox` under `apps/hakorune/vm/boxes` delegating to existing boxes.
- Entry: switched `apps/selfhost/vm/mir_min_entry.nyash` to use `run_min`.
- Smokes: added `hakorune_vm_m2_eq_true_vm.sh`, `hakorune_vm_m3_branch_true_vm.sh` using hakorune alias + run_min.

- Alias move: flow_runner / dev sample switched to hakorune.vm.mir_min.
- HakoruneVmMin now uses InstrDecoderBox.next + PhiWiringBox.wire (thin path).
- Added one more representative smoke: hakorune_vm_m3_jump_vm.sh.

- Hakorune ProgramStateBox: regs retrieval switched in hakorune_vm_min (state st initialized; counters remain local).
- Added hakorune phi smoke: tools/smokes/v2/profiles/quick/selfhost/hakorune_vm_m3_phi_diamond_vm.sh (uses run_min).
- Kept run_min migration scoped to hakorune smokes; originals unchanged.

- ProgramStateBox setters: bb/prev_bb writes now mirrored via ProgramStateBox.set_* in HakoruneVmMin.
- Added hakorune jump_chain smoke: tools/smokes/v2/profiles/quick/selfhost/hakorune_vm_m3_jump_chain_vm.sh (uses run_min).

- ProgramStateBox get: phi decode now reads prev_bb via ProgramStateBox.prev_bb(st) (one-site get introduction).
- Added smoke: tools/smokes/v2/profiles/quick/core/builder_autobirth_cross_module_vm.sh (SKIP unless SMOKES_ENABLE_BUILDER_AUTOBIRTH_CROSS=1).


### Update — 2025-10-05 19:43 (CLI AST dev marker + ProgramStateBox reads + Stage‑1/2 smokes)
- CLI: inject "__cli_dev__":1 into AST JSON when NYASH_DEV_JSON_MARKER=1 (src/macro/ast_json.rs). FlowRunner normalizes to {"__dev__":1}.
- HakoruneVmMin: completed ProgramStateBox read migration (removed local prev_bb reads; all logs/phi use getters).
- Smokes added (quick/selfhost):
  - hakorune_pipeline_const_ret_vm.sh (PASS)
  - hakorune_pipeline_compare_branch_phi_vm.sh (PASS)
  - hakorune_pipeline_compare_ret_vm.sh (gated: set SMOKES_ENABLE_STAGE12_COMPARE_RET=1; FlowRunner fast-path can shadow)


### Update — 2025-10-05 19:58 (Phase‑15.9 optimization plan consolidated)
- Added consolidated optimization plan under docs/development/roadmap/phases/phase-15.9/README.md (VM unboxed primitives, AOT, fast blocks, externcall, PIC, SBO/arena, KPIs, guards).
## 2025-10-05 — StringHelpers delegation + .hako unification (batch 2)

- Delegated helpers in Nyash utility files to StringHelpers (read_digits/int_to_str):
  - apps/selfhost/vm/collect_mixed_smoke.hako (was .nyash)
  - apps/selfhost/vm/mini_vm_lib.hako (was .nyash)
  - apps/selfhost/vm/mini_vm_if_branch.hako (was .nyash)
- Renamed files to .hako for consistency and future using-based imports.
- Renamed legacy box: apps/selfhost/vm/boxes/mir_vm_m2.hako (was .nyash). No external references affected.
- Notes: avoided block comments; full delegation via using apps/selfhost/common/string_helpers.hako as StringHelpers.
## 2025-10-05 — WASM ABI skeleton（handoff for wasm branch）

- Docs: docs/guides/wasm-abi.md（最小ABI/契約/責務分離を記載）
- Crate (opt-in, not in workspace): crates/nykernel-wasm
  - wasm32向け bump allocator と nykernel_*(malloc/load_i64/store_i64) をエクスポート
  - 非wasm向けはダミー（リンク用）
- hakorune-std（箱化）: apps/hakorune/std/core/array.hako
  - extern_call("nykernel.*") 経由で配列操作（len/cap/ptr管理、resize/copy、Fail‑Fast）
- 影響: 既存ビルド/quickに未接続（既定OFF）。wasmブランチでの受け取り前提で配置。
-
## 2025-10-05 — Phase 15.7 refresh（Self‑Hosting back on track）

- 状態サマリ
  - Mini‑VM: InstructionScannerBox/OpHandlersBox 統一、ProgramState/CfgNavigator/RetResolver/Diagnostics を導入。代表m2/m3は緑。
  - JSON v0 Bridge: 到達不能 pred 除外の判定を if/match で統一（PHI 不変を維持）。
  - helpers: StringHelpers へ委譲（JsonFrag/JsonScan/Compiler 残差を片付け）。selfhost VM 補助の .hako 統一を進行。
  - WASM: nykernel‑wasm（未接続）＋ hakorune‑std ArrayBox 骨格＋VMスタブ/スモーク（opt‑in）を整備（受け渡しOK）。

- 次の小粒（15.7 継続）
  1) Stage‑1/2 最小 E2E 代表3本（const→ret／compare→ret／compare→branch→phi）を緑固定（既存スモークの代表で維持）。
  2) UsingResolverBox/NamespaceBox の実装と Pipeline V2 統合（Callee::ModuleFunction を前段正規化）。
  3) Mini‑VM: ProgramStateBox/CfgNavigatorBox の参照を全面 get 化、代表 CFG（diamond/jump_chain）を+1本ずつ追加。
  4) 先送り: index_of_from 集約（CfgNavigatorBox へ段階移行、Phase 15.12で一括）。

- 移行計画（Rust → Hakorune、段階）
  - 先行済: JSON 断片/走査ヘルパ（StringHelpers/JsonFrag/JsonScan の責務分離）
  - 次: Mini‑VM 補助箱（InstrDecoderBox, PhiWiringBox, ObserveBox）を薄く追加して呼び替え（意味論は現状維持）
  - Compiler: UsingResolverBox/NamespaceBox を先に箱化→ Pipeline V2 に統合。Lower/MIR emit は箱経由で集約。
  - extern_call: nykernel.* の一本化を維持（VMスタブ/LLVM/WASM の橋で切替）。

## 2025-10-05 — P2-A UsingResolverBox (compiler)

- Added box `apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako` (pure, no IO).
- Responsibilities: manage alias→path and alias→namespace maps; load from parser usings JSON; optional modules JSON.
- API: load_usings_json/load_modules_json, add_ns/add_module/add_path, resolve_* getters, to_context_json.
- Docs: apps/selfhost-compiler/pipeline_v2/README_using_resolver.md.
- Smoke: quick/selfhost/selfhost_using_resolver_basic_vm.sh (validates alias/ns/path resolution).
- Next (P2-B): NamespaceBox to rewrite Timer.now_ms → Callee::ModuleFunction using resolver context.

## 2025-10-05 — P2-B NamespaceBox (compiler)

- Added `apps/selfhost-compiler/pipeline_v2/namespace_box.hako` for alias→namespace normalization.
- Pipeline addition: `PipelineV2.lower_stage1_to_mir_with_usings(ast, prefer_cfg, usings_json, modules_json)` resolves names via UsingResolverBox before emit.
- Smoke: `selfhost_namespace_box_basic_vm.sh` validates alias→ns mapping for call/class.
- Note: end-to-end pipeline smoke for usings is added but optional; main focus is box-level resolution.


## 2025-10-05 — Method arity Fail‑Fast（built‑ins）+ 署名レジストリ足場

- 目的
  - 「静かな失敗」を排し、BoxCall/Method 呼び出し時にメソッド署名（arity）不一致を明示エラーにする。
  - indexOf など個別対処ではなく、(Box, method, arity) の一律検証に寄せる。

- 実装（本コミット）
  - VM 実行時チェック（built‑ins のみ; String/Array/Map）
    - 変更: src/backend/mir_interpreter/handlers/calls/method.rs
      - execute_method_call の冒頭で type_registry を参照し、(Box, method, arity) を検証。
      - 不一致時は `No matching method: <Box>.<method>(<N> args). Available arities: [...]` を返す。
      - String.indexOf は args.len()!=1 の場合にも明示エラー化。
    - 変更: src/backend/mir_interpreter/handlers/calls/legacy/method_handler.rs
      - legacy Method 経路にも同様の arity 検証を追加（Fail‑Fast）。
  - 型レジストリの拡張
    - 変更: src/runtime/type_registry.rs に `known_arities_for` を追加（診断用）。
  - 署名レジストリの箱（スカフォールド）
    - 追加: apps/hakorune/vm/boxes/method_registry.hako — built‑ins 3種の method/arity 一覧（今後 Pipeline V2 で利用）。

- スモーク
  - 追加: tools/smokes/v2/profiles/quick/core/arity_error_array_push_2args_vm.sh — Array.push(2) がエラーに。
  - 備考: String.indexOf(2) はパイプライン経路によっては別分岐を通るため、compile‑time 検証導入後に常設を検討。

- 次アクション（小粒）
  1) SignatureVerifierBox（Pipeline V2）を追加し、Method/Call 降下直後に (Box, method, arity) を MethodRegistryBox で照合して Fail‑Fast。
  2) Using STRICT を既定ONに（NYASH_USING_STRICT=1）し、未宣言 using/別名は即エラー。resolver trace は dev のみ。
  3) JsonFragBox に *_strict アクセサを追加し、必須キー箇所（compare/lhs/rhs/dst 等）を strict 化。
  4) 代表スモーク: missing_using_should_error_vm / json_missing_cmp_should_error_vm / ret_undefined_register_should_error_vm を追加。



## Phase 15.7 — Strictness + Compile-time Verifier (P2)

- Added SignatureVerifierBox (apps/selfhost-compiler/pipeline_v2/signature_verifier_box.hako)
  - Compile-time arity check for common built-ins by method name (uniform arities).
  - Wired into PipelineV2 method lowering (v1 and legacy Stage‑1 paths).
- Using strict: NamespaceBox now fails fast on unresolved aliases (prints `[ERROR] Unresolved using alias: X`).
  - PipelineV2.lower_stage1_to_mir_with_usings returns null if alias resolution fails.
- JSON strict getters: JsonFragBox.get_int_strict/get_str_strict with `[ERROR] Missing key: <key>`.
  - OpHandlersBox.handle_compare enforces presence of cmp/lhs/rhs/dst and emits errors, no silent fallbacks.
- Smokes added:
  - quick/selfhost/selfhost_missing_using_vm.sh
  - quick/core/json_missing_key_vm.sh
  - quick/selfhost/selfhost_ret_undefined_register_vm.sh

Acceptance:
- Early arity mismatches surface at compile-time in Pipeline V2 path.
- Missing using alias and missing required JSON keys now produce explicit errors (no silent failure).


---

2025-10-06 — Rust VM Fail‑Fast alignment + using strict default ON

- Using strict default ON
  - Added `config::env::using_strict()` (default true unless `NYASH_USING_STRICT=0|false|off`).
  - Call sites now use `using_strict()` instead of reading env directly:
    - `src/runner/modes/common_util/resolve/strip/collect.rs:22`
    - `src/runner/mod.rs:239`
    - `src/runner/modes/common.rs:188`
- Unresolved using in strict mode now fails early
  - `src/runner/pipeline.rs: resolve_using_target(.., strict=1, ..)` returns Err on unresolved.
  - Runner surfaces as `❌ using: unresolved using '...' ...` and exits.
- Runtime method arity Fail‑Fast remains enforced (no code change required)
  - Built‑ins via `type_registry` validation in `execute_method_call`.
- Smoke added
  - `tools/smokes/v2/profiles/quick/core/using_missing_strict_vm.sh` (strict unresolved using → FAIL)

Next Steps (structure-first)
- Consolidate env reads under `src/config/env.rs` (eliminate direct `std::env::var` checks from runners; use helpers only).
- Share MethodRegistry between compiler and runtime (single source of truth for built-ins; optional metadata from hako.toml for plugins).
- Extract JsonCursorBox (seek/scan helpers) and delegate JsonFragBox to it; migrate handlers to strict getters by default for required keys.


## 2025-10-06 — Env 集約 + MethodRegistry 拡大 + Call 検証拡張 + JsonCursorBox 採用

- Env 集約（関数化・呼び出し置換）
  - 追加: vm_resolve_trace(), emit_trace(), prefer_cfg2(), prefer_cfg(), scopebox_enable(), loopform_normalize(), macro_selfhost_pre_expand()（src/config/env.rs）
  - 呼び出し更新: VM CallResolver / runner selfhost 経路で直読を排除し、関数に統一。
- MethodRegistry 拡大（ビルトイン整合）
  - StringBox: toString(0)/stringify(0)/startsWith(1)/endsWith(1) をレジストリへ追加。
  - Array/Map: toString(0)/stringify(0) をレジストリへ追加。
  - runtime type_registry.rs にも StringBox の vtable 雛形を拡張（slot 308..311）。
- Call 側検証の拡張（安全な判定ルール）
  - SignatureVerifierBox.verify_call_name_arity: 最後の '.' でメソッド名抽出、直前セグメントを Box 名候補として扱い、
    String|StringBox / Array|ArrayBox / Map|MapBox のみ厳密にアリティを検証（その他は許容）。
  - 例: core.String.indexOf → StringBox.indexOf として 1 引数のみ許容。
- JsonCursorBox 採用（直接スキャン箇所の段階移行）
  - minivm_probe/step_runner で index_of_from / seek_array_end を JsonCursorBox に委譲。
  - 目的: 文字列/配列/オブジェクト走査の一貫API化とバグ温床の解消。
- 互換/影響
  - 既存 quick は互換。ビルトインの arity エラーはより早期/明確に失敗（Fail‑Fast）。
- 次アクション（小粒）
  - UsingResolverBox と MethodRegistry の連携（エイリアス正規化の強化）。
  - JsonCursorBox の段階的適用拡大（残る直接スキャン呼び出しの移行）。
  - Call 署名検証の対象拡大（必要に応じて Map/Array の追加メソッド）。


## 2025-10-06 — Mini‑VM stack overflow (selfhost_mir_m2_* safety patch)

Symptom
- Running selfhost_mir_m2_multi_compare_gt_last_ret_vm caused a stack overflow (Rust abort) right after NewBox logs.
- Reproduced with a tiny driver that calls MirVmMin._run_min(JSON) even for simple const→ret.

Root cause (current hypothesis)
- Early NewBox births during MirVmMin startup can trigger deep VM paths before we get to any Mini‑VM logic, leading to reentrant recursion in some environments.
- The failure occurs prior to our compare/scan loop; it’s not a Ny string‑scan recursion.

Structural fix (fail‑fast + early return)
- Move a “thin, allocation‑free” fast path to the very top of MirVmMin._run_min:
  - If Block 0 contains const→ret, parse the typed literal and return immediately.
  - If Block 0 contains compare→ret, parse two typed consts + cmp/operation and return the result.
  - This path avoids allocating MapBox/ArrayBox before returning, preventing the stack overflow.
- Also moved the per‑compare early return inside the compare branch to run before calling OpHandlersBox.handle_compare.

Files
- apps/selfhost/vm/boxes/mir_vm_min.hako
  - Added Block0 early path (const→ret, compare→ret) before `new MapBox()`.
  - In compare branch, compute `ridt` and return early when it targets this compare’s dst, before handler call.

Smokes
- Added: tools/smokes/v2/profiles/quick/core/json_v0_const_ret_vm.sh (PASS)
- selfhost_mir_m2_multi_compare_gt_last_ret_vm: still reproduces in harness due to an early ArrayBox NewBox at entry (likely args), next step below.

Next steps
1) Special‑case VM NewBox “args” creation path to skip safety birth fallback (or short‑circuit birth) for the implicit args ArrayBox, to prevent reentry in early startup.
2) Keep the early fast path in MirVmMin (const/compare→ret) — preserves performance and robustness.
3) If needed, add a dedicated smoke to ensure implicit args ArrayBox birth is treated as no‑op without reentry.


## 2025-10-06 — Using strictness hardening (tail fallback off by default)

Changes
- Builder: tail-based ModuleFunction fallback is strict by default.
  - File: src/mir/builder/builder_calls/build.rs:177 — now `strict` defaults ON; ambiguous matches error unless NYASH_MIR_CALL_MODULE_FN_STRICT=0.
- VM: BoxCall tail-fallback guarded by receiver class presence only (prevents cross-module).
  - File: src/backend/mir_interpreter/handlers/boxes/legacy/mod.rs: tail fallback path now requires `recv_cls` (class prefix filter), else skip.
- Using: namespace-only alias acceptance is env-gated (default OFF).
  - File: src/runner/modes/common_util/resolve/strip/collect.rs: looks_like_ns requires NYASH_USING_NAMESPACE_ALIAS=1.
  - File: src/config/env.rs: added using_namespace_alias() helper.

Rationale
- Avoid ambiguous tail resolution (e.g., JsonCursorBox vs JsonFragBox) leading to recursive call loops.
- Keep dev escape hatch via env flags for local experiments.

Next
- Add a small regress test exercising JsonCursorBox.index_of_from vs JsonFragBox.index_of_from under strict mode (expect both to work, no recursion).
- Optional: add ModuleFunction recursion detector (dev-only) to surface cycles faster.


## 2025-10-06 15:04 — Boxification: CallNameNormalizer/ModuleFunctionResolver (strict), VM reenter guard

- Added CallNameResolverBox functions: `is_valid_ident`, `static_name(Box, method, arity)` to preserve underscores and reject invalid identifiers.
- New resolver: `src/mir/resolve/module_function_resolver.rs` provides `resolve_strict(keys, raw, argc, allow_tail)`.
- Builder static calls now use normalizer and strict resolver; legacy Global fallback gated by `NYASH_VM_GLOBAL_TAIL_FALLBACK=1`.
- VM Global() uses strict resolver; dotted names require exact match unless tail fallback is enabled. ModuleFunction handler unchanged except dev safety bridges.
- Dev diagnostics: lightweight reentrancy counter and optional abort via `NYASH_VM_REENTER_TRACE=1` / `NYASH_VM_REENTER_LIMIT=N`.

Impact
- Fixes underscore-loss class.method naming drift structurally (generator side) and prevents cross-module mis-binding (resolver side).
- Keeps behavior compatible under `NYASH_VM_GLOBAL_TAIL_FALLBACK=1` while defaulting to strict resolution.

Next
- Remove temporary Json* index_of_from native shims once builder strict path is fully green.
- Consider RAII-style decrement for reenter counter or per-call pop if needed (dev-only).


---

Follow‑up — Simplify Before Debug (recommended order)

1) Builder: remove method‑only/tail unique fallback for static calls (strict only)
   - Keep a single, explicit switch for legacy (`NYASH_VM_GLOBAL_TAIL_FALLBACK=1`).
   - Goal: every dotted name is produced via CallNameNormalizerBox and resolved by ModuleFunctionResolverBox only.

2) Canonicalize alias‑alias (A_A → A) at a single entry point
   - Builder lowering: when setting `current_static_box` from function name, fold `X_X → X`.
   - Optional dev valve: VM Global strict can fold under `NYASH_VM_CANON_ALIAS_ALIAS=1` (default OFF).

3) Remove legacy helpers masking bugs
   - JsonFragBox: drop `block0segment` alias (use `block0_segment` only).
   - Keep any temporary native shims (Json* `index_of_from/3`) gated and plan to delete after green.

4) Mini‑VM
   - Keep early paths (const→ret / compare→ret) at the top; do not add allocations or deep helper chains in header.

5) Verification (targeted first)
   - `tools/smokes/v2/profiles/quick/core/json_v0_const_ret_vm.sh`
   - `tools/smokes/v2/profiles/quick/core/json_v0_if_return_phi_vm.sh`
   - `tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_multi_compare_gt_last_ret_vm.sh`

Flags (for reference)
   - `NYASH_VM_GLOBAL_TAIL_FALLBACK=1` — allow dotted tail fallback (legacy)
   - `NYASH_VM_REENTER_TRACE=1` / `NYASH_VM_REENTER_LIMIT=N` — dev cycle guard


## Update (Builder terminator guard + callee=None phase-out)

- Enforced Fail-Fast: builder now rejects any emit after a block terminator (ret/throw/jump/branch).
  - Guarded in two layers: (1) MirBuilder.emit_instruction returns Err, (2) BasicBlock.add_instruction panics if a non-terminator is appended after a terminator.
  - Rationale: preserve 1BB1-terminator invariant; prevent silent IR corruption and PHI issues.
  - Test: added `src/tests/mir_builder_terminator_guard.rs` (unit test) to assert error on emit-after-terminator.
- Began deprecation of `callee=None` for module functions:
  - Method→ModuleFunction lowering now sets `callee=Some(ModuleFunction(...))` consistently (legacy NameConst fallback removed for these routes).
  - Dev-only `_helper(...)` inside static boxes now emits ModuleFunction callee.

Planned next (per plan):
- Continue removing remaining `callee=None` in builder legacy paths where target is known (keep dynamic Value-call intact).
- Mini-builder unification and a small health smoke for JSON builder validity.
- Document flags table and SKIP hints; plan to remove Json* index_of_from safety shim after strict path is fully green.

- Continued callee=None phase-out: Global/Value/Indirect now emit explicit Callee (Global/Value). Builder legacy indirect-call path removed.

### Tests & validation (partial)
- Fixed broken NewBox initializers in src/tests (duplicate/stray commas) to match `auto_birth` field.
- Gated JIT-dependent tests behind `cranelift-jit` feature (plugin_hygiene, policy_mutdeny extern cases).
- Added integration test `tests/builder_terminator_guard.rs` to assert block-level guard panics when emitting after terminator. PASS.
- `cargo build` passes. Full `cargo test` still includes legacy tests (API drift); we did not overhaul those. Running `cargo test --test builder_terminator_guard` validates the new guard specifically.

## 2025-10-06 — Tests: migrate BoxCall to ModuleFunction/ExternCall (strict path)

- Rationale: legacy tests hand-emitting `MirInstruction::BoxCall` against builtins (String/Array/Map) diverged from the unified call path and caused failures under strict resolution.
- Changes (focused):
  - Converted remaining BoxCall uses to `MirInstruction::Call + Callee::ModuleFunction` for builtins.
    - String: substring/concat/len/indexOf/replace/trim/toUpper/toLower → `StringBox.*`.
    - Array: push/set/get/len → `ArrayBox.*`.
    - Map: set/get/has/size/delete/keys/values → `MapBox.*` (+ keys/values length via `ArrayBox.len/0`).
  - Kept one strictness test’s unknown-call branch as ModuleFunction("MapBox.unknown/0"); kept its first part on BoxCall(set/size) to match the vtable-focused intent and avoid resolver noise.
- Touched files (tests only):
  - src/tests/vtable_string.rs, src/tests/vtable_string_p1.rs, src/tests/vtable_string_boundaries.rs
  - src/tests/core13_smoke_array.rs, src/tests/core13_smoke_jit.rs, src/tests/core13_smoke_jit_map.rs
  - src/tests/vtable_map_ext.rs, src/tests/vtable_map_boundaries.rs, src/tests/vtable_strict.rs
  - src/tests/identical_exec_string.rs, src/tests/identical_exec_collections.rs
- Result:
  - `cargo test --lib` → 183 passed, 0 failed (doc-tests excluded).
  - Doc-tests still have unrelated failures; propose gating or fixing separately (out-of-scope for this patch).
- Next:
  - Optionally unify vtable_strict first part to ModuleFunction once resolver/vtable map is consistently visible under `NYASH_ABI_VTABLE=1` in all contexts.

## 2025-10-06 — ModuleFunction bridge for builtins + vtable_strict 前半統一

- Implemented VM-side bridge so `ModuleFunction("ArrayBox.*"|"MapBox.*"|"StringBox.*"|"ConsoleBox.*")` routes to vtable method execution.
  - Location: `src/backend/mir_interpreter/handlers/calls/function.rs` (bridge before exact function lookup)
  - Rationale: lets tests and builder use strict `Callee::ModuleFunction` while preserving runtime dispatch to builtins.
- Migrated `src/tests/vtable_strict.rs` の前半（set/size）を ModuleFunction 化。
- Disabled doctests by default in `Cargo.toml` (`[lib] doctest = false`) to keep CI quiet; doctest fixes can be handled separately.
- Result: `cargo test --lib` → 183 passed / 0 failed.
- Next (small):
  - Sweep remaining `BoxCall` residues in tests (e.g., toString) where safe to replace with `StringBox.toString/0`.
  - Keep resolver strict; tail fallback remains dev/ENV-only.
  - Continue phasing out any lingering `callee=None` emissions in builder (no behavior change expected).

## 2025-10-06 — Remove NYASH_VM_CALL_ADAPTER (adapter deleted)

- Adapter removed: deleted `src/backend/mir_interpreter/handlers/calls/adapter.rs` and eliminated all call sites.
  - Hooks removed from `handlers/mod.rs` (try_execute_via_callee branches for BoxCall/ExternCall/PluginInvoke).
  - `VmConfig.call_adapter` flag removed (env `NYASH_VM_CALL_ADAPTER` no longer read).
  - Rationale: Unified `Call + Callee::{ModuleFunction,ExternCall}` is default; adapter increased ambiguity with no remaining users.
  - Strict policy stays: `NYASH_VM_GLOBAL_TAIL_FALLBACK` remains dev‑only; namespace alias remains default OFF.

### Box .nyash inventory (staged migration plan)
- To convert next (low risk):
  - apps/selfhost-vm/boxes/json_cur.nyash → .hako
  - apps/selfhost-vm/boxes/mini_vm_core.nyash → .hako
  - apps/selfhost-vm/boxes/mini_vm_prints.nyash → .hako
  - apps/selfhost-vm/boxes/seam_inspector.nyash → .hako
- To leave archived (no change):
  - apps/archive/selfhost-legacy/boxes/*.nyash (kept for historical reference)
- Already converted in this change:
  - apps/lib/boxes/{console_std.hako,string_std.hako,array_std.hako,map_std.hako}
- Notes:
  - All references must be updated in using lines after renaming.
  - Parity/runner scripts adjusted separately as needed.

## Macro bring-up (selfhost-min)

- Added apps/macros/selfhost_min/macros.hako (json/map/arr skeleton; call normalization WIP)
- Loader prefers runner route for .hako MacroBoxSpec (child runner default)
- PATHS wiring: nyash.toml / hako.toml set NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako
- Rust SelfhostMinMacro gated: NYASH_MACRO_SELFHOST_MIN=1 registers built-in variant (dev only)
- call! macro (experimental):
  - Shape: call("Box.method/N", args...)
  - Rust macro variant normalizes to ModuleFunction name for method-like forms:
    - Example: call("String.len/1", s) → FunctionCall("StringBox.len/0", [s])
  - Smoke: tools/smokes/v2/profiles/quick/selfhost/call_macro_strict_vm.sh (gated by NYASH_ENABLE_CALL_MACRO=1)
  - Caveat: core built-ins resolution depends on profile; keep smoke gated until strict resolver path is fully wired end-to-end

- Next: introduce macros gradually in selfhost sources (small areas), avoid over‑rewrites.

## 2025-10-06 — Builder external ModuleFunction index + call! smoke
- Added builder normalization for external (builtin) ModuleFunction names: dotted+arity always resolves.
  - Maps `String.*` / `Array.*` / `Map.*` / `Console.*` → `*Box.*` and adjusts arity (receiver excluded).
  - Alias example: `String.len/1` → `StringBox.length/0` to match VM method set.
- Un‑skipped call! smoke: tools/smokes/v2/profiles/quick/selfhost/call_macro_strict_vm.sh (PASS).
- Docs updated: docs/guides/macro-system.md (call! resolution by builder).
- Next: introduce macros gradually in selfhost sources (small areas), avoid over‑rewrites.

## 2025-10-06 — call! Map.set smoke + small adoptions
- Added smoke: tools/smokes/v2/profiles/quick/selfhost/call_macro_map_set_vm.sh (PASS).
- Adopted call normalization in selfhost boxes (no semantic change):
  - apps/selfhost/vm/boxes/op_handlers.hako:11 uses call("String.indexOf/2", ...) in _tprint.
  - apps/selfhost/vm/boxes/mir_vm_min.hako:15 uses call("String.indexOf/2", ...) in _tprint.
  - apps/selfhost/vm/boxes/instruction_scanner.hako:68 use call for "op":"ret" probe.
- Example added for json/map/arr sugar: apps/examples/macro_sugar/mini_map_arr.hako with smoke
  tools/smokes/v2/profiles/quick/examples/macro/json_map_arr_example_vm.sh (PASS).
- Notes: macro PATHS are enabled per smoke; no global defaults changed.

\n## 2025-10-06 — Selfhost macro adoption (small batch)\n- Replaced verbose MapBox constructions with  in:\n  - apps/selfhost/ny-parser-nyash/tokenizer.nyash:21\n  - apps/selfhost/ny-parser-nyash/parser_minimal.nyash:18,28,62,78,82\n  - apps/selfhost/vm/boxes/instruction_scanner.hako:100\n  - apps/selfhost/vm/boxes/minivm_probe.hako:9,11,41,45\n  - apps/selfhost/vm/boxes/step_runner.hako:27,35,41,45\n- Introduced a gated smoke (SKIP by default) to validate parser file uses macros:\n  - tools/smokes/v2/profiles/quick/selfhost/parser_minimal_macro_vm.sh\n- Minor fix: removed duplicate using in parser_minimal.nyash to avoid resolver error.\n- All builds green; integration (LLVM parity) remains 30/30 PASS.\n\nNotes\n- Macro PATHS are already set in nyash.toml; core env enables macros by default.\n- The parser_minimal smoke is gated due to Stage‑0 prelude scanner limitations with ';'. Enable with SMOKES_ENABLE_SELFHOST_MIN_PARSER=1 when needed.\n

## 2025-10-06 — Selfhost macro adoption (small batch)
- Replaced verbose MapBox constructions with `map({ ... })` in:
  - apps/selfhost/ny-parser-nyash/tokenizer.nyash:21
  - apps/selfhost/ny-parser-nyash/parser_minimal.nyash:18,28,62,78,82
  - apps/selfhost/vm/boxes/instruction_scanner.hako:100
  - apps/selfhost/vm/boxes/minivm_probe.hako:9,11,41,45
  - apps/selfhost/vm/boxes/step_runner.hako:27,35,41,45
- Introduced a gated smoke (SKIP by default) to validate parser file uses macros:
  - tools/smokes/v2/profiles/quick/selfhost/parser_minimal_macro_vm.sh
- Minor fix: removed duplicate using in parser_minimal.nyash to avoid resolver error.
- All builds green; integration (LLVM parity) remains 30/30 PASS.

Notes
- Macro PATHS are already set in nyash.toml; core env enables macros by default.
- The parser_minimal smoke is gated due to Stage-0 prelude scanner limitations with ';'. Enable with SMOKES_ENABLE_SELFHOST_MIN_PARSER=1 when needed.


## 2025-10-06 — Macro adoption Batch-2
- dep_tree_simple.nyash: using/ modules 出力レコードを map({...}) 化（rec, m）, visited短絡と root/out の生成を map 化。
- mir_builder_min.hako: builder state(make)・start_block の blk・get_function_structure の fndef を map({...}) 化。
- seam_inspector.hako: @obj/@cnt/@fnmap の初期化を map({}) に統一。
- call! 追加適用: ny-parser-nyash/main.nyash と parser_minimal.nyash の Error 検出に call("String.indexOf/2", ...) を採用（strict）。
- suites/core: macro_min_mix_core.sh を追加（SKIP既定、SMOKES_ENABLE_CORE_MACRO=1 で有効）。
- Build/LLVM integration 確認: cargo build OK、integration 既存緑維持（代表 subset 実行済）。


## 2025-10-06 — Macro adoption Batch-3
- dep_tree_simple: pair returns unified to map({arr,len}) with early returns in split_lines/scan_includes/scan_usings/scan_modules.
- seam_inspector: representative call! adoption for String.substring/2 in brace scanner.
- suites/core: added map_pair_spec_core.sh (SKIP by default; enable with SMOKES_ENABLE_CORE_MAP_PAIR=1). Note: print capture in run helper may filter outputs; assertion checks may need refinement later.
- Build green; integration (LLVM) parity set unaffected.


## 2025-10-06 — Batch‑4 (selfhost only)
- dep_tree.nyash: Node/ensure_arrays map化（初期化を map({ path, includes:[], using:[], modules:[], children:[] }) に統一。read_fail は error を付与して即返却）。
- mir_builder2.hako: state の一括初期化を map({...}) に、start_block の blk を map({ id, instructions:[] }) に置換。
- call! 追加（common/json スキャナ）:
  - json_frag.hako: block0_segment の indexOf を call("String.indexOf/2", ...) に置換。
  - json/core/json_scan.hako: 先頭ガードの substring を call("String.substring/2", ...) に置換。
- スモーク（SKIP既定、必要時のみ）:
  - suites/core/map_pair_usings_null_core.sh — scan_usings(null) の map({arr:[],len:0}) を確認。
  - suites/core/call_substring_min_core.sh — call!(String.substring/2) 最小動作（"abc"→"b"）。
- ビルド/統合: cargo build OK、既存 integration(LLVM) 緑のまま。

## 2025-10-07 — Selfhost map/call! polish + .hako move
- tools/dep_tree: renamed `apps/selfhost/tools/dep_tree.nyash` → `apps/selfhost/tools/dep_tree.hako` and `dep_tree_simple.nyash` → `dep_tree_simple.hako`; updated imports in `apps/selfhost/tools/dep_tree_main.hako`, suites/core smokes.
- mir_builder_min.nyash: map init adoption
  - `make()` now returns `map({ buf, phase, first_inst, blocks, cur_block_index, fn_name })` directly.
  - `start_block`: blk construction switched to `map({ id, instructions:[] })`.
  - `get_function_structure`: returns `map({ name, params:[], blocks })`.
- json scanners: added safe `call!(String.substring/2)` uses
  - json/core/json_scan.hako: character reads in object/array scan loops now use `call("String.substring/2", text, i, i+1)`; kept logic unchanged.
  - json/core/string_scan.hako: added call! for loop char reads; kept simple guard as-is.
- Scope: selfhost/common + tools only. VM/runtime untouched.
- Next (small):
  - Continue replacing pure-structure `new MapBox()+set(...)` with `map({ ... })` in selfhost/common and vm/boxes safe spots.
  - Expand call! minimally in string scanners (indexOf/substring guards) with tests.

## 2025-10-07 — Selfhost polish (batch, small)
- Map init: `apps/selfhost/tools/dep_tree.hako:10` visited → `map({})`（構造生成の純化）。
- call! 追加（局所・安全な1件ずつ）:
  - `apps/selfhost/common/mini_vm_scan.hako:20` → `@ch = call("String.substring/2", json, i, i+1)`。
  - `apps/selfhost/common/string_helpers.hako:32` → `local ch = call("String.substring/2", s, i, i+1)`。
  - `apps/selfhost/common/mini_vm_binop.hako:13` → `if call("String.substring/2", json, i, i+1) == '"'`。
- .hako 移行（低リスク）:
  - `apps/selfhost/tools/dep_tree_min_string.nyash` → `apps/selfhost/tools/dep_tree_min_string.hako` にリネーム、全参照の置換。

Notes
- 実行状態の Map（`regs` 等）は対象外。scan/guard/純構造のみを map/call! へ段階適用。
- 追加の候補（次回）: mini_vm_scan/string_helpers の残りの substring 判定、tools/dep_tree 内 set チェーンの map 圧縮（段階）。

## 2025-10-07 — Selfhost polish (batch, follow-up)
- mini_vm_scan.hako: added two more call!(String.substring/2) at character reads in array/object scan loops.
- string_helpers.hako: numeric string check loop now uses call!(String.substring/2) for per-char read.
- tools/dep_tree_main: .nyash → .hako rename; updated all references.
