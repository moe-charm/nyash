# LLVM Lowering Rules (Phase 15)

This document describes the active LLVM lowering rules used in Phase 15. Only the LLVM path is authoritative at this time.

## General
- Box values are represented as i64 handles when crossing the NyRT boundary.
- String operations prefer i8* fast paths (AOT helpers) when possible. Handle conversions are done only at explicit boundaries.

## NewBox
- StringBox:
  - When constructed from a constant string, lowering produces i8* via `nyash_string_new` and keeps it as i8* (no immediate handle conversion).
  - Builder skips redundant birth calls for StringBox.
- Other boxes:
  - Minimal birth shims exist (e.g., `nyash.box.birth_h`, `nyash.box.birth_i64`) using Box type ids.

## BoxCall: String.concat fast path
- If the receiver is annotated as String (or StringBox), lower to AOT helpers directly:
  - `concat_ss(i8*, i8*) -> i8*`
  - `concat_si(i8*, i64) -> i8*` (right operand is a handle coerced to string by NyRT)
  - `concat_is(i64, i8*) -> i8*`
- For non‑String receivers or plugin cases, fall back to plugin/by‑id paths as needed.

## BinOp Add: String concatenation
- Primary path: AOT helpers selected by operand shapes at IR time:
  - `i8* + i8*  -> concat_ss`
  - `i8* + i64  -> concat_si`
  - `i64  + i8* -> concat_is`
- Fallback policy: keep to the minimum. Do not add implicit conversions beyond the above without clear MIR type annotations. If mixed forms miscompile, fix MIR annotations first.

## ExternCall selection (console/debug)
- `env.console.{log,warn,error}` and `env.debug.trace` inspect the argument at lowering time:
  - If argument is `i8*`, call the C‑string variant: `nyash.console.{log,warn,error}` / `nyash.debug.trace`.
  - Otherwise convert to `i64` and call the handle variant: `nyash.console.{log,warn,error}_handle` / `nyash.debug.trace_handle`.
- The result values are ignored or zeroed as appropriate (side‑effecting I/O).

## Return/Result mapping
- For plugin/by‑id calls that return an i64 handle but the destination is annotated as pointer‑like (String/Box/Array/Future/Unknown), the handle is cast to an opaque pointer for SSA flow. Integers/Bools remain integers.

## Backend Consistency Notes
- VM/Cranelift/JIT are not MIR14‑ready and may not follow these rules yet. LLVM behavior takes precedence; other backends will be aligned later.
- Any new fallback must be justified and scoped; wide catch‑alls are prohibited to prevent backend divergence.

