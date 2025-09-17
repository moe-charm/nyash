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
- llvmlite stability for MIR13
  - Resolver: forbids cross‑block non‑dominating vmap reuse; for multi‑pred and no declared PHI, synthesizes a localization PHI at block head.
  - Finalize remains function‑local; `block_end_values` snapshots and placeholder wiring still in place.
- Parity runner pragmatics
  - `tools/pyvm_vs_llvmlite.sh` compares exit code by default; use `CMP_STRICT=1` for stdout+exit.
  - Stage‑2 smokes更新: `tools/selfhost_stage2_smoke.sh` に "Peek basic" を追加。

Current Status
- Self‑hosting Bridge → PyVM smokes: PASS (Stage‑2 reps: array/string/logic/if/loop).
- Curated LLVM (PHI‑off default): PASS.
- Curated LLVM (PHI‑on experimental): `apps/tests/loop_if_phi.nyash` shows a dominance issue (observed; not blocking, MIR13 recommended).

Next (short plan)
1) Legacy Interpreter/VM offboarding (phase‑A):
   - Introduce `vm-legacy` feature (default OFF) to gate old VM execution層。
   - 抽出: JIT が参照する最小型（例: `VMValue`）を薄い共通モジュールへ切替（`vm_types` 等）。
   - `interpreter-legacy`/`vm-legacy` を既定ビルドから外し、ビルド警告を縮小。
2) Legacy Interpreter/VM offboarding (phase‑B):
   - 物理移動: `src/archive/{interpreter_legacy,vm_legacy}/` へ移設（ドキュメント更新）。
3) PHI‑on lane（任意）: `loop_if_phi` 支配関係を finalize/resolve の順序強化で観察（低優先）。
4) Runner refactor（小PR）:
   - `selfhost/{child.rs,json.rs}` 分離; `modes/common/{io,resolve,exec}.rs` 分割; `runner/mod.rs`の表面削減。
5) Optimizer/Verifier thin‑hub cleanup（非機能）: orchestrator最小化とパス境界の明確化。

How to Run
- PyVM reference smokes: `tools/pyvm_stage2_smoke.sh`
- Bridge → PyVM smokes: `tools/selfhost_stage2_bridge_smoke.sh`
- LLVM curated (PHI‑off default): `tools/smokes/curated_llvm.sh`
- LLVM PHI‑on (experimental): `tools/smokes/curated_llvm.sh --phi-on`
- Parity (AOT vs PyVM): `tools/pyvm_vs_llvmlite.sh <file.nyash>` (`CMP_STRICT=1` to enable stdout check)

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
