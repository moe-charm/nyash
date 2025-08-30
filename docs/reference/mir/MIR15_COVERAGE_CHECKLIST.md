# MIR Core (~15) Coverage Checklist

Goal: Verify that the core MIR set executes correctly across VM and JIT (exe), then proceed to LLVM.

Target instructions (representative core):
- Basics: Const, UnaryOp, BinOp, Compare, TypeOp
- Memory: Load, Store
- Control: Branch, Jump, Return, Phi
- Box: NewBox, BoxCall, PluginInvoke
- Arrays: ArrayGet, ArraySet
- External: ExternCall

How to verify
- VM path
  - Run representative examples or unit snippets via `--backend vm`.
  - Enable VM stats for visibility: `NYASH_VM_STATS=1`

- JIT (compiler-only, exe emission where applicable)
  - Enable JIT compile path and hostcall: `NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1`
  - For PHI minimal path tests: `NYASH_JIT_PHI_MIN=1`
  - Optional DOT/trace: `NYASH_JIT_DUMP=1` and/or `NYASH_JIT_EVENTS_COMPILE=1`

Quick smoke
- Build: `cargo build --release --features cranelift-jit`
- Run: `tools/mir15_smoke.sh release`
- Policy: Core-1 is required green; hostcall-based cases are optional (fallback allowed).

Suggested minimal scenarios
- Const/Return: function returns 0/1/42.
- BinOp/Compare: arithmetic and boolean conditions.
- Branch/Jump/Phi: single-diamond if/else with merging value.
- Load/Store: local slot store → load (VM) and JIT local slots (lower/core) coverage.
- TypeOp: `is`/`as` via builder emits TypeOp.
- NewBox/BoxCall: call basic methods (e.g., StringBox.length, IntegerBox.get via PluginInvoke where applicable).
- PluginInvoke/by-name: `nyash.handle.of` + invoke name path.
- Arrays: len/get/set/push hostcalls (JIT: handle-based externs wired).
- ExternCall: `env.console.log`, `env.debug.trace`, `env.runtime.checkpoint`.

Notes
- Debug/Safepoint/Future/Await are rewritable via env toggles; core stays focused on the above.
- JIT direct path is read-only; mutating ops should fallback or be whitelisted accordingly.

Once coverage is green on VM and JIT, proceed to LLVM feature work (inkwell-backed) following docs in execution-backends.
