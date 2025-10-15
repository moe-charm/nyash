Quick/Selfhost — emit-only & pipeline_v2

Purpose
- Keep selfhost-related fast checks isolated from core language smokes.
- Ensure JSON header presence, minimal MIR(JSON) generation (const/binop/compare/branch/jump/ret), and LocalSSA materialization where applicable.

Conventions
- Always source lib/test_runner.sh and run require_env + preflight_plugins.
- Child pipeline:
  - Do not pass NYASH_QUIET to the child (avoid silent stdout).
  - Do pass NYASH_JSON_ONLY=1 for JSON-only acceptance.
- Trace (dev-only):
  - Parent ENV → child args mapping:
    - NYASH_EMIT_TRACE=1 → --emit-trace
    - NYASH_PREFER_CFG=1 → --prefer-cfg
    - NYASH_PREFER_CFG2=1 → --prefer-cfg2 (materialize copy)

Scope
- Minimal emit-only behaviors (header non-empty, branch/jump/ret generation).
- Pipeline V2 lowering path sanity checks.
- LocalSSA ensure_after_phis_copy representative cases.
- Front-end Ny→JSON v0 parity (VM/LLVM Result line compare for const/if/loop) — see `selfhost_front_min_vm_llvm.sh`.

Out-of-scope
- Heavy integration or LLVM parity (move to integration/parity or quick/llvm).
- Plugins-dependent flows (move to plugins suite).
