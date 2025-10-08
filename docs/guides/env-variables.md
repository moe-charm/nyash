Nyash Environment Variables — Guide (Phase 15)

Overview
- Goal: keep behavior deterministic (CLI first), use env only for development/diagnostics and small switches.
- Policy:
  - Public/Stable: fixed defaults, documented; prefer CLI flags.
  - Dev/Diagnostics: default OFF; verbose-only logs; safe to ignore in prod.
  - Deprecated/Retired: warn once when used; plan removal.

Categories (selected)

1) Core behavior (Public/Stable)
- NYASH_USING: 1|0 — Enable using/namespace (default: 1)
- NYASH_CHECK_CONTRACTS: 1|0 — Unborn guard & contracts (default: 1)
- NYASH_ROOT: project root (aliases: HAKO_ROOT/HAKU_ROOT/HRN_ROOT → mapped to NYASH_ROOT)

2) VM diagnostics (Dev/Diagnostics; default OFF)
- NYASH_VM_TRACE, NYASH_VM_CALL_TRACE, NYASH_VM_CALL_ARG_TRACE, NYASH_VM_RET_TRACE,
  NYASH_VM_BRANCH_TRACE, NYASH_VM_PARAM_TRACE, NYASH_VM_RESOLVE_TRACE, NYASH_VM_TRACE_EXEC
- Limits: NYASH_VM_MAX_INSTRUCTIONS, NYASH_VM_MAX_BLOCK_EXEC
- Fallback knobs: NYASH_VM_TOLERATE_VOID, NYASH_VM_RECV_ARG_FALLBACK (avoid in prod)

3) LLVM line
- NYASH_LLVM_USE_HARNESS: 1 — Force Python llvmlite harness (integration/full default)
- NYASH_NY_LLVM_COMPILER, NYASH_EMIT_EXE_NYRT — harness/compiler paths (dev only)
- Diagnostics: NYASH_LLVM_DUMP_LL, NYASH_LLVM_LL_OUT, NYASH_LLVM_VERIFY (dev)

4) Macro
- NYASH_MACRO_ENABLE (default by profile), NYASH_MACRO_PATHS, NYASH_MACRO_STRICT, NYASH_MACRO_TRACE
- Deprecated (compat only): NYASH_MACRO_BOX_NY*, NYASH_MACRO_TOPLEVEL_ALLOW,
  NYASH_MACRO_BOX_CHILD_RUNNER (warn once; removal planned)

5) Plugins
- NYASH_DISABLE_PLUGINS, NYASH_PLUGIN_POLICY, NYASH_PLUGIN_ONLY, NYASH_PLUGIN_META,
  NYASH_PLUGIN_ABI_FINAL, NYASH_PLUGIN_CAPS_ENFORCE (dev/CI knobs; prefer nyash.toml)

6) Misc
- NYASH_CLI_VERBOSE — Verbose CLI logging (dev)
- NYASH_SCRIPT_ARGS_JSON — Inject argv for tests (dev)
- NYASH_STR_CP — String length/index as code points (dev)

Deprecated / Retired
- NYASH_VM_CALL_ADAPTER — Removed (use MirInstruction::Call + Callee::{ModuleFunction,ExternCall})
- NYASH_ENABLE_USING — Use NYASH_USING instead (warn once)
- NYASH_VM_BIRTH_AFTER_NEW — Deprecated (default OFF). Builder/Bridge emits birth(); enable only for legacy debugging.
- HAKO_USING — Use NYASH_USING instead (warn once; alias copied when unset)
- NYASH_MACRO_* legacy set — Warn once; prefer NYASH_MACRO_PATHS/ENABLE/STRICT.

CLI first
- Prefer CLI flags or profiles for public behavior. Env variables are for development, diagnostics, and temporary gates.
  Example: --backend llvm and profile env overlays for smokes; avoid mixing multiple envs that shift semantics.

Removal plan
- Show one-line deprecation when verbose; keep default OFF; remove code path after one release.

Equality & Operators (Box)
- NYASH_BUILDER_BOX_EQ_TO_EQUALS — Retired
  - Replaced by unified `nyrt.ops.op_eq` path at MIR via MirCall::Extern.
  - Rationale: single entry guarantees VM/LLVM parity; op_eq internally invokes `.equals/1` when present.

Using/Entry defaults
- NYASH_USING — default 1 (enabled). Prefer this over deprecated NYASH_ENABLE_USING.
- NYASH_USING_AST — default 0. Dev/test overlay may set 1 for stability in nested alias tests. Planned retirement.
- NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN — default ON (unset = true). Aliases `HAKO_ENTRY_ALLOW_TOPLEVEL_MAIN` etc. are mapped to NYASH_ prefixes automatically.

Call Unification
- MirCall only (see docs/reference/mir/call-unified.md). Legacy ExternCall has been removed.
