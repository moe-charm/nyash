# PHI Observability and Trace Checking (Draft)

Motivation
- SSA construction bugs are subtle. We want a first‑class, machine‑checkable record of PHI decisions.

Approach
- Emit structured JSONL events across PHI lifecycle: `finalize_begin`, `finalize_dst`, `finalize_target`, `wire_choose`, `add_incoming`, `finalize_summary`.
- Provide a checker that validates coverage and invariants (e.g., each dst has at least one add_incoming; chosen preds ⊆ CFG preds).

Code References
- Trace writer: `src/llvm_py/llvm_builder.py`, `src/llvm_py/phi_wiring/wiring.py`
- Checker: `tools/phi_trace_check.py`

Usage
- `NYASH_LLVM_TRACE_PHI=1 NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl bash tools/test/smoke/bridge/try_result_mode.sh`
- `python3 tools/phi_trace_check.py --summary tmp/phi.jsonl`

Next
- Expand checks (dominance, grouping at head), integrate into CI as optional gate.

