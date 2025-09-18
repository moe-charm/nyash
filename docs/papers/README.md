# Nyash Research Notes (Draft)

This folder collects short, structured notes toward potential publications based on the Nyash Phase‑15 work. Each note is self‑contained and links to code paths and smokes for reproducibility.

Topics
- 01 — PHI‑Off Edge‑Copy + Harness PHI Synthesis (phi_off_harness.md)
- 02 — Structured Exceptions via Result‑Mode (result_mode_exceptions.md)
- 03 — PHI Observability and Trace Checking (phi_trace_observability.md)
- 04 — Block‑Postfix Catch Language Design (block_postfix_catch.md)

How to Reproduce (quick)
- Build: `cargo build --release`
- Bridge(Result‑mode) smokes: `bash tools/test/smoke/bridge/try_result_mode.sh`
- PHI trace (optional): `NYASH_LLVM_TRACE_PHI=1 NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl bash tools/test/smoke/bridge/try_result_mode.sh`

Notes
- Target audience: systems and compiler practitioners; emphasis on simplicity, robustness, and observability.
- Draft status: living documents; code references use stable paths in this repo.

