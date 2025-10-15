Using Strict — Fail‑Fast Policy for `using`

Overview
- Goal: eliminate “silent unresolved using” by failing fast when a `using` target or alias cannot be resolved.
- Default: ON. Disable explicitly with `NYASH_USING_STRICT=0|false|off`.

Resolution Sources
- nyash.toml / hako.toml `[using]` and `[modules]` entries (packages, aliases, module paths)
- Env overlays: `NYASH_MODULES`, `NYASH_USING_PATH`, `NYASH_ALIASES`
- Runner collects these into a context and resolves `using` targets accordingly.

Behavior (strict = ON)
- Alias or module name not found:
  - Runner returns error and prints: `❌ using: unresolved using '<name>' ...`
  - Process exits with non-zero status (pipeline stop).
- AST-prelude disabled when using present:
  - Prints: `❌ Pipeline error: \`using\` resolution error: AST prelude merge is disabled ...`

Behavior (strict = OFF)
- Unresolved targets are logged (debug) and left as-is for compatibility in dev-only scenarios.

Config
- `NYASH_USING_STRICT=1|true|on` → strict enabled (default)
- `NYASH_USING_STRICT=0|false|off` → strict disabled
- `NYASH_USING_AST=1` → enable AST prelude merge; otherwise using preludes are not merged

Compiler (Selfhost) Notes
- `UsingResolverBox` + `NamespaceBox` normalize names before emit. When an alias is missing, they print `[ERROR] Unresolved using alias: <head>` and abort the path.
- This compiler-side fail-fast complements the runner-side strict policy.

Smokes
- `tools/smokes/v2/profiles/quick/core/using_missing_strict_vm.sh` — unresolved using with strict ON must fail.

Updates (2025‑10‑06)
- Env access is centralized in `src/config/env.rs` (using_strict(), resolve_trace(), resolve_trace_json(), import_trace()).
- Call‑side arity checks apply independently; strict using focuses solely on resolution, not method signatures.
