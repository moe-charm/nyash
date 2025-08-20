# Issue 62 Update Proposal: Enable String Constants in WASM Backend First

This is a concrete request to implement minimal string support in the WASM backend so that Issue #62 can proceed. It reflects the current repo state.

## Background

- As noted in Issue #61, the current WASM backend does not support string constants yet.
- Issue #62 depends on string support and cannot be completed without it.
- Current state:
  - `src/backend/wasm/codegen.rs` → `generate_const` handles only Integer/Bool/Void; String is not implemented.
  - `src/backend/wasm/memory.rs` already defines a basic layout for `StringBox`:
    - Header: `[type_id:i32][ref_count:i32][field_count:i32]`
    - Fields: `[data_ptr:i32][length:i32]`
    - `StringBox` type_id = `0x1001`.

## Goal

Add minimal string constant support to the WASM backend:

- Allow `ConstValue::String` in codegen by embedding UTF-8 string bytes and constructing a `StringBox` with `[data_ptr,length]`.
- Provide a minimal debugging import `env.print_str(ptr,len)` to verify strings at runtime.
- Unblock Issue #62 implementation and tests that require strings.

## Scope

Minimal features required:

1) Data segments for string literals
   - Extend `WasmModule` (in `codegen.rs`) with a `data_segments: Vec<String>` field.
   - Update `to_wat()` to emit `(data ...)` after memory/globals and before functions/exports.
   - For each string constant, create a unique offset and emit a `(data (i32.const <offset>) "...bytes...")` entry.

2) Codegen for `ConstValue::String`
   - In `generate_const`, when encountering `ConstValue::String(s)`,
     - Allocate a data segment for `s` (UTF-8 bytes) and get its offset and length.
     - Allocate a `StringBox` using existing helpers (see `MemoryManager`),
       then set its fields: `data_ptr` and `length`.
     - Return the `StringBox` pointer (i32) in the destination local.

3) Helper for `StringBox` allocation
   - Either:
     - Provide a dedicated WAT helper function `$alloc_stringbox` that calls `$malloc`, writes header (`type_id=0x1001`, `ref_count=1`, `field_count=2`), and returns the box pointer, then inline store `data_ptr`/`length`.
   - Or:
     - Use `$box_alloc` with `(type_id=0x1001, field_count=2)` and then store `data_ptr`/`length` via generated `i32.store` sequences.

4) Runtime import for string output (for verification)
   - Extend `RuntimeImports` (`src/backend/wasm/runtime.rs`) with:
     - `(import "env" "print_str" (func $print_str (param i32 i32)))`
   - In host (Node/Browser), implement `importObject.env.print_str = (ptr,len) => { decode UTF-8 from memory; console.log(...) }`.

5) E2E test
   - Add a tiny program that produces/prints a string (e.g., Const String → call `env.print_str(ptr,len)` via a minimal MIR program) and verify it logs the correct text.
   - Option: update `test_runner.js` to include `print_str` and decode from memory using `TextDecoder('utf-8')`.

## Out of Scope (for this change)

- String operations (concat/substr/compare), normalization, encoding conversions.
- GC/RC or freeing memory (current allocator is bump-only).
- Returning StringBox directly from `main` (keep verification via `print_str`).

## Acceptance Criteria

- Generated WAT includes `(data ...)` segments for string literals and correct offsets.
- `ConstValue::String` codegen constructs a valid `StringBox` with proper `[data_ptr,length]`.
- `env.print_str` correctly prints UTF-8 strings in both Browser and Node runners.
- Issue #62 tasks that rely on strings can proceed.

## References (repo paths)

- String unsupported path: `src/backend/wasm/codegen.rs` (`generate_const`)
- Memory/layout: `src/backend/wasm/memory.rs` (StringBox, type_id=0x1001)
- Runtime imports: `src/backend/wasm/runtime.rs` (currently only `env.print(i32)`)
- Node runner: `test_runner.js` (has `env.print`; extend with `print_str`)

## Notes

- Data segment approach is the simplest for initial support; future work may add constant pooling and deduplication.
- Keeping verification via `print_str(ptr,len)` avoids complicating function return types for now.
- UTF-8 decoding is available in hosts via `TextDecoder('utf-8')`.

