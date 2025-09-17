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
  - Relaxed (default): short names allowed when unique。
  - Strict: plugin短名にprefix必須（env `NYASH_PLUGIN_REQUIRE_PREFIX=1` または nyash.toml `[plugins] require_prefix=true`）。
- Aliases:
  - nyash.toml `[imports] HttpClient = "network.HttpClient"`
  - needs sugar: `needs plugin.network.HttpClient as HttpClient` (file‑scoped alias)

## Plugins
- Unified namespace with Boxes. Prefer short names when unique.
- Qualified form: `network.HttpClient`
- Per‑plugin control (nyash.toml): `prefix`, `require_prefix`, `expose_short_names`
  - 現状は設定の読み取りのみ（導線）。挙動への反映は段階的に実施予定。

## `needs` sugar (optional)
- Treated as a synonym to `using` on the Runner side; registers aliases only.
- Examples: `needs utils.StringHelper`, `needs plugin.network.HttpClient as HttpClient`, `needs plugin.network.*`

## nyash.toml keys (MVP)
- `[imports]`/`[aliases]`: short name → fully qualified
- `[plugins.<name>]`: `path`, `prefix`, `require_prefix`, `expose_short_names`

## Index and Cache (Runner)
- BoxIndex（グローバル）：プラグインBox一覧とaliasesを集約し、Runner起動時（plugins init後）に構築・更新。
  - `aliases: HashMap<String,String>`（nyash.toml `[aliases]` と env `NYASH_ALIASES`）
  - `plugin_boxes: HashSet<String>`（読み取り専用）
- 解決キャッシュ：グローバルの小さなキャッシュで同一キーの再解決を回避（キー: `tgt|base|strict|paths`）。
- トレース：`NYASH_RESOLVE_TRACE=1` で解決手順やキャッシュヒット、未解決候補を出力。

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
- CLI from-the-top registration: `--using "ns as Alias"` or `--using '"apps/foo.nyash" as Foo'` (repeatable)
- Strict mode (plugin prefix required): `NYASH_PLUGIN_REQUIRE_PREFIX=1` または `nyash.toml` の `[plugins] require_prefix=true`
- Aliases from env: `NYASH_ALIASES="Foo=apps/foo/main.nyash,Bar=lib/bar.nyash"`
- Additional search paths: `NYASH_USING_PATH="apps:lib:."`
- Selfhost pipeline keeps child stdout quiet and extracts JSON only: `NYASH_JSON_ONLY=1` (set by Runner automatically for child)
- Selfhost emits `meta.usings` automatically when present; no additional flags required.

Notes
- Phase 15 keeps resolution in the Runner to minimize parser complexity. Future phases may leverage `meta.usings` for compiler decisions.
- Unknown fields in the top‑level JSON (like `meta`) are ignored by the current bridge.
 - 未解決時（非strict）は実行を継続し、`NYASH_RESOLVE_TRACE=1` で候補を提示。strict時はエラーで候補を表示。

## Include/Export (Phase 1)

Simple include expression for file‑scoped modules（Phase 1 提案）。将来は `using`/Runner 解決へ統合予定。

Overview
- One file exports one static box. `include(path)` evaluates the file and returns that Box instance.

Syntax
```
local Math = include "lib/math.nyash"
local r = Math.add(1, 2)
```

Rules
- Single static box per file（0/複数はエラー）
- Expression form: `include(...)` は Box インスタンスを返す式
- Caching: 同一パスは一度だけ評価（2回目以降はキャッシュ返却）
- Path resolution（MVP）:
  - Relative allowed; absolute discouraged
  - nyash.toml `[include.roots]` で `std=/stdlib` 等のルート定義を許可
  - 省略拡張は `.nyash`、ディレクトリなら `index.nyash`

Backends
- Interpreter: 実行時に評価し Box を返す
- VM/AOT: MIR Builder が対象ファイルを読み取り、同一 MIR モジュールに static box を降ろす（専用 MIR 命令は追加しない）

Limitations
- 循環 include の検出/診断は未実装（後続で active-load 追跡と経路表示を追加）

Rationale
- MIR 仕様に変更を入れず、実用的なモジュール分割を提供
- Everything‑is‑Box に整合（モジュール=Box、メソッド/フィールド=API）
