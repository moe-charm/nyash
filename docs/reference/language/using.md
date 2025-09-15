# using — Imports and Namespaces (Phase 15+)

Status: Accepted (Runner‑side resolution). Selfhost parser accepts using as no‑op and attaches `meta.usings` for future use.

Policy
- Accept `using` lines at the top of the file to declare module namespaces or file imports.
- Resolution is performed by the Rust Runner when `NYASH_ENABLE_USING=1`.
  - Runner strips `using` lines from the source before parsing/execution.
  - Registers modules into an internal registry for path/namespace hints.
- Selfhost compiler (Ny→JSON v0) collects using lines and emits `meta.usings` when present. The bridge currently ignores this meta field.

## Namespace Resolution (Runner‑side)
- Goal: keep IR/VM/JIT untouched. All resolution happens in Runner/Registry.
- Default search order (3 stages, deterministic):
  1) Local/Core Boxes (nyrt)
  2) Aliases (nyash.toml [imports] / `needs … as …`)
  3) Plugins (short name if unique, otherwise qualified `pluginName.BoxName`)
- On ambiguity: error with candidates and remediation (qualify or define alias).
- Modes:
  - Relaxed (default): short names allowed when unique.
  - Strict: require plugin prefix (env `NYASH_PLUGIN_REQUIRE_PREFIX=1` or nyash.toml `[plugins] require_prefix=true`).
- Aliases:
  - nyash.toml `[imports] HttpClient = "network.HttpClient"`
  - needs sugar: `needs plugin.network.HttpClient as HttpClient` (file‑scoped alias)

## Plugins
- Unified namespace with Boxes. Prefer short names when unique.
- Qualified form: `network.HttpClient`
- Per‑plugin control (nyash.toml): `prefix`, `require_prefix`, `expose_short_names`

## `needs` sugar (optional)
- Treated as a synonym to `using` on the Runner side; registers aliases only.
- Examples: `needs utils.StringHelper`, `needs plugin.network.HttpClient as HttpClient`, `needs plugin.network.*`

## nyash.toml keys (MVP)
- `[imports]`/`[aliases]`: short name → fully qualified
- `[plugins.<name>]`: `path`, `prefix`, `require_prefix`, `expose_short_names`

## Index and Cache (Runner)
- Build a Box index once per run to make resolution fast and predictable:
```
struct BoxIndex {
    local_boxes: HashMap<String, PathBuf>,
    plugin_boxes: HashMap<String, Vec<PluginBox>>,
    aliases: HashMap<String, String>,
}
```
- Maintain a small resolve cache per thread:
```
thread_local! {
    static RESOLVE_CACHE: RefCell<HashMap<String, ResolvedBox>> = /* ... */;
}
```
- Trace: `NYASH_RESOLVE_TRACE=1` prints resolution steps (for debugging/CI logs).

Syntax
- Namespace: `using core.std` or `using core.std as Std`
- File path: `using "apps/examples/string_p0.nyash" as Strings`
- Relative path is allowed; absolute paths are discouraged.

Style
- Place all `using` lines at the top of the file, before any code.
- One using per line; avoid trailing semicolons. Newline separation is preferred.
- Order: sort alphabetically by target. Group namespaces before file paths.
- Prefer an explicit alias (`as ...`) when the target is long. Suggested alias style is `PascalCase` (e.g., `Std`, `Json`, `UI`).

Examples
```nyash
using core.std as Std
using "apps/examples/string_p0.nyash" as Strings

static box Main {
  main(args) {
    local console = new ConsoleBox()
    console.println("hello")
    return 0
  }
}
```

Qualified/Plugins/Aliases examples
```nyash
# nyash.toml
[plugins.network]
path = "plugins/network.so"
prefix = "network"
require_prefix = false

[imports]
HttpClient = "network.HttpClient"

# code
needs plugin.network.HttpClient as HttpClient

static box Main {
  main(args) {
    let a = new HttpClient()         # alias
    let b = new network.HttpClient() # qualified
  }
}
```

Runner Configuration
- Enable using pre‑processing: `NYASH_ENABLE_USING=1`
- Selfhost pipeline keeps child stdout quiet and extracts JSON only: `NYASH_JSON_ONLY=1` (set by Runner automatically for child)
- Selfhost emits `meta.usings` automatically when present; no additional flags required.

Notes
- Phase 15 keeps resolution in the Runner to minimize parser complexity. Future phases may leverage `meta.usings` for compiler decisions.
- Unknown fields in the top‑level JSON (like `meta`) are ignored by the current bridge.
