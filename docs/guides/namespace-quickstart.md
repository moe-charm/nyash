# Namespace Quickstart — Module‑First, 3‑Stage Resolution

This guide describes how Hakorune resolves module namespaces and how to opt into the simpler Module‑First policy.

## 3‑Stage Priority (simple and predictable)

1) [modules.workspace]
- List module manifests (hako_module.toml/module.toml) and expose their `[exports]`.
- Namespace = `[module].name` + `.` + export key (e.g., `selfhost.compiler.json_minify`).

2) [modules.overrides]
- Explicit per‑namespace path overrides. Wins over workspace when present.

3) Dir‑as‑NS (discovery)
- Development convenience. Converts file paths under `apps/` to namespaces.
- When Module‑First is ON, discovery fallback only strips `.hako` and optional `_box` suffix; no kebab→dot rewriting.

## Policy switch

- Env: `NYASH_NS_POLICY={path-first|module-first}` (default: `path-first`).
- Dev tip: set `NYASH_NS_POLICY=module-first` to prefer manifest‑based namespaces; keep discovery as a thin fallback.

## Migration: aliases → overrides

- `[modules.aliases]` is deprecated. Use `[modules.overrides]` to map a namespace explicitly to a file path.
- Example:

```toml
[modules.workspace]
members = [
  "apps/selfhost-compiler/hako_module.toml",
  "apps/selfhost/vm/hako_module.toml",
]

[modules.overrides]
selfhost.tools.dep_tree_core = "apps/selfhost/tools/dep_tree_core.hako"
```

## CLI helpers

- List all: `hakorune --list-modules`
- Show one: `hakorune --modules-show <namespace>`
  - The first line prints the active policy: `[policy] module-first` (or `path-first`).
- Resolve file: `hakorune --modules-resolve <file>`
  - Example first line: `[policy] module-first`

## Strict diagnostics

- With `NYASH_USING_CHECKS_STRICT=1`, resolver prints a single-line diagnostic and exits non-zero:
  - Missing dependency: `workspace missing dependency: <module> → <dep> (<req>)`
  - Conflict: `workspace namespace conflict: <ns> has multiple paths: <p1,p2,...>`
- JSON diagnostics are also emitted for tooling (`{"kind":"modules_error", ...}`), but the one-line string is stable for grepping.

## Recommended defaults

- Development: `NYASH_USING_DIR_NS=1`, `NYASH_NS_POLICY=module-first`
- CI/Production: `NYASH_USING_DIR_NS=0` and use `[modules.workspace]` + `[modules.overrides]` only.

## Minimal Example

1) Module manifest (`apps/selfhost-compiler/hako_module.toml`):

```toml
[module]
name = "selfhost.compiler"
version = "1.0.0"

[exports]
pipeline = "pipeline_v2/pipeline.hako"
using_resolver = "pipeline_v2/using_resolver_box.hako"
```

2) Workspace + override (`hako.toml` at repo root):

```toml
[modules.workspace]
members = [
  "apps/selfhost-compiler/hako_module.toml",
]

[modules.overrides]
selfhost.tools.dep_tree_core = "apps/selfhost/tools/dep_tree_core.hako"
```

3) Using in code:

```nyash
using selfhost.compiler.pipeline as Pipeline
```

## Mini‑VM Debugging (Dev)

- Enable trace printing for selfhost Mini‑VM by injecting a JSON flag:
  - Use wrapper: `MiniVmEntryBox.run_trace(json_text)` which adds `{"__trace__":1,...}` and runs.
  - Or manually add `"__trace__":1` at the head of the JSON object.
- Hakorune VM: use `__dev__=1` to enable DiagnosticsBox.debug; many smokes also accept `SMOKES_DEV_LOG=1` to show logs.
- Note: smokes filter some lines; final numeric result is still the last line the runner prints.
