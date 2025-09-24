MIR13 Mode (legacy PHI-off fallback)

Overview
- Goal: Retain the Phase‑14 edge-copy compatibility path for debugging historical MIR dumps or diagnosing SSA regressions.
- Default: MIR14 (PHI-on) now ships as the standard. MIR13 must be explicitly enabled through environment flags.

Why keep MIR13 around?
- Reproducibility: Some archived JSON v0 fixtures were captured before PHI-on shipped. MIR13 allows replaying them without regeneration.
- Diagnostics: Edge-copy runs make it easier to isolate builder regressions by removing PHI synthesis from the equation.
- Tooling parity: Certain scripts still compare MIR13 traces; they will be retired once PHI-on parity checks are complete.

Flags and Behavior
- NYASH_MIR_NO_PHI (default: 0)
  - 0: Builders emit PHIs at merge heads (MIR14, default).
  - 1: Builders drop PHIs and insert per-predecessor edge copies (MIR13 fallback).
- NYASH_VERIFY_ALLOW_NO_PHI (default: 0 unless PHI-off is requested)
  - Set this to 1 together with `NYASH_MIR_NO_PHI=1` when you intentionally relax SSA verification.
- NYASH_LLVM_USE_HARNESS=1 (AOT via llvmlite harness)
  - In MIR13 mode the harness synthesizes PHIs. In MIR14 it simply validates incoming edges.

LLVM (llvmlite) Responsibilities
- `setup_phi_placeholders()`: still records declared PHIs; in MIR13 mode it creates placeholders for later wiring.
- `block_end_values`: snapshots per block end to materialize predecessor values (dominance-safe).
- `finalize_phis()`: wires incoming edges for declared PHIs; when MIR13 runs, it creates PHIs on the fly to recover SSA.
- `Resolver.resolve_i64()`:
  - single-pred: take predecessor end value;
  - multi-pred + declared PHI: reuse the placeholder at the block head;
  - multi-pred + no PHI: synthesize a localization PHI at the current block head (MIR13 compatibility);
  - avoids reusing non-dominating vmap values across blocks.

Bridge/Builder (JSON v0) Behavior
- MIR14 (default): If/Loop/Try placements emit PHIs up front; loop latches, break/continue, and structured joins have explicit incoming pairs.
- MIR13 (fallback): Merges are performed with edge copies (`merge_var_maps`). Use only when reproducing historical issues.

Testing (v2)
- Integration suite (LLVM harness/PHI invariants):
  - `tools/smokes/v2/run.sh --profile integration`
- Bridge/PyVM の検証は v2 スイートに統合（必要に応じてフィルタを使用）

How to Force PHI-off (MIR13 fallback)
- Set: `NYASH_MIR_NO_PHI=1 NYASH_VERIFY_ALLOW_NO_PHI=1`
- Run integration: `tools/smokes/v2/run.sh --profile integration`
- Label the run as legacy in `CURRENT_TASK.md` if results inform shared debugging.

Known Limitations (current)
- MIR13 no longer receives new feature work; expect missing coverage for recent LoopForm updates.
- PHI-on is the supported path. MIR13 bugs are fixed only when they block diagnostics.
