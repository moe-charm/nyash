# hako.toml / nyash.toml — Configuration Reference (Phase 15.5)

Branding note: Preferred file name is `hako.toml`. For backward compatibility, `nyash.toml` (and `hakorune.toml`) are also accepted by the loader in the documented search order. Content/schema is identical.

Status: Proposed（受け口から段階導入。未指定時は現行既定を維持）

## 目的
- 依存関係・実行方針の**唯一の真実（SSOT）**。
- using の解決（AST プレリュード）と、将来の Provider/Type 分離（受け口）を一元管理。

## セクション一覧

### [env]
任意の既定環境変数（`HAKO_*` 推奨、互換で `NYASH_*` も可）。CI/ローカルで上書き可。

### [using]
検索ルート `paths = ["apps","lib","."]`、名前付きパッケージ `[using.<name>]`、エイリアス `[using.aliases]`。

例:
```toml
[using]
paths = ["apps", "lib", "."]

[using.json_native]
path = "apps/lib/json_native/"
main = "parser/parser.nyash"

[using.string_utils]
path = "apps/lib/json_native/utils/string.nyash"

[using.aliases]
json = "json_native"
StringUtils = "string_utils"
```

### Provider/Type（受け口・既定OFF）

Stable Type Name（STN）を Provider ID（PVN）にマッピング。未指定時は現行ランタイム既定。

```toml
[types.StringBox]
provider = "kernel:string@1.0"
interop  = "forbid"   # forbid|explicit|auto（既定: forbid）

[providers."kernel:string@1.0"]
crate = "nyash-plugin-base-string"   # 静的リンクのブートストラップ提供者

[providers."acme:string@2.1"]
path = "./plugins/libacme_string.so"
override = true

[policy]
factory = "plugin-first"    # plugin-first|compat_plugin_first|static_only
```

注意:
- 本仕様は「受け口」の段階。実行挙動は段階導入（Verify→Lock→実行）。
- 互換性重視のため、未指定時は現行と同じ既定にフォールバックする。

### [plugins.bootstrap] / [plugins.dynamic]（提案）
静的リンクのブートストラップ束／動的ロード（開発）を明示。

```toml
[plugins.bootstrap]
string  = { crate = "nyash-plugin-base-string", version = "2.3.0" }
integer = { crate = "nyash-plugin-base-integer", version = "1.5.1" }

[plugins.dynamic]
# string = { path = "./plugins/libnyash_string_plugin.so", override = true }
```

## Profiles（using / AST）
`NYASH_USING_PROFILE={dev|ci|prod}`

- dev/ci: AST プレリュード既定ON（file-usingはdevで許可、ciは警告/限定）
- prod: AST 既定OFF（toml 由来のみ、file-using はエラー）

実装ノート:
- AST 既定は `src/config/env.rs: using_ast_enabled()` でプロファイルに従い決定。
- 既存のレガシー前置きは prod で禁止、dev/ci でも段階的に削除予定。

## Verify（plugin-tester）
CI/起動前に最低限の契約を検査（例: String の `birth/fini/toUtf8/fromUtf8/equals/length/concat`）。欠落時は即停止。

## 参考
- Kernel/Plugin 方針: docs/reference/runtime/kernel-and-plugins.md
- ADR: docs/development/adr/adr-001-no-corebox-everything-is-plugin.md
