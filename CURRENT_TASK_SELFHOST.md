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

Next actions
- [x] Add/update CURRENT_TASK for selfhost (this file)
- [x] Append a short PHI `values` spec to reference docs
- [x] Enable commit/push for selfhost branches (local hooks updated)
- [x] PipelineV2: Apply LocalSSA.ensure_cond as final pass (fail‑safe)
- [x] Add quick smokes for If(Compare) CFG and loop counter
- [x] Verify Jump lowering and add docs pointers (quick/selfhost jump smokes; LLVM PHI harness smokes)
 - [x] v1→v0 downgrade extern fallback (harness‑only): unresolved Global → externcall when NYASH_LLVM_DOWNGRADE_V1=1
   - Runner emit updated: src/runner/mir_json_emit.rs
   - New smoke: tools/smokes/v2/profiles/quick/llvm/harness_v1_downgrade_global_extern_compile_ok.sh
   - Fail‑Fast maintained for VM/AOT (unresolved Global → error)

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
  - Added: `apps/selfhost-compiler/pipeline_v2/flow_entry.hako` (FlowEntryBox) — emit‑only entry, v0 / v1→v0 互換
  - Added: `apps/selfhost/vm/flow_runner.hako` (FlowRunner) — FlowEntry→Mini‑VM 実行の薄い箱
  - Mapped modules: `selfhost.compiler.pipeline_v2.flow_entry`, `selfhost.vm.flow_runner`（nyash.toml/hako.toml）
  - New smoke: `tools/smokes/v2/profiles/quick/selfhost/selfhost_flow_runner_return_int_vm.sh`（Return(Int 42)→exec=42）
- LocalSSA 材化ポリシー整理（PHI直後に統一）
  - ensure_calls: 実装済（v0/v1のrecv/argsに対応）
  - ensure_cond: ブロック先頭→PHI直後に変更（copy位置を統一）
  - 既存 selfhost PipelineV2 smokesは NYASH_PIPELINE_V2=1 で回して緑を確認
