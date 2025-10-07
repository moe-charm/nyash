Mini‑VM Debugging Guide

Scope
- Hakorune Mini‑VM boxes: `apps/hakorune/vm/boxes/hakorune_vm_min.hako` and legacy `apps/selfhost/vm/boxes/mir_vm_min.hako`.
- JSON v0 MIR strings executed via `run_min` or `MiniVmEntryBox.run_min`.

Quick Start
- Run a minimal JSON with the entry box:
  - `using selfhost.vm.entry as MiniVmEntryBox`
  - `local v = MiniVmEntryBox.run_min(json)` then `print(MiniVmEntryBox.int_to_str(v))`.

Trace Modes (Dev)
- Inline trace flag in JSON head:
  - Add `{"__trace__":1, ...rest...}` to enable `[DEBUG]` lines from Mini‑VM.
  - Helper: `MiniVmEntryBox.run_trace(json)` injects `__trace__=1` automatically.
- Thin mode for concise output:
  - Add `{"__thin__":1, ...}` or call `HakoruneVmMin.run_thin(json)` to skip non‑essential logging.

What You’ll See
- `[minivm] bb=… prev=… start=…` per basic block.
- `[minivm] op=const|copy|compare|branch|jump|phi|ret|throw` per instruction segment.
- Early shortcuts:
  - compare→ret fast path shows only the final numeric result (no intermediate noise).

Common Pitfalls
- Smokes filter diagnostic noise by default. Numeric result is the last line that matches `^-?[0-9]+$`.
  - See `tools/smokes/v2/lib/test_runner.sh: filter_noise()`.
- If your print is missing: ensure it is the last line and uses `int_to_str` helpers for integers.
- Multi‑compare in one block: fast paths used to mask the last compare. Current Mini‑VM avoids this; if you re‑introduce a fast path, guard against multi‑compare.

Handy Snippets
- Enable contracts/log noise filters in dev runs:
  - `NYASH_CHECK_CONTRACTS=1` (already default ON)
  - `NYASH_RESOLVE_TRACE=1` to trace `using` resolver (Runner side).
- Extract last numeric line from output:
  - `awk '/^[[:space:]]*-?[0-9]+[[:space:]]*$/{val=$0} END{gsub(/\r/,"",val); gsub(/^[[:space:]]+|[[:space:]]+$/ , "", val); print val}'`

E2E Examples
- Quick pipeline namespace test:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_namespace_with_usings_vm.sh`
- Mini‑VM basic:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_minivm_thin_vs_legacy_compare_ret_vm.sh`

Notes
- Dev‑only auto‑register (module‑first): when `NYASH_VM_AUTO_REGISTER_DIR_NS=1` and `NYASH_NS_POLICY=module-first`, the Runner also scans `apps/selfhost-compiler/builder/ssa` and `apps/selfhost-compiler/pipeline_v2` to make ModuleFunctions callable in Mini‑VM tests without manual `NYASH_MODULES`.
- For unknown ModuleFunction errors, the VM reports a single‑line message: `Unknown module function: Name (arity=N)`.
