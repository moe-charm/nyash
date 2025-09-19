# PHI‑Off Edge‑Copy + Harness PHI Synthesis (Draft)

Problem
- Frontend/Builder PHI construction intertwines with optimization order and dominance; fragile in multi‑backend settings.

Contribution (Nyash)
- Keep builders PHI‑off using Edge‑Copy policy (pred→copy→merge), and synthesize PHIs late in the harness.
- Constrain PHI placement: always at block head (grouped) via `phi_at_block_head`/`ensure_phi`.
- Make PHI wiring observable by JSONL traces + checker.

Method
- IR discipline: JSON v0 Bridge emits no PHIs; records edge‑end values per block.
- Harness (`finalize_phis`) resolves `(decl_b, src_vid)` pairs to the nearest predecessor on CFG paths and wires incoming at block heads.
- Self‑carry handling: prefer a non‑self initial source if present.
- Helpers: `phi_at_block_head`, `ensure_phi`, `wire_incomings`.

Code References
- Bridge edge‑copy: `src/runner/json_v0_bridge/lowering/*`
- Harness PHI: `src/llvm_py/phi_wiring/wiring.py`, `src/llvm_py/llvm_builder.py`
- Head placement helper: `phi_at_block_head()`

Evaluation Plan
- Correctness: run curated smokes with complex mixes (loops, ifs, returns, exceptions) and diff traces.
- Robustness: mutate block orders; verify PHIs remain grouped at heads and traces stable.
- Cost: count PHIs vs. classic SSA on samples (optional).

Reproduce
- `cargo build --release`
- `bash tools/test/smoke/bridge/try_result_mode.sh`
- Optional trace: `NYASH_LLVM_TRACE_PHI=1 NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl ...`

