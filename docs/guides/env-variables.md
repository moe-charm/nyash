# ENV Variables — Core (Plugins/Provider)

Key variables (current)
- `HAKO_PLUGIN_POLICY` (auto|off|force) — primary plugin load policy (preferred)
- `NYASH_PLUGIN_CONFIG` — plugin config path (prefer `hako.toml`)
- `NYASH_PLUGIN_MAP_ARRAY_HANDLE` — Stage‑2: 1 で keys/values HostHandle 経路を有効化。0 で Stage‑1(keysS/valuesS) シム（plugins プロファイルは既定ON）。
- `NYASH_MAP_FORCE_HOST` — Dev/Test: Map.size/has/get/set を HostHandleRouter の slot(200/202/203/204) へ強制。既定OFF（plugins プロファイルはON）。
- `NYASH_ARRAY_FORCE_HOST` — Dev/Test: Array.size/get/set を HostHandleRouter の slot(102/100/101) へ強制。既定OFF（plugins プロファイルはON）。
- `NYASH_ARRAY_SIZE_FORCE_HOST` — Dev/Test: Array.size を HostHandleRouter の slot(102) へ強制（互換）。
- `NYASH_STRING_SIZE_FORCE_HOST` — Dev/Test: String.size/len を HostHandleRouter の slot(300) へ強制。既定OFF（plugins プロファイルはON）。
- `HAKO_HOST_HANDLE_TRACE` / `NYASH_HOST_HANDLE_TRACE` — HostHandle slot呼び出しの観測ログ（短命/既定OFF）
- `HAKO_MIRIO_PROVIDER` (scan|yyjson) — MirIoBox の入力プロバイダー選択（既定=scan）。yyjson は JSON プラグイン配置が必要。
  - 互換: `NYASH_JSON_PROVIDER`（legacy）。同時指定時は `HAKO_MIRIO_PROVIDER` を優先。
- `HAKO_ALLOW_USING_FILE` — using でファイルパス参照を許可（開発/スモーク用）。
- `NYASH_USING_AST` — using prelude の AST マージを有効化（開発/スモーク用）。

Deprecated (compat) — avoid in new scripts
- `NYASH_USE_PLUGIN_BUILTINS` — superseded by `HAKO_PLUGIN_POLICY`
- `NYASH_PLUGIN_OVERRIDE_TYPES` — superseded by `HAKO_PLUGIN_POLICY`
- `NYASH_BUILTIN_DISABLE_{STRING|ARRAY|MAP}` — superseded by `HAKO_PLUGIN_POLICY`

Profiles
- plugin‑on: sets `HAKO_PLUGIN_POLICY=auto`, `NYASH_PLUGIN_CONFIG=hako.toml`
- plugins: Stage‑2 HostHandle 既定ON（`NYASH_PLUGIN_MAP_ARRAY_HANDLE=1`）＋ HostHandleRouter 経路を優先（Map/Array/String の強制ENVをON）
- quick: 段階導入（最小）— `NYASH_ARRAY_SIZE_FORCE_HOST=1` のみ既定ON。他は必要時に opt‑in。

Birth Adoption
- VM will call `birth()` when a plugin box is created with `instance_id=0`, and adopt the returned handle.
- No‑op when `birth` does not exist.

Notes
- Prefer CLI/Profiles over ENV when possible; ENV should be minimal and scoped.
- Primary names are `HAKO_*`; `NYASH_*` are compatibility aliases. 新規は `HAKO_*` を優先。短命のデバッグ用 ENV のみプロファイル内で使用。
- HostHandleRouter フェーズイン中の強制ENV（`NYASH_MAP_FORCE_HOST` / `NYASH_ARRAY_FORCE_HOST` / `NYASH_ARRAY_SIZE_FORCE_HOST` / `NYASH_STRING_SIZE_FORCE_HOST`）は開発・スモーク専用。長期運用は想定しない（将来的に削除）。

Direct load (dev‑only)
- `NYASH_PLUGIN_DIRECT_LIB` / `NYASH_PLUGIN_DIRECT_PATH` / `NYASH_PLUGIN_DIRECT_BOXES`
  - テスト/スモークで特定の .so を強制ロードしたい場合に使用（短命）。
  - 例: JSON プロバイダー: `LIB=libnyash_json_plugin.so`, `PATH=plugins/nyash-json-plugin/libnyash_json_plugin.so`, `BOXES=JsonDocBox,JsonNodeBox`。

TTL/cleanup
- 実験・観測用 ENV は短命。機能が安定したら削除または CLI/プロファイルへ昇格。


Adapter/Fallbacks
- Stage‑1 keys/values fallback is implemented in `runtime/adapters/map_keys_values_stage1.rs` and is active when `NYASH_PLUGIN_MAP_ARRAY_HANDLE` is not `1`.
- Stage‑2 (HostHandle arrays) requires `NYASH_PLUGIN_MAP_ARRAY_HANDLE=1` and returns real arrays; identity/parity tests are part of plugin‑on smokes.
# Environment Variables Guide
## HostHandle Router (development flags)

These flags force specific methods to route via HostHandleRouter early slots. They are for observation during phased rollout and default to OFF unless profiles enable them.

- Array
  - `NYASH_ARRAY_SIZE_FORCE_HOST=1` → slot 102 (`len/size/length`)
- Map
  - `NYASH_MAP_SIZE_FORCE_HOST=1` → slot 200 (`size/len`)
  - `NYASH_MAP_HAS_FORCE_HOST=1`  → slot 202 (`has/1`)
  - `NYASH_MAP_GET_FORCE_HOST=1`  → slot 203 (`get/1`)
  - `NYASH_MAP_SET_FORCE_HOST=1`  → slot 204 (`set/2`)
- String
  - `NYASH_STRING_SIZE_FORCE_HOST=1` → slot 300 (`size/len/length`)

Notes
- Flags are development-only and will be removed once unified routes are stable.
- Quick profile enables a minimal, safe subset; plugins profile enables broader coverage for Stage‑2 verification.

## Test Hooks (HostHandleRouter)

- `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1`
  - Purpose: Simulate a return-type mismatch (e.g., String.size expecting Integer but receiving non-Integer) to validate boundary handling (−14).
  - Scope: Affects String.size (slot 300) HostHandle path; returns `-14` directly when enabled.
  - Usage: Enable only in boundary tests (plugins profile). Do not use in normal runs.
