# Refactoring Candidates — Phase 15 (Mainline only)

Scope: PyVM/LLVM/Runner mainline. JIT/Cranelift is explicitly out of scope for this pass.

Goals
- Improve maintainability and locality of logic in the primary execution paths.
- Reduce ad-hoc env access; prefer `src/config/env.rs` helpers.
- Clarify control-flow lowering responsibilities in LLVM codegen.

High‑Value Candidates
- Runner CLI directives
  - Extract header‐comment directives scanning from `runner/mod.rs` → `runner/cli_directives.rs` (done).
  - Next: Move using/alias/env merge traces behind a `runner::trace` helper to keep `mod.rs` slim.

- LLVM codegen structure
  - Introduce `instructions/terminators.rs` (return/jump/branch emit glue) and `instructions/select.rs` (cond/short‑circuit pre‑normalize). Initially re‑export from `flow.rs`; later migrate implementations.
  - Keep `function.rs` focused on BB visitation + delegations (no heavy logic inline).

- VM dispatch integration
  - Gradually route the big `match` in VM through `backend/dispatch.rs` (already scaffolded). Start with side‑effect‑free ops (Const/Compare/TypeOp) to de‑risk.
  - Add opt‑in flag `NYASH_VM_USE_DISPATCH=1` to gate behavior during the transition.

- Env access centralization
  - Replace scattered `std::env::var("NYASH_*")` with `config::env` getters in hot paths (VM tracing, GC barriers, resolver toggles).
  - Add tiny wrappers where a tri‑state is needed (off/soft/on).

- MIR builder granularity (non‑breaking)
  - Extract loop helpers: `mir/builder/loops.rs` (headers/exits/latch snapshot utilities).
  - Extract phi helpers: `mir/builder/phi.rs` (if‑else merge, header normalize for latch).

Low‑Risk Cleanups
- Tools scripts: dedupe `tools/*smoke*.sh` into `tools/smokes/` with common helpers (env, timeout, exit filtering).
- Tests naming: prefer `*_test.rs` and `apps/tests/*.nyash` consistency for smokes.
- Logging: add `NYASH_CLI_VERBOSE` guards consistently; provide `runner::trace!(...)` macro for concise on/off.

Suggested Sequencing
1) Runner small extractions (directives/trace). Validate with existing smokes.
2) LLVM `terminators.rs`/`select.rs` scaffolding + staged migration; zero behavior change initially.
3) VM dispatch gating: land flag and migrate 3–5 simple opcodes; parity by unit tests.
4) MIR builder helpers: extract without API changes; run PyVM/LLVM curated smokes.

Notes
- Keep JIT/Cranelift untouched in this phase to avoid drift from the mainline policy.
- Prefer file‑level docs on new modules to guide incremental migration.

