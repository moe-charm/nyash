# Structured Exceptions via Result‑Mode (Draft)

Problem
- Language exceptions are hard to implement portably (unwinding, landing pads) and complicate SSA/CFG.

Contribution (Nyash)
- Express try/catch/cleanup with Result‑mode lowering (no MIR Throw/Catch): structured blocks and jumps.
- Thread‑local ThrowCtx to aggregate nested `throw` to a single catch.
- PHI‑Off: merge variables via edge‑copy; harness synthesizes PHIs.
- Block‑Postfix Catch syntax: `{ body } catch (e) { … } [cleanup { … }]` with single‑catch policy.

Method
- Parser (gated): accept try/throw + postfix catch, normalize to `ASTNode::TryCatch`.
- Bridge: set ThrowCtx on try entry; route `throw` to catch BB; bind catch param via incoming.
- Cleanup: always runs; merge at exit with PHI‑off rules.

Code References
- Parser: `src/parser/statements.rs`
- Bridge lowering: `src/runner/json_v0_bridge/lowering/{try_catch.rs, throw_ctx.rs}`
- Smokes (v2): covered by `tools/smokes/v2` integration runs

Evaluation Plan
- Semantic parity: PyVM vs. harness binaries on representative cases.
- Control‑flow complexity: nested if/loop + cleanup; ensure merges are stable.

Reproduce (v2)
- `tools/smokes/v2/run.sh --profile integration`
