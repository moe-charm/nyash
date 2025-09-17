MIR13 Mode (PHI-off by default)

Overview
- Goal: Stabilize execution by turning off PHI emission in the Bridge/Builder and letting the LLVM (llvmlite) layer synthesize PHIs as needed.
- Default: MIR13 is ON by default (PHI-off). Use env flags to flip.

Why
- Fewer SSA obligations in the front-end (Bridge/Builder) reduces CFG corner cases (short‑circuit, nested if/loop merges).
- Centralizes SSA invariants in a single place (llvmlite resolver/finalizer), improving correctness and maintenance.

Flags and Behavior
- NYASH_MIR_NO_PHI (default: 1)
  - 1: Bridge/Builder emit edge copies instead of PHIs at merges (MIR13).
  - 0: Bridge/Builder may emit PHIs (MIR14 experimental).
- NYASH_VERIFY_ALLOW_NO_PHI (default: 1)
  - Relaxes verifier checks that assume PHIs at merges.
- NYASH_LLVM_USE_HARNESS=1 (AOT via llvmlite harness)
  - Resolver/finalizer synthesize PHIs at block heads when needed.

LLVM (llvmlite) Responsibilities
- setup_phi_placeholders(): predeclare JSON‑provided PHIs and collect incoming metadata.
- block_end_values: snapshot per block end to materialize predecessor values (dominance‑safe).
- finalize_phis(): wire incoming edges for declared PHIs at block heads.
- Resolver.resolve_i64():
  - single‑pred: take predecessor end value;
  - multi‑pred + declared PHI: reuse placeholder at block head;
  - multi‑pred + no PHI: synthesize a localization PHI at the current block head (MIR13 compatibility);
  - avoids reusing non‑dominating vmap values across blocks.

Bridge/Builder (JSON v0) Behavior
- If/Loop/Try are lowered without PHIs when MIR13 is ON; merges are performed with edge copies (merge_var_maps) and the final value is reconstituted by the LLVM layer if needed.
- Helper split for readability (no behavior change):
  - lowering/{if_else.rs, loop_.rs, try_catch.rs, merge.rs}

Testing
- Curated LLVM (default = PHI‑off):
  - tools/smokes/curated_llvm.sh  (use --phi-on to exercise MIR14)
- PHI invariants/parity (AOT vs PyVM):
  - tools/pyvm_vs_llvmlite.sh (default compares exit code; use CMP_STRICT=1 for stdout+exit)
- Bridge/PyVM:
  - tools/selfhost_stage2_bridge_smoke.sh

How to Force PHI‑on (MIR14 experimental)
- Set: NYASH_MIR_NO_PHI=0 and run tools/smokes/curated_llvm.sh --phi-on
- Known: loop_if_phi may trip dominance issues in PHI‑on; MIR13 is the recommended default.

Known Limitations (current)
- No full tracing GC; object lifetime is managed via handle registries and Arc lifetimes.
- PHI‑on path is still being refined for some control‑flow patterns.

