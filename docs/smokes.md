# Smoke Matrix (Backends)

Policy
- PyVM: partial coverage only (no async/nowait/await/GC/sync). Used for semantics sanity.
- LLVM: full coverage via llvmlite harness. Preferred path for async/nowait and Stage-3 control-flow.
- JIT: not maintained for these smokes (skip).

Curated Smokes
- Core (LLVM): `examples/llvm11_core_smoke.nyash`
- Async (LLVM only):
  - `apps/tests/async-await-min/main.nyash`
  - `apps/tests/async-spawn-instance/main.nyash`
  - `apps/tests/async-await-timeout-fixed/main.nyash` (set `NYASH_AWAIT_MAX_MS=100`)

Runner Scripts
- `tools/smokes/curated_llvm.sh [--phi-off]`
  - `--phi-off`: enables `NYASH_MIR_NO_PHI=1` + verifier relaxor.
- Archived (not maintained): see `tools/smokes/archive/`
  - `smoke_phase_10_10.sh`, `smoke_vm_jit.sh`, `smoke_async_spawn.sh`, `jit_smoke.sh`, `aot_smoke_cranelift.sh`
  - These target JIT/Cranelift-era flows. Use the curated LLVM runner instead.

Flags
- `NYASH_LLVM_USE_HARNESS=1`: use llvmlite harness for AOT.
- `NYASH_MIR_NO_PHI=1` and `NYASH_VERIFY_ALLOW_NO_PHI=1`: PHI-less MIR (edge-copy) mode.

Notes
- Marked async smokes should not be run under PyVM/JIT.
- Legacy/archived smokes may exist; curated runner avoids them.
