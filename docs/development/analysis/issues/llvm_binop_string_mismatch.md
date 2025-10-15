# LLVM lowering: string + int causes binop type mismatch

Status: open

Summary

- When compiling code that concatenates a string literal with a non-string (e.g., integer), LLVM object emission fails with a type mismatch in binop.
- Example from `apps/ny-map-llvm-smoke/main.nyash`: `print("Map: v=" + v)` and `print("size=" + s)`.

Environment

- LLVM 18 / inkwell 0.5.0
- Phase 11.2 lowering

Repro

1) Run: `NYASH_LLVM_ARRAY_SMOKE=1 ./tools/llvm_smoke.sh release` (or build/link the map smoke similarly)
2) Observe: `❌ LLVM object emit error: binop type mismatch`

Expected

- String concatenation should be lowered to a safe runtime shim (e.g., NyRT string builder or `nyash_string_concat`) that accepts `(i8* string, i64/int)` and returns `i8*`.

Observed

- `+` binop is currently generated as integer addition for non-float operands, leading to a type mismatch when one side is a pointer (string) and the other is integer.

Plan

1) Introduce string-like detection in lowering: if either operand is `String` (or pointer from `nyash_string_new`), route to a NyRT concat shim.
2) Provide NyRT APIs:
   - `nyash.string.concat_ss(i8*, i8*) -> i8*`
   - `nyash.string.concat_si(i8*, i64) -> i8*`
   - Optional: `concat_sf`, `concat_sb` (format helpers)
3) As an interim simplification for smoke, emit `print("..." )` in two steps to avoid mixed-type `+` until the concat shim is ready.

CI

- Keep `apps/ny-llvm-smoke` OFF by default. Re-enable once concat shim lands and binop lowering is updated.

