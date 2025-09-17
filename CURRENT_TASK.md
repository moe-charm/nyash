# Current Task — Phase 15 Self‑Hosting (2025‑09‑17)

Summary
- Default execution is MIR13 (PHI‑off). Bridge/Builder do not emit PHIs; llvmlite synthesizes PHIs when needed. MIR14 (PHI‑on) remains experimental for targeted tests.
- PyVM is the semantic reference engine; llvmlite is used for AOT and parity checks.

What Changed (recent)
- MIR13 default enabled
  - `mir_no_phi()` default set to true (can disable via `NYASH_MIR_NO_PHI=0`).
  - Curated LLVM runner defaults to PHI‑off; `--phi-on` enables MIR14 lane.
  - Added doc: `docs/development/mir/MIR13_MODE.md`; README references it.
- JSON v0 Bridge lowering split (non‑functional)
  - Added `src/runner/json_v0_bridge/lowering/{if_else.rs, loop_.rs, try_catch.rs, merge.rs}` and routed calls from `lowering.rs`.
- llvmlite stability for MIR13
  - Resolver: forbids cross‑block non‑dominating vmap reuse; for multi‑pred and no declared PHI, synthesizes a localization PHI at block head.
  - Finalize remains function‑local; `block_end_values` snapshots and placeholder wiring still in place.
- Parity runner pragmatics
  - `tools/pyvm_vs_llvmlite.sh` compares exit code by default; use `CMP_STRICT=1` for stdout+exit.

Current Status
- Self‑hosting Bridge → PyVM smokes: PASS (Stage‑2 reps: array/string/logic/if/loop).
- Curated LLVM (PHI‑off default): PASS.
- Curated LLVM (PHI‑on experimental): `apps/tests/loop_if_phi.nyash` shows a dominance issue (observed; not blocking, MIR13 recommended).

Next (short plan)
1) JSON v0 lowering: split remaining helpers (peek/ternary/expr) without behavior change.
2) PHI‑on lane (optional): investigate `loop_if_phi` dominance by tightening finalize ordering and resolver materialization (low priority).
3) Runner refactor (small PRs):
   - `selfhost/{child.rs,json.rs}` split; `modes/common/{io,resolve,exec}.rs` split; reduce `runner/mod.rs` surface.
4) Optimizer/Verifier thin‑hub cleanup (non‑functional): orchestrator minimalization and pass boundaries clarified.

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

