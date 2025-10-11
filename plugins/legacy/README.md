# Legacy Plugins (not yet on Hako ABI)

This folder is a temporary parking area for plugins that have not yet migrated to the unified TLV/handle ABI (Hako ABI).

Scope
- These crates still depend on legacy helper functions or ad‑hoc TLV encoders/decoders.
- They should not be built in normal profiles; keep them out of the default workspace members if necessary.

How to restore (migration checklist)
1) Extend `hako_abi_impl` if a missing TLV primitive is required (e.g., void, bytes).
2) Refactor the plugin to use standard TLV encode/decode helpers only.
3) Provide a `pub fn register_static(host: &mut PluginHostV2)` entry to support static linking.
4) Move the crate back under `plugins/` root (out of `legacy/`).
5) Run `plugins` profile smokes; confirm no new warnings and green tests.

Known gaps and notes
- console: requires `write_tlv_void()` and first-arg string helpers (to be added to `hako_abi_impl`).
- egui: depends on legacy runtime functions; must switch to standard TLV.
- encoding: requires TLV bytes; add `write_tlv_bytes()`/`read_arg_bytes()` first.
