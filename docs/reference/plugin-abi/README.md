Nyash Plugin ABI — Overview

Scope
- Summarize v2 (today) vs Final Vision (phase‑in) and migration path.

Current (v2)
- Minimal TypeBox with TLV-ish arguments, string/bytes as raw pointers.
- `invoke_id(instance_id, method_id, tlv)` fast path, optional legacy fallbacks.
- Focus on simplicity for first‑party plugins.

Final Vision (phase‑in; additive)
- NyValue/NyResult: typed values and rich error info.
- resolve+invoke only (method() fallback removed) for clarity/perf.
- Meta: `get_method_meta/get_all_methods/get_type_info` (tools/IDE/docs).
- Boundary hardening (plugins only): required_capabilities, method_effects, contracts.
- Optional Component Model (WIT) info for ecosystem integration.

Migration
- Default OFF: runtime maintains v2 behavior.
- Enable probes via env:
  - `NYASH_PLUGIN_ABI_FINAL=1` → prefer NyResult invoke when available.
  - `NYASH_PLUGIN_META=1` → query meta (silent when missing).
  - `NYASH_PLUGIN_CAPS_ENFORCE=1` → enforce capabilities (dev/ci).
  - `NYASH_TRACE_EFFECTS=1`, `NYASH_CHECK_CONTRACTS=1` → log-only.
- Existing plugins continue to work; new features are opt‑in.

Reference
- v2: docs/reference/plugin-abi/nyash_abi_v2.md
- Final Vision: docs/reference/plugin-abi/nyash_abi_final_vision.md
