## MIR PHI Policy (Phase‑15)

Status
- Default: PHI‑off (edge‑copy mode). Builders/Bridge do not emit PHI; merges are realized via per‑predecessor Copy into the merge destination.
- Dev‑only: PHI‑on is experimental for targeted tests (enable with `NYASH_MIR_NO_PHI=0`).
- LLVM: PHI synthesis is delegated to the LLVM/llvmlite path (AOT/EXE). PyVM serves as the semantic reference.

Rationale
- Simplify MIR builders and JSON bridge by removing PHI placement decisions from the core path.
- Centralize SSA join formation at a single backend (LLVM harness), reducing maintenance and divergence.
- Keep PyVM parity by treating merges as value routing; short‑circuit semantics remain unchanged.

Operational Rules (PHI‑off)
- Edge‑copy only: predecessors write the merged destination via `Copy { dst=merged, src=pred_value }`.
- Merge block: must not emit a self‑Copy for the same destination; merged value is already defined by predecessors.
- Verifier: `verify_allow_no_phi()` is on by default; dominance and merge checks are relaxed in PHI‑off.

- Developer Notes (PHI‑on dev mode)
- Requires cargo feature `phi-legacy`. Build with `--features phi-legacy` and enable with `NYASH_MIR_NO_PHI=0`.
  Builders may place `Phi` at block heads with inputs covering all predecessors.
- Use `NYASH_LLVM_TRACE_PHI=1` for wiring trace; prefer small, isolated tests.

Backends
- LLVM harness performs PHI synthesis based on predecessor copies and dominance.
- Other backends (Cranelift/JIT) are secondary during Phase‑15; PHI synthesis there is not required.

Acceptance
- Default smokes/CI run with PHI‑off.
- Parity checks compare PyVM vs. LLVM AOT outputs; differences are resolved on the LLVM side when they stem from PHI formation.
