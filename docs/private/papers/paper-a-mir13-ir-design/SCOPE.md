# Scope & Assumptions (2025-09-19)

## Current Execution Lines
- In-scope: PyVM (semantic reference), LLVM/llvmlite (AOT/EXE harness)
- Out-of-scope: Cranelift JIT and Rust MIR Interpreter (maintenance only during Phase‑15)

## MIR14 Policy
- PHI-off at MIR: merges are represented by per-predecessor edge copies
- PHI synthesis is delegated to the LLVM harness (block-head PHIs, typed incoming)
- Verifier runs with `verify_allow_no_phi()` and edge-copy checks

## Parity & Evaluation
- Parity: PyVM vs LLVM output equivalence on curated smokes
- Metrics: runtime, startup time, memory; report only for PyVM/LLVM lines
- Tracing: `NYASH_LLVM_TRACE_PHI=1`, IR dump `NYASH_LLVM_DUMP_IR=...`

## Toggles
- `NYASH_MIR_NO_PHI=1` (default) — PHI disabled at MIR
- Dev-only PHI-on: build with `--features phi-legacy` and set `NYASH_MIR_NO_PHI=0`

## Notes
- Figures and text must avoid implying MIR-side PHI placement as default.
- Reference canonical specs live under `docs/reference/`.

