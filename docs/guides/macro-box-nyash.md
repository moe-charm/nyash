# MacroBox in Nyash (Design Draft)

Status: Design draft (Phase 16+). This document specifies the Nyash-side API for user-extensible macros that execute during compilation (pre-MIR). Rust prototype exists; this spec focuses on the Nyash interface and constraints.

Philosophy
- Box Independence (loose coupling): macro expands against public interfaces only; avoid coupling to private internals.
- Deterministic, side-effect free: no IO, no randomness, no time/env dependence.
- Safety and staging: runs in a sandbox before MIR generation; never observes runtime state.

API (Nyash)
```nyash
box MacroBoxSpec {
  // Required entry. Receives entire AST and returns a transformed AST.
  static function expand(ast) { /* pure transform */ }

  // Optional metadata.
  static function name() { return "MyMacro" }
}
```

Registration
- Declared in `nyash.toml` (planned):
```
[macros]
paths = [
  "apps/macros/my_macro.nyash",
  "apps/macros/json_lints.nyash"
]
```
- Loading policy (dev-only gate): `NYASH_MACRO_BOX_NY=1` enables loading Nyash MacroBoxes from configured paths.
- Isolation: loaded in a dedicated interpreter with IO disabled; only AST utilities and safe helpers exposed.
 - Interim mapping (prototype): `name()` may map to built-in MacroBoxes for effects (e.g., `"UppercasePrintMacro"`). Otherwise, identity transform.

Execution Order
- Built-in (Rust) macro passes
- User MacroBoxes (Nyash) in registration order
- Test harness injection
- Stop on fixed point, or when reaching the pass/cycle guard limits.

Guards
- Max passes: `NYASH_MACRO_MAX_PASSES` (default 32)
- Cycle window: `NYASH_MACRO_CYCLE_WINDOW` (default 8)

Constraints
- Pure transform: no external calls (FFI/hostcall) and no global writes.
- Public-only policy for derives: when generating methods like `equals` or `toString`, operate on public fields only.
- Diagnostics: write via `NYASH_MACRO_TRACE=1` (compiler-owned), not via print in user code.

Future Work
- Packaging: macro crates with versioning; integrity checks.
- Capability tokens: opt-in capabilities for limited read-only context (e.g., box names list).
- Attribute-style hooks: `@derive`, `@rewrite`, `@lint` as MacroBox conventions.
