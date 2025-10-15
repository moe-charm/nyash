# Using System — Modules, AST Prelude, Aliases (Phase 15+)

Purpose
- Make `using` Just Work in dev without juggling many env flags.
- Explain the resolution flow, new `HAKO_USING` modes, and quick diagnostics.

Quick Start (dev)
- Defaults: development-friendly. No env required.
- Minimal example:
  ```nyash
  using "selfhost.shared.mir.builder" as BlockBuilderBox;
  static box Main { main(args){ print(BlockBuilderBox.const_ret_ops(42)); return 0; } }
  ```
- Recommended pattern: prefer quoted module using with module names registered in:
  - `hako.toml` → `[modules]`
  - workspace `hako_module.toml` → `[exports]`

Modes (env)
- `HAKO_USING` (primary; overrides legacy `NYASH_*` flags when set)
  - `full` (default): using ON + AST prelude merge ON + file-using allowed
  - `basic`: using ON, AST merge OFF, file-using OFF
  - `off`/`0`: using OFF
- Legacy (compat): `NYASH_USING`, `NYASH_USING_AST`, `NYASH_ALLOW_USING_FILE`
  - When `HAKO_USING` is present, it takes precedence.

Resolution Flow (dev path)
1) resolve_prelude_paths_profiled: collect module/file preludes + alias pairs
2) parse_preludes_to_asts: read/parse preludes (when AST is enabled)
3) merge_prelude_asts_with_main: prelude AST + main AST → Program
4) alias_desugar: rewrite `Alias.X` → `Alias_X` and static calls → `Alias_Box.method/arity`

Diagnostics (what to look for)
- Enable trace: `NYASH_RESOLVE_TRACE=1`
  - `[using/trace] preludes: <N> alias_pairs: <M>`
  - `[using/trace] prelude: <path>` (each file)
  - `[using/trace] parsed preludes: <K>`
- Self‑check (auto):
  - `[using/selfcheck] modules resolved but no prelude paths collected ...`
    - Causes: AST merge disabled / file using disabled / resolved to dylib
    - Quick fix: `HAKO_USING=full` (or set `NYASH_USING_AST=1`, `HAKO_ALLOW_USING_FILE=1`)
- Undefined variable diagnostics (builder):
  - Shows Using enabled / AST merge / File using flags and quick fixes

Best Practices
- Prefer module names (quoted) over direct file paths, e.g. `using "ns.module" as Alias;`
- Register modules in `hako.toml [modules]` or workspace `hako_module.toml [exports]`.
- Treat `NYASH_MODULES` as dev‑only and temporary; migrate entries into `hako.toml`.
- Keep AST merge ON in development (`HAKO_USING=full`), OFF or `basic` in production if you want stricter behavior.

Migration (NYASH_MODULES → modules)
- Before (dev only):
  ```bash
  export NYASH_MODULES="ns.mod=path/to/mod.hako"
  ```
- After (preferred): `hako.toml`
  ```toml
  [modules]
  ns.mod = "path/to/mod.hako"
  ```
- Or workspace export (`hako_module.toml`)
  ```toml
  [module]
  name = "ns"
  version = "1.0.0"
  [exports]
  mod = "path/to/mod.hako"
  ```

FAQ
- Q: Do I still need HAKO_ALLOW_USING_FILE?
  - A: No for dev; `HAKO_USING=full`（既定）で許可済み。prodで禁止したい場合のみ `basic`/`off` を選択。
- Q: Why quoted module using?
  - A: It unambiguously signals a module name. The resolver will map it via `[modules]`/workspace exports before any file fallback.

References
- Language: `docs/reference/language/using.md`
- Env: `docs/guides/env-variables.md`
- Smokes: `docs/guides/smokes-profiles.md`
