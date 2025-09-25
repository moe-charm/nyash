# using — Imports and Namespaces (Phase 15+)

**実装状況**: Phase 15.5後に本格実装予定 | 基本ドット記法は実装済み

Status: Accepted (Runner‑side resolution). Selfhost parser accepts using as no‑op and attaches `meta.usings` for future use.

> Phase 15.5 指針（いいとこ取り）
> - 依存の唯一の真実（SSOT）: `nyash.toml` の `[using]`（aliases/packages/paths）
> - 実体の合成: テキスト結合は廃止し、AST マージに一本化（曖昧さ根絶）
> - プロファイル運用: `NYASH_USING_PROFILE={dev|ci|prod}` で厳格度を段階的に切替
>   - dev: toml + ファイル内 using を許可（実験/便利）
>   - ci: toml 優先、ファイル using は警告または限定許可
>   - prod: toml のみ。ファイル using/path はエラー（追記ガイドを提示）

## 🎯 設計思想：Everything has Namespace

### **核心コンセプト**
すべてのBox、関数、メンバーが明確な名前空間を持ち、衝突・曖昧性を根本解決。

```nyash
// ✅ 実装済み：ドット記法
network.HttpClient()         // プラグイン修飾名
plugin.network.HttpClient() // フルパス

// 🚧 Phase 15.5後：明示的スコープ演算子
::print("global")           // グローバルスコープ
builtin::StringBox("test")  // 内蔵版明示
plugin::StringBox("test")   // プラグイン版明示
```

### **MIR Callee革新との統合**
[MIR Callee革新設計](../../development/architecture/mir-callee-revolution.md)と完全統合：

```rust
// Phase 1: 型安全関数呼び出し（実装済み）
pub enum Callee {
    Global(String),          // ::print, global::func
    Method { box_name, method, receiver }, // obj.method()
    Extern(String),          // nyash.console.log
    Value(ValueId),          // 第一級関数
}

// Phase 3: 完全修飾名対応（Phase 15.5後）
pub enum QualifiedCallee {
    Qualified { namespace: Vec<String>, name: String },
    Scoped { scope: ScopeKind, name: String },
}
```

## 📊 実装状況

### ✅ **現在実装済み**
- **ドット記法**: `plugin.BoxName`、`namespace.member`
- **using基本構文**: ファイルトップでの宣言
- **エイリアス**: `using long.path as Alias`
- **プラグイン修飾**: `network.HttpClient`

### 🚧 **Phase 15.5後実装予定**
- **built-in namespace**: `builtin.StringBox` vs `plugin.StringBox`
- **完全修飾名**: `nyash.builtin.print`、`std.console.log`
- **スコープ演算子**: `::global_func`、`Type::static_method`
- **厳密解決**: コンパイル時名前空間検証

Policy
- Accept `using` lines at the top of the file to declare module namespaces or file imports.
- Resolution is performed by the Rust Runner when `NYASH_ENABLE_USING=1`.
- 実体の結合は AST マージのみ。テキストの前置き/連結は行わない（レガシー経路は呼び出し側から削除済み）。
- Runner は `nyash.toml` の `[using]` を唯一の真実として参照（prod）。dev/ci は段階的に緩和可能。
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

## nyash.toml — Unified Using（唯一の真実 / SSOT）

Using resolution is centralized under the `[using]` table. Three forms are supported:

- `[using.paths]` — additional search roots for path lookups
  - Example: `paths = ["apps", "lib", "."]`
- `[using.<name>]` — named packages (file or directory)
  - Keys: `path = "lib/math_utils/"`, optional `main = "math_utils.nyash"`
  - Optional `kind = "dylib"` with `bid = "MathBox"` for plug‑ins (dev only)
- `[using.aliases]` — alias mapping from short name to a package name
  - Example: `aliases.json = "json_native"`

Notes
- Aliases are fully resolved: `using json` first rewrites to `json_native`, then resolves to a concrete path via `[using.json_native]`.
- `include` は廃止。代替は `using "./path/to/file.nyash" as Name`。prod では `nyash.toml` への登録が必須。

### Dylib autoload (dev guard)
- Enable autoload during using resolution: set env `NYASH_USING_DYLIB_AUTOLOAD=1`.
- Resolution returns a token `dylib:<path>`; when autoload is on, Runner calls the plugin host to `load_library_direct(lib_name, path, boxes)`.
- `boxes` is taken from `[using.<name>].bid` if present; otherwise the loader falls back to plugin‑embedded TypeBox metadata.
- Safety: keep OFF by default. Prefer configuring libraries under `nyash.toml` for production.

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

nyash.toml examples
```toml
[using]
paths = ["apps", "lib", "."]

[using.json_native]
path = "apps/lib/json_native/"
main = "parser.nyash"

[using.aliases]
json = "json_native"

# Dylib (dev)
[using.math_plugin]
kind = "dylib"
path = "plugins/math/libmath.so"
bid = "MathBox"
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
- Using profiles (phase‑in): `NYASH_USING_PROFILE={dev|ci|prod}`
  - dev: AST マージ 既定ON、legacy前置きは既定で無効（必要時は `NYASH_LEGACY_USING_ALLOW=1` で一時許可）
  - ci: AST マージ 既定ON、legacy前置きは既定で無効（同上の一時許可）
  - prod: AST マージ 既定OFF、toml のみ（file using/path はエラー・追記ガイド）
- Strict mode (plugin prefix required): `NYASH_PLUGIN_REQUIRE_PREFIX=1` または `nyash.toml` の `[plugins] require_prefix=true`
- Aliases from env: `NYASH_ALIASES="Foo=apps/foo/main.nyash,Bar=lib/bar.nyash"`
- Additional search paths: `NYASH_USING_PATH="apps:lib:."`
- Selfhost pipeline keeps child stdout quiet and extracts JSON only: `NYASH_JSON_ONLY=1` (set by Runner automatically for child)
- Selfhost emits `meta.usings` automatically when present; no additional flags required.

Note: Provider/Type 分離（型名は不変で提供者のみを切替）については ADR を参照。  
docs/development/adr/adr-001-no-corebox-everything-is-plugin.md

## 🔬 Quick Smokes（AST + Profiles）

開発・CIで最小コストに確認できるスモークを用意しています。AST プレリュードとプロファイル（dev/prod）の基本動作をカバーします。

- dev: `using "file"` 許可 + AST マージ
- prod: `using "file"` 禁止（toml へ誘導） / alias・package は許可

実行例（quick プロファイル）

```
# 1) dev で file using が通る（AST マージ）
./tools/smokes/v2/run.sh --profile quick --filter "using_profiles_ast.sh$"

# 2) 相対パス using（サブディレクトリ）
./tools/smokes/v2/run.sh --profile quick --filter "using_relative_file_ast.sh$"

# 3) 複数プレリュード（toml packages）+ 依存（B→A）
./tools/smokes/v2/run.sh --profile quick --filter "using_multi_prelude_dep_ast.sh$"
```

テストソース
- `tools/smokes/v2/profiles/quick/core/using_profiles_ast.sh`
- `tools/smokes/v2/profiles/quick/core/using_relative_file_ast.sh`
- `tools/smokes/v2/profiles/quick/core/using_multi_prelude_dep_ast.sh`

注意
- ログに `[using] stripped line:` が出力されますが、これは AST マージ前の using 行の除去ログです（機能上問題ありません）。
- 実行バイナリは `target/release/nyash` を前提とします。未ビルド時は `cargo build --release` を実行してください。

## 🔗 関連ドキュメント

### **設計・アーキテクチャ**
- [MIR Callee革新設計](../../development/architecture/mir-callee-revolution.md) - 型安全関数呼び出し
- [Phase 15.5 Core Box統一](../../development/roadmap/phases/phase-15.5/README.md) - プラグイン統一計画
- [Box Factory設計](../../reference/architecture/box-factory-design.md) - builtin vs plugin優先順位

### **実装ガイド**
- [Callee実装ロードマップ](../../development/roadmap/phases/phase-15/mir-callee-implementation-roadmap.md)
- [プラグインシステム](../../reference/plugin-system/) - プラグイン開発ガイド
- [完全言語リファレンス](../LANGUAGE_REFERENCE_2025.md) - 全構文仕様

## 📝 実装ノート

Notes
- Phase 15 keeps resolution in the Runner to minimize parser complexity. Future phases may leverage `meta.usings` for compiler decisions.
- レガシー実装の扱い: テキスト前置き/括弧補正などのシムは段階的に削除（prod プロファイルから先に無効化）。
- AST マージは dev/ci/prod の全プロファイルで共通基盤とし、曖昧性（宣言≻式）問題の再発を原理的に回避する。
- Unknown fields in the top‑level JSON (like `meta`) are ignored by the current bridge.
- 未解決時（非strict）は実行を継続し、`NYASH_RESOLVE_TRACE=1` で候補を提示。strict時はエラーで候補を表示。
- **Phase 15.5完了により、現代的な名前空間システムを実現予定**

## Deprecated: Include/Export（廃止）

このセクションは移行期の参考情報です。`include` は設計上の一貫性と学習コスト低減のため廃止しました。今後はすべて `using` に一本化してください（ファイル・パッケージ・DLL すべてを `using` で扱えます）。既存コードの移行は以下の対応例を推奨します。

- `local M = include "./path/module.nyash"` → `using "./path/module.nyash" as M`
- `include` の探索ルートは `[using.paths]` に統合（`nyash.toml`）

注: `include` は完全に非推奨です。コードは `using` に書き換えてください（互換シムは提供しません）。

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
