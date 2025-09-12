# llvmlite Harness (Experimental)

Purpose
- Provide a fast, scriptable LLVM emission path using Python + llvmlite for validation and prototyping.
- Run in parallel with the Rust/inkwell path; keep outputs functionally equivalent for targeted smokes.

Switch
- Set `NYASH_LLVM_USE_HARNESS=1` to prefer the harness (future: wired in LLVM backend entry).

Protocol (tentative)
- Input: MIR14 JSON file path (subset sufficient for dep_tree_min_string initially).
- Output: `.o` object file written to `NYASH_AOT_OBJECT_OUT` or `--out` path.
- Entry function: `ny_main(i64 argc, i8** argv) -> i64` (returns app exit code/box-handle per ABI).

Quick Start
- Install deps: `python3 -m pip install llvmlite`
- Generate a dummy object to validate toolchain:
  - `python3 tools/llvmlite_harness.py --out /tmp/dummy.o`
  - Link with NyRT as usual to produce an executable.

Intended Wiring (Rust side)
- LLVM backend checks `NYASH_LLVM_USE_HARNESS=1` and, if set, exports MIR14 of the target module to a temp JSON, then invokes:
  - `python3 tools/llvmlite_harness.py --in <mir.json> --out <obj.o>`
- On success, the normal link step continues using `<obj.o>`.

Scope (Phase 15)
- Minimal ops: i64 arithmetic, comparisons, branches, PHI(Sealed), basic string ops through NyRT shims.
- Target case: `apps/selfhost/tools/dep_tree_min_string.nyash` builds and runs.

Acceptance
- A5: Harness ON vs OFF produce functionally equivalent output for the target smoke.

Notes
- The first version may ignore MIR details and emit a fixed `ny_main` body for smoke scaffolding; then iterate to lower MIR ops.
- Keep the harness self-contained; no external state besides inputs and env.

