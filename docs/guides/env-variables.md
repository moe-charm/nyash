# ENV Variables — Core (Plugins/Provider)

Key variables (current)
- `HAKO_PLUGIN_POLICY` (auto|off|force) — plugin load policy
- `NYASH_PLUGIN_CONFIG` — plugin config path (prefer `hako.toml`)
- `NYASH_USE_PLUGIN_BUILTINS` — allow plugin to override core box types
- `NYASH_PLUGIN_OVERRIDE_TYPES` — comma list (e.g., `StringBox,ArrayBox,MapBox`)
- `NYASH_BUILTIN_DISABLE_{STRING|ARRAY|MAP}` — disable builtin core boxes (dev gate)
- `NYASH_PLUGIN_MAP_ARRAY_HANDLE` — 1 to enable Stage‑2 keys/values HostHandle path; 0 for Stage‑1 keysS/valuesS shim

Profiles
- plugin‑on: sets `HAKO_PLUGIN_POLICY=auto`, `NYASH_PLUGIN_CONFIG=hako.toml`
- plugins: keeps Stage‑1 keys/values (HostHandle OFF) for stability

Birth Adoption
- VM will call `birth()` when a plugin box is created with `instance_id=0`, and adopt the returned handle.
- No‑op when `birth` does not exist.

Notes
- Prefer CLI/Profiles over ENV when possible; ENV should be minimal and scoped.


Adapter/Fallbacks
- Stage‑1 keys/values fallback is implemented in `runtime/adapters/map_keys_values_stage1.rs` and is active when `NYASH_PLUGIN_MAP_ARRAY_HANDLE` is not `1`.
- Stage‑2 (HostHandle arrays) requires `NYASH_PLUGIN_MAP_ARRAY_HANDLE=1` and returns real arrays; identity/parity tests are part of plugin‑on smokes.

