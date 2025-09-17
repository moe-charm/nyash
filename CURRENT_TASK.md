# Current Task — Phase 15 Self‑Hosting (2025‑09‑17)

Summary
- Default execution is MIR13 (PHI‑off). Bridge/Builder do not emit PHIs; llvmlite synthesizes PHIs when needed. MIR14 (PHI‑on) remains experimental for targeted tests.
- PyVM is the semantic reference engine; llvmlite is used for AOT and parity checks.

What Changed (recent)
- MIR13 default enabled
  - `mir_no_phi()` default set to true (can disable via `NYASH_MIR_NO_PHI=0`).
  - Curated LLVM runner defaults to PHI‑off; `--phi-on` enables MIR14 lane.
  - Added doc: `docs/development/mir/MIR13_MODE.md`; README references it.
- JSON v0 Bridge lowering refactor + features
  - Split helpers: `src/runner/json_v0_bridge/lowering/{if_else.rs, loop_.rs, try_catch.rs, merge.rs}`（既存）に加え、式系を `lowering/expr.rs` に分離（振る舞い不変）。
  - 新規サポート: Ternary/Peek の Lowering を実装し、`expr.rs` から `ternary.rs`/`peek.rs` へ委譲（MIR13 PHI‑off=Copy合流／PHI‑on=Phi 合流）。
  - Self‑host 生成器（Stage‑1 JSON v0）に Peek emit を追加: `apps/selfhost-compiler/boxes/parser_box.nyash`。
  - Selfhost/PyVM スモークを通して E2E 確認（peek/ternary）。
- llvmlite stability for MIR13（bring‑up進行中）
  - Control‑flow 分離: `instructions/controlflow/{branch,jump,while_.py}` を導入し、`llvm_builder.py` の責務を縮小。
  - プリパス導入（環境変数で有効化）: `NYASH_LLVM_PREPASS_LOOP=1`
    - ループ検出（単純 while 形）→ 構造化 lower（LoopForm失敗時は regular while）
    - CFG ユーティリティ: `cfg/utils.py`（preds/succs）
  - 値解決ポリシー共通化: `utils/values.py`（prefer same‑block SSA → resolver）
  - vmap の per‑block 化: `lower_block` 内で `vmap_cur` を用意し、ブロック末に `block_end_values` へスナップショット。cross‑block 汚染を抑制。
  - Resolver 強化: end‑of‑block解決で他ブロックのPHIを安易に採用しない（自己参照/非支配回避）。
- Parity runner pragmatics
  - `tools/pyvm_vs_llvmlite.sh` compares exit code by default; use `CMP_STRICT=1` for stdout+exit.
  - Stage‑2 smokes更新: `tools/selfhost_stage2_smoke.sh` に "Peek basic" を追加。

Current Status
- Self‑hosting Bridge → PyVM smokes: PASS（Stage‑2 代表: array/string/logic/if/loop/ternary/peek/dot-chain）
- PyVM core fixes applied: compare(None,x) の安全化、Copy 命令サポート、最大ステップ上限（NYASH_PYVM_MAX_STEPS）
- MIR13（PHI‑off）: if/ternary/loop の合流で Copy が正しく JSON に出るよう修正（emit_mir_json + builder no‑phi 合流）
- Curated LLVM（PHI‑off 既定）: 継続（個別ケースの IR 生成不備は未着手）
- LLVM ハーネス（llvmlite）:
  - `loop_if_phi`: プリパスON＋構造化whileで EXE 退出コード 0（緑）。
  - `ternary_nested`: vmap per‑block で安定度向上。残タスク: merge(ret) の PHI 配線をプリパス/resolve 側で確定・重複排除。

Next (short plan)
0) Refactor/Structure（継続）
   - controlflow の切出し完了（branch/jump/while）。binop/compare/copy の前処理を `utils/values.resolve_i64_strict` に集約（完了）。
   - vmap per‑block 化（完了）。builder の責務縮小と prepass/cfg/util への移譲（進行中）。
   - if‑merge プリパス実装: ret‑merge の構造化/PHI確定（予定）。
1) Legacy Interpreter/VM offboarding (phase‑A):
   - ✅ Introduced `vm-legacy` feature (default OFF) to gate old VM execution層。
   - ✅ 抽出: JIT が参照する最小型（例: `VMValue`）を薄い共通モジュールへ切替（`vm_types`）。
   - ✅ `interpreter-legacy`/`vm-legacy` を既定ビルドから外し、既定は PyVM 経路に（`--backend vm` は PyVM へフォールバック）。
   - ✅ Runner: vm-legacy OFF のとき `vm`/`interpreter` は PyVM モードで実行。
   - ✅ HostAPI: VM 依存の GC バリアは vm-legacy ON 時のみ有効。
   - ✅ PyVM/Bridge Stage‑2 スモークを緑に再整備（短絡/三項/合流 反映）
2) Legacy Interpreter/VM offboarding (phase‑B):
   - 物理移動: `src/archive/{interpreter_legacy,vm_legacy}/` へ移設（ドキュメント更新）。
3) LLVM/llvmlite 整備（優先中）:
   - MIR13 の Copy 合流を LLVM IR に等価反映（pred‑localize or PHI 合成）: per‑block vmap 完了、resolver 強化済。
   - 代表ケース:
     - `apps/tests/loop_if_phi.nyash`: プリパスONで緑（退出コード一致）。
     - `apps/tests/ternary_nested.nyash`: if‑merge プリパスでの構造化/PHI 確定を実装 → IR 検証通過・退出コード一致まで。
   - `tools/pyvm_vs_llvmlite.sh` で PyVM と EXE の退出コード一致（必要に応じて CMP_STRICT=1）。
4) PHI‑on lane（任意）: `loop_if_phi` 支配関係を finalize/resolve の順序強化で観察（低優先）。
5) Runner refactor（小PR）:
   - `selfhost/{child.rs,json.rs}` 分離; `modes/common/{io,resolve,exec}.rs` 分割; `runner/mod.rs`の表面削減。
6) Optimizer/Verifier thin‑hub cleanup（非機能）: orchestrator最小化とパス境界の明確化。

How to Run
- PyVM reference smokes: `tools/pyvm_stage2_smoke.sh`
- Bridge → PyVM smokes: `tools/selfhost_stage2_bridge_smoke.sh`
- LLVM curated (PHI‑off default): `tools/smokes/curated_llvm.sh`
- LLVM PHI‑on (experimental): `tools/smokes/curated_llvm.sh --phi-on`
- Parity (AOT vs PyVM): `tools/pyvm_vs_llvmlite.sh <file.nyash>` (`CMP_STRICT=1` to enable stdout check)
  - 開発時の補助: `NYASH_LLVM_PREPASS_LOOP=1` を併用（loop/if‑merge のプリパス有効化）。

Operational Notes
- 環境変数
  - `NYASH_PYVM_MAX_STEPS`: PyVM の最大命令ステップ（既定 200000）。ループ暴走時に安全終了。
  - `NYASH_VM_USE_PY=1`: `--backend vm` を PyVM ハーネスへ切替。
  - `NYASH_PIPE_USE_PYVM=1`: `--ny-parser-pipe` / JSON v0 ブリッジも PyVM 実行に切替。
  - `NYASH_CLI_VERBOSE=1`: ブリッジ/エミットの詳細出力。
- スモークの実行例
  - `timeout -s KILL 20s bash tools/pyvm_stage2_smoke.sh`
  - `timeout -s KILL 30s bash tools/selfhost_stage2_bridge_smoke.sh`

Backend selection (Phase‑A after vm‑legacy off)
- Default: `vm-legacy` = OFF, `interpreter-legacy` = OFF
- `--backend vm` → PyVM 実行（python3 と `tools/pyvm_runner.py` が必要）
- `--backend interpreter` → legacy 警告の上で PyVM 実行
- `--benchmark` → vm‑legacy が必要（`cargo build --features vm-legacy`）

Enable legacy VM/Interpreter (opt‑in)
- `cargo build --features vm-legacy,interpreter-legacy`
- その後 `--backend vm`/`--backend interpreter` が有効

Key Flags
- `NYASH_MIR_NO_PHI` (default 1): PHI‑off when 1 (MIR13). Set `0` for PHI‑on.
- `NYASH_VERIFY_ALLOW_NO_PHI` (default 1): relax verifier for PHI‑less MIR.
- `NYASH_LLVM_USE_HARNESS=1`: route AOT through llvmlite harness.
- `NYASH_LLVM_TRACE_PHI=1`: trace PHI resolution/wiring.

Notes / Policies
- Focus is self‑hosting stability. JIT/Cranelift is out of scope (safety fixes only).
- PHI generation remains centralized in llvmlite; Bridge/Builder keep PHI‑off by default.
- No full tracing GC yet; handles/Arc lifetimes govern object retention. Safepoint/barrier/roots are staging utilities.
 - Legacy Interpreter/VM は段階的にアーカイブへ。日常の意味論確認は PyVM を基準として継続。
