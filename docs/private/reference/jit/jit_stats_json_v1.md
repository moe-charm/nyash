JIT Stats JSON Schema — Version 1
=================================

This document describes the fields emitted in the JIT statistics JSON outputs (version=1).

Sources
- Unified JIT stats (VM-side): printed when `NYASH_JIT_STATS_JSON=1` (or CLI equivalent)
- Box API: `new JitStatsBox().toJson()` returns a compact JSON, `summary()` returns a pretty JSON summary (via VM dispatch)

Version
- `version`: number — Schema version (currently `1`)

Unified JIT Stats (JSON)
- `version`: number — Schema version (1)
- `sites`: number — Number of JIT call sites observed
- `compiled`: number — Count of functions compiled by the JIT
- `hits`: number — Sum of entries across all functions (hotness hits)
- `exec_ok`: number — Count of successful JIT executions
- `trap`: number — Count of JIT executions that trapped and fell back to VM
- `fallback_rate`: number — Ratio `trap / (exec_ok + trap)` (0 when denominator is 0)
- `handles`: number — Current number of live JIT handles in the registry
- `abi_mode`: string — `"i64_bool"` or `"b1_bool"` (when toolchain supports b1)
- `abi_b1_enabled`: boolean — Whether b1 ABI is requested by config
- `abi_b1_supported`: boolean — Whether current toolchain supports b1 in signatures
- `b1_norm_count`: number — Count of b1 normalizations (e.g., i64!=0 to b1)
- `ret_bool_hint_count`: number — Count of functions lowered with return-bool hint
- `phi_total_slots`: number — Total PHI slots encountered (always-on, LowerCore-based)
- `phi_b1_slots`: number — PHI slots classified as boolean (heuristics)
- `top5`: array of objects — Top hot functions
  - `name`: string
  - `hits`: number
  - `compiled`: boolean
  - `handle`: number (0 when not compiled)

Compact Box JSON — `JitStatsBox.toJson()`
- `version`: number — Schema version (1)
- `abi_mode`: string — Current ABI mode for booleans (see above)
- `abi_b1_enabled`: boolean — Whether b1 ABI is requested by config
- `abi_b1_supported`: boolean — Whether current toolchain supports b1 in signatures
- `b1_norm_count`: number — b1 normalizations count
- `ret_bool_hint_count`: number — return-bool hint count
- `phi_total_slots`: number — Total PHI slots (accumulated)
- `phi_b1_slots`: number — Boolean PHI slots (accumulated)

Summary Box JSON — `JitStatsBox.summary()`
- Pretty-printed JSON containing:
  - All fields from `toJson()`
  - `top5`: same structure as unified stats

Notes
- Counters reflect process-lifetime totals (reset only on process start).
- `phi_*` counters are accumulated per LowerCore invocation (function lower), independent of dump flags.
- `abi_mode` switches to `b1_bool` when toolchain support is detected and b1 ABI is enabled.

Examples
```
{
  "version": 1,
  "sites": 1,
  "compiled": 0,
  "hits": 1,
  "exec_ok": 0,
  "trap": 0,
  "fallback_rate": 0.0,
  "handles": 0,
  "abi_mode": "i64_bool",
  "abi_b1_enabled": false,
  "abi_b1_supported": false,
  "b1_norm_count": 0,
  "ret_bool_hint_count": 0,
  "phi_total_slots": 2,
  "phi_b1_slots": 1,
  "top5": [ { "name": "main", "hits": 1, "compiled": false, "handle": 0 } ]
}
```

