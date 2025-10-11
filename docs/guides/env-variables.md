# ENV Variables — Core (Plugins/Provider)

Key variables (current)
- `HAKO_PLUGIN_POLICY` (auto|off|force) — plugin load policy
- `NYASH_PLUGIN_CONFIG` — plugin config path (prefer `hako.toml`)
- `NYASH_USE_PLUGIN_BUILTINS` — allow plugin to override core box types
- `NYASH_PLUGIN_OVERRIDE_TYPES` — comma list (e.g., `StringBox,ArrayBox,MapBox`)
- `NYASH_BUILTIN_DISABLE_{STRING|ARRAY|MAP}` — disable builtin core boxes (dev gate)
- `NYASH_PLUGIN_MAP_ARRAY_HANDLE` — Stage‑2: 1 で keys/values HostHandle 経路を有効化。0 で Stage‑1(keysS/valuesS) シム（plugins プロファイルは既定ON）。
- `HAKO_HOST_HANDLE_TRACE` / `NYASH_HOST_HANDLE_TRACE` — HostHandle slot呼び出しの観測ログ（短命/既定OFF）

Profiles
- plugin‑on: sets `HAKO_PLUGIN_POLICY=auto`, `NYASH_PLUGIN_CONFIG=hako.toml`
- plugins: Stage‑2 HostHandle 既定ON（`NYASH_PLUGIN_MAP_ARRAY_HANDLE=1`）。

Birth Adoption
- VM will call `birth()` when a plugin box is created with `instance_id=0`, and adopt the returned handle.
- No‑op when `birth` does not exist.

Notes
- Prefer CLI/Profiles over ENV when possible; ENV should be minimal and scoped.
- Primary names are `HAKO_*`; `NYASH_*` are compatibility aliases. 新規は `HAKO_*` を優先。

TTL/cleanup
- 実験・観測用 ENV は短命。機能が安定したら削除または CLI/プロファイルへ昇格。


Adapter/Fallbacks
- Stage‑1 keys/values fallback is implemented in `runtime/adapters/map_keys_values_stage1.rs` and is active when `NYASH_PLUGIN_MAP_ARRAY_HANDLE` is not `1`.
- Stage‑2 (HostHandle arrays) requires `NYASH_PLUGIN_MAP_ARRAY_HANDLE=1` and returns real arrays; identity/parity tests are part of plugin‑on smokes.

