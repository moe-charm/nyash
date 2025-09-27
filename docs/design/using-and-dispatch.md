# Using Resolution and Runtime Dispatch — Design Overview (SSOT + AST, Phase‑15)

Purpose
- Make name/module resolution deterministic at build time while keeping runtime dispatch only for plugin/extern paths.
- Preserve the language surface (`obj.method()`) and guarantee it executes in prod without runtime fallbacks.

Key Principles
- Single Source of Truth (SSOT): `nyash.toml` governs using packages/aliases.
- Profiles: dev|ci|prod
  - prod: file‑using is rejected; only toml packages/aliases are allowed; AST prelude merge is required when using is present.
  - dev/ci: file‑using can be enabled via env; AST prelude merge on by default.
- Static resolution at using time:
  - Strip `using` lines, collect prelude source paths, parse each to AST, and merge before macro expansion.
  - Materialize user box methods as standalone functions: `Box.method/Arity`.
  - Builder rewrites instance calls: `obj.m(a)` → call `Box.m/Arity(obj,a)`.
- Runtime resolution:
  - Plugin/extern dispatch remains dynamic.
  - User instance BoxCall fallback (VM) is disallowed in prod to catch builder misses; dev/ci may allow with WARN.

Environment Knobs (Summary)
- Using system
  - `NYASH_ENABLE_USING` (default ON)
  - `NYASH_USING_PROFILE={dev|ci|prod}` (default dev)
  - `NYASH_USING_AST=1|0` (dev/ci default ON; prod default OFF)
  - `NYASH_ALLOW_USING_FILE=1|0` (default OFF; dev only when needed)
- Builder
  - `NYASH_BUILDER_REWRITE_INSTANCE=1|0` (default ON across profiles)
- VM
  - `NYASH_VM_USER_INSTANCE_BOXCALL=1|0` (default: prod=0, dev/ci=1)

Code Responsibilities
- Using resolution (static)
  - Entry: `src/runner/modes/common_util/resolve/resolve_prelude_paths_profiled`
  - Core: `collect_using_and_strip` (SSOT enforcement, path/alias/package resolution, profile gates)
  - Consumers: all runners (VM, LLVM/harness, PyVM, selfhost) call the same helper to avoid drift.
- Builder (lowering)
  - Instance→Function rewrite and method materialization live in `src/mir/builder/*`.
- VM (runtime)
  - User Instance BoxCall fallback gate in `src/backend/mir_interpreter/handlers/boxes.rs`.

Testing Strategy
- Prod safety: obj.method() works under prod with runtime fallback disabled (rewrite required).
  - Smoke: `tools/smokes/v2/profiles/quick/core/oop_instance_call_vm.sh` (prod + forbid VM fallback)
- Using/AST parity: quick/integration call `resolve_prelude_paths_profiled` from all runner modes.

Future Small Refactors (non‑behavioral)
- Factor a helper to parse prelude paths into ASTs (single place), and use it from all runners.
- Add dev‑only WARN when a candidate `Box.method/Arity` is missing from `module.functions` during rewrite.

