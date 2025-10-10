# LAYER GUARD — legacy/ (VM convenience bridge)

Scope: `src/backend/mir_interpreter/handlers/boxes/legacy/`

Responsibility
- Minimal convenience bridge only (VM → PluginHost, last‑resort fallbacks)
- No core algorithms; no codec or loader logic here
- Long‑term: deprecate and delete after plugin parity stays green

Allowed imports
- VM interpreter primitives (`MirInterpreter`, `VMValue`)
- `crate::runtime::plugin_host_box` (facade) and `provider_box`

Forbidden imports
- `plugin_loader_v2` internals (loader/instance/ffi/specs)
- TLV helpers (`plugin_ffi_common`) — use via facade/codec box only

Fail‑Fast policy
- Do not add new hardcoded fast‑paths.
- Prefer routing through `PluginHostBox` or back to the normal dispatcher.

