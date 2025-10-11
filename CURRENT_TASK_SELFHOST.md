# CURRENT TASK — Self‑Host Quick Resume (Phase 15)

Scope
- This file tracks the Self‑Hosting line (VM/LLVM first). WASM work lives in a separate folder/branch. Ignore WASM commits here except the minimal shared specs.

Status — Pre‑restart checks (done)
- Pushed to selfhost branch: recent fixes are on record
  - PHI JSON format unified to `values[]` (no `incoming` in output)
    - commit: 5e7bc9ea
  - CLI `--entry` wired directly to VM entry resolution (Strict `Main.main`, CLI override)
    - commit: 7c500bae
  - README and smokes updated to prefer `hako` CLI; `nyash` kept as alias (deprecation banner only)
    - commits: 57151b1c, a4fe896c
  - Rust VM array output/prints stabilization (ArrayBox + collect_prints)
    - commits: c2e4eeae, 41f0cf6b

Resume after restart
- Build
  - `cargo build --release`
  - Binary: `target/release/hako`（nyash は互換エイリアスがある環境も）
- Quick smokes (optional)
  - `SMOKES_ENABLE_ENTRY=1 NYASH_DISABLE_PLUGINS=1 tools/smokes/v2/profiles/quick/core/cli_entry_ok.sh`
  - Run entry‑gated cases with the env gate ON
- Representative strict runs
  - `./target/release/hako --backend vm apps/APP/main.nyash`
  - Alternate entry: `./target/release/hako --backend vm --entry App.main apps/APP/main.nyash`

Notes
- PHI JSON is unified to `values[]`; emitters must not output `incoming`. Readers accept `values` primarily and may accept legacy `incoming` for compatibility.
- CLI naming: バイナリは `hako`。`nyash` は環境により互換で利用可能。
- Entry smokes are gated via `SMOKES_ENABLE_ENTRY=1` by design.

Next actions — Phase 15.9 front MVP (Ny→JSON v0)
- [ ] Front pipeline MVP (const/binop/compare/if/loop/ret/call) emitting JSON v0 via ParserBox + emitter (fail-fast on unsupported forms)
- [ ] CLI: `--emit-mir` and `--emit-exe` wiring on `hakorune` (ENV stays opt-in; document defaults)
- [ ] Quick smokes (Result-line compare) for:
  - selfhost_min_const_ret (VM/LLVM)
  - selfhost_min_if_merge (VM/LLVM)
  - selfhost_min_loop_sum (VM/LLVM)
- [ ] CURRENT_TASK_SELFHOST + docs: keep timeline (Day 1/2 front MVP, Day 3/4 calls/boxcall, Day 5 CLI polish) in sync
- [ ] Bench harness notes: ensure LLVM/WASM legs point to the same `NYASH_NY_LLVM_COMPILER` / `NYASH_EMIT_EXE_NYRT`

Progress (2025‑10‑02)
- [x] Added `JsonProgramBox` for AST JSON normalization + `meta.usings` injection.
- [x] Emitter delegates to the normalization box; runner inline path updated accordingly.
- [x] Quick smokes added:
  - `selfhost_source_inline_min_json_vm` (Runner→child `--source-inline`)
  - `selfhost_min_json_shape_if_vm` (If ノード存在)
  - `selfhost_json_normalize_shapes` (If/Loop/Call/Return + meta.usings)
  - `selfhost_json_normalize_edges`（Loop body null→[]、Call args null→[]、Nullノード保持）
- [x] JsonProgramBox 正規化を Local/Const/If/Loop/Return/Call まで拡張（キー順・既定値・空配列の扱いを固定）

Phase 15.7 — NyKernel (Option B) minimal AOT step
- [x] Introduce `crates/hako_kernel` minimal static shim (C‑ABI stubs)
  - Exports: nyash.box.from_i8_string / nyash.string.* (len_h, concat_hh, eq_hh, substring_hii, lastIndexOf_hh, to_i8p_h, from_u64x2, birth_h), nyash.any.length_h, nyash.env.box.new_i64x, births for Array/Map
  - Provides `main()` → calls `ny_main()` (no output, exit code propagated)
- [x] ny-llvmc links exe with `libhako_kernel.a` (or `libhako_kernel.a`) automatically
- [x] Quick AOT smokes (compile+link+run)
  - tools/smokes/v2/profiles/quick/llvm/aot_const_ret_exe.sh (expects exit=0)
  - tools/smokes/v2/profiles/quick/llvm/aot_compare_branch_exe.sh (expects exit=1)
- [ ] Expand stubs toward real semantics (string/collections) as needed; keep strict and minimal for now

Notes
- These stubs do not allocate or hold handles; they exist to unblock AOT linking and basic integer‑only execution.
- When real string/collections are exercised, swap to `hako_kernel` (full shim) or gradually enrich it.

Updated: 2025‑10‑01

---

Addendum — 2025‑10‑01 (late)
- MirVmMin: Minimal exec added for call/boxcall/newbox (i64 sum of args; pure). Also handles v1 `mir_call` similarly.
- New quick VM smokes (exec):
  - tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_call_exec_vm.sh
  - tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_method_exec_vm.sh
  - tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_newbox_exec_vm.sh
- Stage‑1 extractors: hardened to accept negatives/whitespace; emitters now accept string‑form args (e.g., "[5, 7]") and materialize ints. Follow‑up: verify PipelineV2 call path end‑to‑end; boundary smoke will be enabled after confirming args materialization.

Addendum — 2025‑10‑02
- Runner/Flow minimal box化（emit-only入口＆VM実行ヘルパ）
  - Added: `apps/selfhost-compiler/pipeline_v2/flow_entry.hako` (FlowEntryBox) — emit-only entry, v0 / v1→v0 互換
  - Added: `apps/selfhost/vm/flow_runner.hako` (FlowRunner) — FlowEntry→Mini-VM 実行の薄い箱
  - Mapped modules: `selfhost.compiler.pipeline_v2.flow_entry`, `selfhost.vm.flow_runner`（nyash.toml/hako.toml）
  - New smoke: `tools/smokes/v2/profiles/quick/selfhost/selfhost_flow_runner_return_int_vm.sh`（Return(Int 42)→exec=42）
- LocalSSA 材化ポリシー整理（PHI直後に統一）
  - ensure_calls: 実装済（v0/v1のrecv/argsに対応）
  - ensure_cond: ブロック先頭→PHI直後に変更（copy位置を統一）
  - 既存 selfhost PipelineV2 smokesは NYASH_PIPELINE_V2=1 で回して緑を確認

Plan snapshot — 2025‑10‑03 preview
- **Day 1–2**: Front MVP
  - Implement Ny→JSON v0 for const/binop/compare/if/loop/ret/call.
  - Document interface expectations in `docs/development/selfhosting/front_mvp.md` (new).
  - Add quick smokes comparing VM vs LLVM outputs (Result lines).
- **Day 3–4**: Calls & BoxCall minimal
  - Extend DP reuse for argument lowering; keep runtime defaults unchanged.
  - Add extern quick cases (string len / concat parity).
- **Day 5**: CLI / Docs polish
  - Wire `--emit-mir` / `--emit-exe` to hakorune; minimize ENV surface.
  - Refresh README & CURRENT_TASK, link bench harness instructions.

Addendum — 2025‑10‑03 (TimerBox P1 + quick 緑化)
- Core Kernel: TimerBox P1 実装（now_ms のみ、単調時刻ms）。
  - VM/LLVM/WASM へ `nyrt.time.now_ms` を配線（WASM は MVP として `Date.now()`）。
  - 薄い箱: `apps/core/timer/TimerBox.hako` を modules に登録（`selfhost.core.timer`）。
  - Builder 正規化: `new TimerBox().now_ms()`/`TimerBox.now_ms()` → `ExternCall("nyrt.time","now_ms")`。
  - Quick スモーク追加: `core/timer_now_ms_vm.sh`（VM）/ `llvm/timer_now_ms_harness.sh`（ハーネス有なら実行、無ければ SKIP）。

- quick プロファイルの安定化（LLVM系）
  - ハーネス未検出時は `run_nyash_llvm` ヘルパ経由で SKIP に変更（quick は高速/安定優先）。
  - AOT で静的プラグインを要するケース（Array/Map）は未構成なら SKIP（`aot_array_push_len_exe.sh`, `aot_map_set_size_exe.sh`）。
  - Selfhost Pipeline V2 の一部は `NYASH_PIPELINE_V2=1` で有効化（既定は SKIP）。

- 現在の状態
  - Quick プロファイル: 全緑（172/172 PASS）を確認。
- modules 既定解決: hako/nyash.toml の重複定義を解消し、`selfhost.core.timer` を含む modules 自動登録が安定（スモークから `NYASH_MODULES` 除去済み）。
  - selfhost_min_* parity: `call_static` ケースを追加、LLVM ハーネス未検出時は SKIP 維持

Addendum — Effects 決定の箱化（Phase‑in 最小）
- 追加ドキュメント: docs/development/mir/effects-resolver.md
- 実装（最小）:
  - `src/mir/builder/effects/{mod.rs,resolver.rs}` を追加
  - `NYASH_USE_EFFECT_RESOLVER=1` で Unified Call の効果決定をテーブル解決（extern/method）に委譲（未知は既存ロジックへフォールバック）
  - `NYASH_EFFECT_TRACE=1` で解決ログを出力
- 併走ガード（今回の目的）:
  - `nyrt.time.now_ms` は常に READ（Unified/Legacy/nyrt.* の全経路）。
  - 既存の `compute_extern_effects` へ `nyrt.time.now_ms` を追加（仕様不変の観点から二重に安全化）

Addendum — Effects verifier / Origin tracker / Call router 骨格
- `NYASH_VERIFY_EFFECTS=1`: Call/BoxCall/ExternCall に PURE 混入があった場合に警告（軽量 Verifier）。
- OriginTrackerBox 導線: `src/mir/builder/origin/tracker.rs` を追加し、`MirBuilder::origin_register/get/propagate` を新設（value_origin_newbox の薄ラップ）。
- CallRoutingBox 骨格: `src/mir/builder/router/call_router.rs` を追加。`NYASH_USE_CALL_ROUTER=1` で TimerBox.now_ms → `nyrt.time.now_ms` の直行経路を委譲（trace: `NYASH_CALL_ROUTER_TRACE=1`）。既定では従来処理のまま。

Addendum — Externs Registry（疎結合アーキテクチャへの導線）
- 目的: MIR 層は extern 情報（意味論）だけを持ち、バックエンド名/ABI は Adapter 側に分離する。
- docs: `docs/development/architecture/externs_registry.md` を追加（設計/段階導入/受け入れ基準）。
- 現状（Phase‑A）: 最小レジストリで effects を一元化。WASM 名/LLVM 名は暫定的に併存し、Resolver 優先で整合を取る（既定挙動不変）。
- 次のステップ（Phase‑B）: Registry を抽象 spec のみに縮退し、各 Backend に Adapter を新設（命名規則＋例外表）。LLVM は dev 時 JSON で spec を取得して参照。
  - 受け手: TimerBox.now_ms / ArrayBox.length|size / MapBox.size（READ/ゼロ引数）を Router=ON で直行化。
  - スモーク: `core/router_timer_now_ms_vm.sh`, `core/router_array_size_vm.sh`, `core/router_map_size_vm.sh` を追加（Router=ON で緑を維持）。
  - LLVM externcall: `nyrt.time.now_ms` / `nyrt.array.size` / `nyrt.map.size` は JSON spec を必須化（欠落時は Fail‑Fast）。
  - WASM: `WasmExternAdapterBox` を追加。`nyrt.*` import は Adapter が生成し、runtime/codegen の直書きを撤去済み。
  - LLVM（ハーネス第一）: Python 側の externcall は JSON Spec を優先し、既定はアンダースコア命名（`nyrt.time.now_ms` → `nyrt_time_now_ms`）。未知 extern は Fail‑Fast。
  - Validator: MIR→JSON 直前に必須キー検証を追加（`src/runner/mir_json_validate.rs`）。unop/binop/compare/externcall/typeop/newbox/boxcall/call/branch/jump/ret/copy をカバー。

Next actions — Phase‑B（Externs Registry → Adapter 分離）
- [x] Registry を ExternCallSpec（抽象）へリファクタ（wasm/llvm 名は削除）
- [x] WASM: WasmExternAdapterBox を追加（規則＋例外；不明は Fail‑Fast）。codegen は Adapter 経由で import 名を解決。
- [ ] LLVM: extern_adapter.py（もしくは Rust 側）で JSON spec をロード→シンボル名解決。未知はエラー（dev では警告可）。
- [ ] VM: VmExternAdapterBox にハンドラ登録テーブルを移設（現行 match を段階撤退）。
- [ ] Router スモーク拡充（READ/ゼロ引数 getter系を 1 つずつ）：追加ごとに 1 本。
- [ ] Docs 更新（追加手順: Registry1行 + Adapter1行 + Smoke1本）。

Open — LLVM harness issues to fix next
  - [ ] Duplicate symbol guard in externcall.py（関数が二重宣言される希なパスの対策）
  - [ ] AOTリンクでの `nyrt_*` 解決を安定化（必要なら Kernel のエクスポート記名を一覧化）

Update — 2025‑10‑02 (Phase‑B 小結)
- 完了:
  - Legacy extern フォールバック（Timer/Array/Map）を撤去。Registry JSON を唯一の情報源に固定。
  - MIR JSON Validator の対象拡大（call/branch/jump/ret/copy を追加）。
  - Router 系スモーク（timer/array/map）を整備し、Router=ON で PASS を確認。
- 残タスク:
  - selfhost_mir_m2_multi_compare_gt_last_ret_vm の期待値ズレ（1→6）調査（比較連鎖の Lowering/実行系の食い違い）。
  - 文字列/boxing 系の legacy sig_map を Registry+Adapter に段階移行（JSON spec 拡充とセット）。
  - Validator の `safepoint/load/store` などへの拡張（登場時に段階導入）。

Next — 直近の実施順（提案）
1) スモーク緑化の仕上げ
   - selfhost_m2_multi_compare の原因切り分け（MIR 出力 vs 実行器）。失敗再現→最小 repro 追加→修正。
2) レガシー純化の継続
   - externcall.py の sig_map（string/boxing 群）の段階削減（先に Registry へ spec を追加）。
3) Adapter の拡充
   - LLVM 側の JSON ローダ周辺の冪等性/重複宣言対策の磨き込み。
4) Docs/Smokes 同期
   - JSON スキーマの最小定義を docs に明記。該当スモークに SKIP 条件（環境未整備）を追加維持。

---

Now — 切り分けと当面の作業（2025‑10‑03）

- 問題切り分け
  - Rust VM 層: TimerBox.now_ms の CSE 混入疑い（ExternCall の EffectMask 判定が散在）
  - Self‑Host(.hako): LocalSSA.ensure_cond の文字列手術が壊れやすい（エスケープ/カンマ/境界）

- 根治方針（構造→ドキュメント→テスト→コード）
  1) 構造: EffectResolver を“唯一の効果決定点”に収束（now_ms は READ 固定）。CSE は READ を再利用不可に明示。
  2) Docs: externs_registry.md に Fail‑Fast と命名規則（dotted/underscores）＋ effects と最適化の関係を追記。
  3) テスト（新規スモーク）
     - quick/selfhost/selfhost_localssa_copy_plain_vm.sh（LocalSSA 出力に素の `{"op":"copy"}` が入る）
     - quick/core/timer_now_ms_nocse_vm.sh（now_ms の 2 回呼びで `delta>0` を保証。0 の場合は RED）
  4) コード（点修正）
     - VM CSE: READ を抑止条件に追加（ExternCall は既定 PURE 不可）。
     - LocalSSA: pipeline_v2 は LocalSSABox の配列ベース挿入を優先。builder の文字列手術は最小修正に留める。

- 受け入れ条件（この段階）
  - 上記 2 本のスモークが quick で PASS（環境未整備時は SKIP）。
  - 既存 quick が常緑（Router/LLVM は現行の SKIP 方針維持）。

Delta — 本コミットの変更点（2025‑10‑03 午前）
- Quick スモークの「コア常在ルール」寄せ（using/new 依存の縮小・ok/ng 判定・プラグイン無効化）
  - core/router_timer_now_ms_vm.sh: using/new を排し、静的 TimerBox.now_ms の単調性を in‑program 判定（ok/ng）に変更。NYASH_DISABLE_PLUGINS=1 を追加。
  - core/router_array_size_vm.sh, core/router_map_size_vm.sh: プラグイン無効化＋ok/ng 判定化。Router/Adapter 経路が不在の環境では SKIP（事前プローブで判定）。
  - core/basic_print.sh: プラグイン無効化を追加（print の純粋性を担保）。
- EffectResolver 一元化の導線強化
  - builder/calls/extern_calls.rs: `compute_extern_effects` が NYASH_USE_EFFECT_RESOLVER=1 のとき Resolver を優先参照 → Registry → 既存ヒューリスティック の順に統一。
  - 既存の挙動は不変（既定OFF）。段階的にヒューリスティックの削減を予定。
- LocalSSA 拡張（設計のみ・次手）
  - pipeline_v2 は既に LocalSSABox を導入済み。次段で ensure_cond を LocalSSABox.ensure_after_phis_copy に寄せ、配列ベース挿入へ一体化する（挙動不変のまま）。

次の一手（この順で進める）
1) ルータ系スモークの追加寄せ（flow/basic などの using/new 依存を削減）。
2) EffectResolver の既定ON 準備（ログとトレースを整え、重複ヒューリスティックを削る）。
3) LocalSSA.ensure_cond を LocalSSABox に接続（配列ベース挿入を既定に）。Validator の `{op,src,dst}` Fail‑Fast を強化。


---

Update — 2025‑10‑04 (direct calls / -O3 / Mini‑VM polish)
- Builder: prefer direct ModuleFunction when resolvable (avoid legacy string callees)
- LLVM harness: enable -O3 pass pipeline via llvmlite (DCE/inline/const fold)
- Mini‑VM: φ decode hardened (empty / all‑malformed) and Throw terminator (-2) added
- Header emit: compiler’s early --min-json path emits locally (prelude‑free); pipeline_v2 HeaderEmitBox kept for later unification

Self‑host next (small, safe)
- Parser finish (static calls): unify expr/stmt boxes to call scan/utils statically (no large refactor)
- Using/Modules E2E: add one more alias case (SKIP if env not ready)
- Later: move early header back to HeaderEmitBox after AST prelude is stable


Update — 2025‑10‑04 (header emit flag / alias commonization)
- Added env `NYASH_MINJSON_USE_HEADER_BOX=1` to prefer HeaderEmitBox for compiler --min-json early path.
  - Parent runner passes `--emit-header-box` to child when set. Default remains local emit.
- Using aliases: added common helper to register alias→canonical path in modules registry.
  - Applied in VM/LLVM/VM-fallback modes to persist aliases even in quiet child pipelines.
- Next: consider making the flag default ON after stability, then retire local emit.

## 2025-10-07 — Self‑Host Prep Recap + Task Outline (Phase 15.7)

Prep done (ready to resume)
- DepTree: annotate + summary boxes wired; dep_tree_main supports `--summary` (optional).
- JSON scan: `StringScanBox.read_char/find_quote/read_string_end` added; `seek_obj_end/seek_array_end` use `find_quote` (1箇所ずつ導入)。
- .hako migration: selfhost/common & tools 完了（mir_v1_adapter を modules に追加、`mir_builder_min.nyash` 削除）。
- Smokes: quick 集約に `suites/core` を包含。`dep_tree_summary_core.sh` を追加（SKIP既定）。
- Lint (report‑only): `.nyash` 監視を 2 本用意（boxes/general）。fail トグルは将来ON予定。

Next tasks — Short list (Day 1–3)
- Front pipeline (Ny→MIR JSON v0) 仕上げ（const/binop/compare/branch/jump/ret/call/newbox/boxcall）。未対応は Fail‑Fast。
- Mini‑VM parity 強化（ret/phi/branch 代表ケースを `suites/core` に1本ずつ加点 → quick 緑維持）。
- Builder 外部 ModuleFunction のテスト拡充（String/Array/Map/Console の代表＋Timer.now_ms）。
- Using resolver E2E を 1 本だけ追加（[modules] alias 経由）。エラーメッセージの統一を継続。

Mid term (Phase 15.7 → 16)
- Externs Registry→Adapter 分離の後半（LLVM ローダの冪等性/重複宣言対策）。
- Macro adoption（call!/map({})）を selfhost 内の安全箇所から段階拡大（スモークは SKIP 既定）。
- Stage‑0/VM ドライバの `.nyash` は最後に一括で `.hako` へ（スモーク置換とセット）。

Guardrails / CI policy（開発中は弱め）
- CI 最小: `cargo build --release` + quick スモークのサブセット。`dep_tree_summary_core` は SKIP 既定。
- Lint は report‑only で数サイクル運用→収束後に `LINT_*_FAIL=1` をON。
- Fail‑Fast: 新規は ENV/flag 既定OFF。観測/ログは最小・既定静音。

Run cheatsheet
- Build: `cargo build --release`
- Quick: `tools/smokes/v2/run.sh --profile quick`
- Summary (on demand): `./target/release/nyash apps/selfhost/tools/dep_tree_main.hako apps/selfhost/ny-parser-nyash/main.nyash --summary`


---

Plugin Unification — Static Registration Plan (Phase 15.7+)

Goal
- Single source of truth under plugins/. Core chooses static (features) or dynamic (dlopen) from the same plugin crates.
- Remove builtin CoreBox implementations (String/Array/Map) after parity; keep runtime boxes (Future/Result/Callable/Token/Console/Time) until plugin versions are wired.

Design
- Kernel features (hako_kernel):
  - core-runtime: future/result/callable/null/debug/token/function (plugins)
  - core-collections: string/array/map (plugins)
  - core-io, core-network, full (optional aggregates)
  - default = ["core-runtime", "core-collections"]
- Static registration API:
  - PluginHostV2::register_static(StaticTypeBox { box_type, type_id, invoke, birth, ... })
  - Each plugin exposes pub fn register_static(host: &mut PluginHostV2)
  - Kernel init: init_static_plugins() -> calls register_static per feature, then dynamic loader load_all_plugins(); type_id duplicates are skipped
- Provider order:
  1) static providers (features) 2) dynamic providers (dlopen) 3) builtin fallback (temporary; will be removed)

Acceptance
- quick: green with default features (bootstrap on)
- plugins profile: green for String/Array/Map/Callable (values/remove/identity)
- AOT: ny-llvmc resolves libhako_kernel.a and runs EXE (plugins optional)
- No duplicate-registration errors in logs; static wins over dynamic

Steps
1) Add features + init_static_plugins() in hako_kernel
2) Add PluginHostV2::register_static + duplicate-skip logic
3) Implement register_static() in nyash-array/map/string-plugin
4) quick: keep bootstrap (default features) ON to remain green; plugin-on smokes remain green
5) Remove builtin CoreBox impls (string/array/map) once parity confirmed
6) Migrate core runtime boxes to plugins (Null/Result/Callable → Future/Token) in small steps; keep env.future.* on host for now

Notes
- Map semantics: get→null missing, set/clear/push/sort/reverse→Null, remove→value-or-null (spec unified)
- methodRef remains VM pseudo-method; plugin does not need slot 113
- TLV tag=8/9 (PluginHandle/HostHandle) preserved; identity re-use via global handle cache is required for round-trips


Delta — Core Collections via Plugins (2025‑10‑11)
- ProviderBox unified NewBox entry. If a plugin box is created with instance_id=0, VM proactively calls birth() and adopts the returned handle.
- Static specs loaded from per‑plugin hako_box.toml (type_id/method slots known without full config).
- Env overlays: plugin‑on uses NYASH_PLUGIN_CONFIG=hako.toml; plugins profile keeps Stage‑1 keys/values (NYASH_PLUGIN_MAP_ARRAY_HANDLE=0).
- Smokes: quick plugin_on_* green (4/4); plugins profile green (21/21).

Next — Cleanup
- Remove any residual references to VM convenience handlers in docs/tests.
- Keep Stage‑2 HostHandle tests gated; re‑enable after host handle resolution tables are generalized.
- Consider raising bootstrap static features default in hako_kernel once AOT path is exercised again.



Update — 2025‑10‑11 (Runtime cleanup)
- Stage‑1 keys/values fallback was moved out of the router into `src/runtime/adapters/map_keys_values_stage1.rs`.
- Introduced `src/runtime/host_handle_router/` (thin entry today) to progressively move slot routing out of `host_api.rs`.
- Added README and LAYER_GUARD files to keep responsibilities explicit.
- Next: stabilize Stage‑2 identity (values() returns Array HostHandle referencing same instance) and add a print‑path smoke (to avoid futex deadlocks).
