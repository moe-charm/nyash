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
- [x] ny-llvmc links exe with `libhako_kernel.a` (or `libnyash_kernel.a`) automatically
- [x] Quick AOT smokes (compile+link+run)
  - tools/smokes/v2/profiles/quick/llvm/aot_const_ret_exe.sh (expects exit=0)
  - tools/smokes/v2/profiles/quick/llvm/aot_compare_branch_exe.sh (expects exit=1)
- [ ] Expand stubs toward real semantics (string/collections) as needed; keep strict and minimal for now

Notes
- These stubs do not allocate or hold handles; they exist to unblock AOT linking and basic integer‑only execution.
- When real string/collections are exercised, swap to `nyash_kernel` (full shim) or gradually enrich `hako_kernel`.

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
  - LLVM externcall: `nyrt.time.now_ms` / `nyrt.array.size` / `nyrt.map.size` は JSON spec を優先し、JSON 未指定時のみ legacy シグネチャへフォールバック（Phase‑B 導線）。
  - WASM: `WasmExternAdapterBox` を追加。`nyrt.*` import は Adapter が生成し、runtime/codegen の直書きを撤去済み。
  - LLVM（ハーネス第一）: Python 側の externcall は JSON Spec を優先し、既定はアンダースコア命名（`nyrt.time.now_ms` → `nyrt_time_now_ms`）。未知 extern は Fail‑Fast。
  - Validator: MIR→JSON 直前に必須キー検証を追加（`src/runner/mir_json_validate.rs`）。unop/binop/compare/externcall の最小スキーマを確認。

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
