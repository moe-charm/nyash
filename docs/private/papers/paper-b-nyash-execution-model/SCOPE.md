# Scope & Assumptions (2025-09-19)

## Execution Lines in Scope
- PyVM: semantic reference path used for parity checking
- LLVM/llvmlite: AOT/EXE harness with PHI synthesis and IR dump capability
- Excluded for this paper version: Cranelift JIT / Rust MIR Interpreter (Phase‑15 maintenance)

## MIR14 and PHI Policy
- MIR14 runs in PHI-off mode (edge-copy merges)
- PHIs are formed in LLVM at block heads with typed incoming pairs
- PyVM confirms semantics; LLVM provides performance and distribution artifacts

## Evaluation Plan (v0)
- Parity: PyVM vs LLVM on representative tests (truthy rules, short-circuiting, BoxCall)
- Performance: report LLVM numbers as primary; PyVM for semantics sanity
- Exclude JIT-related comparisons until stability returns

## Dev Toggles
- `NYASH_LLVM_USE_HARNESS=1`, `NYASH_LLVM_TRACE_PHI=1`, `NYASH_LLVM_DUMP_IR=...`
- `NYASH_MIR_NO_PHI=1` (default)

