# ArrayBox get/set -> Invalid arguments (plugin side)

Status: open

Summary

- Error messages observed during AOT/LLVM smoke that touches ArrayBox:
  - "Plugin invoke error: ArrayBox.set -> Invalid arguments"
  - "Plugin invoke error: ArrayBox.get -> Invalid arguments"
- VInvoke(MapBox) path is stable; issue is isolated to ArrayBox plugin invocation.

Environment

- LLVM 18 / inkwell 0.5.0 (llvm18-0)
- nyash-rust with Phase 11.2 lowering
- tools/llvm_smoke.sh (Array smoke is gated via NYASH_LLVM_ARRAY_SMOKE=1)

Repro

1) Enable array smoke explicitly:
   - `NYASH_LLVM_ARRAY_SMOKE=1 ./tools/llvm_smoke.sh release`
2) Observe plugin-side errors for ArrayBox.get/set.

Expected

- Array get/set should be routed to NyRT safety shims (`nyash_array_get_h/set_h`) with handle + index/value semantics that match the core VM.

Observed

- Plugin path is taken for ArrayBox.get/set and the plugin rejects arguments as invalid.

Notes / Hypothesis

- LLVM lowering is intended to map ArrayBox.get/set to NyRT shims. The plugin path should not be engaged for array core operations.
- If by-name fallback occurs (NYASH_LLVM_ALLOW_BY_NAME=1), the array methods might route to plugin-by-name with i64-only ABI and mismatched TLV types (index/value encoding).

Plan

1) Confirm lowering branch for BoxCall(ArrayBox.get/set) always selects NyRT shims under LLVM, regardless of by-name flag.
2) If by-name fallback is unavoidable in current scenario, ensure integer index/value are encoded/tagged correctly (tag=3 for i64) and receiver is a handle.
3) Add a targeted smoke (OFF by default) that calls only `get/set/length` and prints deterministic result.
4) Optional: Add debug env `NYASH_PLUGIN_TLV_DUMP=1` to print decoded TLV for failing invokes to speed diagnosis.

Workarounds

- Keep `NYASH_LLVM_ARRAY_SMOKE=0` in CI until fixed.

