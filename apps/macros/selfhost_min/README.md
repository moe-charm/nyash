Selfhost Minimal Macros (dev-only)

Overview
- Purpose: small sugar to make self-hosting Ny code shorter and clearer without changing semantics.
- Scope: dev/CI only. Enable via env `NYASH_MACRO_SELFHOST_MIN=1` (macros system must be enabled; default ON).

Macros
- `json({ ... })` / `map({ ... })`
  - Input: a map literal `{ key: value, ... }` (identifier or string keys).
  - Output: the same Map literal (expansion is identity at AST level; later lowering builds MapBox with set calls).
- `arr([ ... ])`
  - Input: an array literal `[e1, e2, ...]`.
  - Output: the same Array literal (later lowering builds ArrayBox with push calls).

- `call("Box.method/N", args...)` (dev, gated)
  - Input: first argument is a string name in the strict form `Box.method/N`.
  - Expansion (Rust MacroBox variant):
    - If it looks like a method (N>=1), it normalizes to a dotted ModuleFunction name and rewrites into a plain function call:
      - Example: `call("String.len/1", s)` → `FunctionCall("StringBox.len/0", [s])`.
    - Otherwise falls back to `FunctionCall(name, [args...])`.
  - Notes:
    - This macro is experimental and gated. Enable with `NYASH_ENABLE_CALL_MACRO=1` in smokes.
    - VM/module resolution for core built‑ins is still being strictified; unresolved errors may occur depending on profile.
    - When in doubt, prefer direct method syntax (`s.len()`) or explicit ModuleFunction calls emitted by the builder.

Notes
- These macros are simply “literal passthrough” adapters. They allow using `json(...)`/`arr(...)` notation in codebases preferring macro-style builders while keeping the generated AST identical to writing `{}`/`[]` directly.
- Nested structures are recursively preserved.
- Dev flags: set `NYASH_SYNTAX_SUGAR_LEVEL=full` to ensure `{}`/`[]` literal parsing is on (if your environment disables sugar by default).

Usage (two options)
- Nyash (.hako) macro package (suggested for users):
  - Put your macros in `apps/macros/selfhost_min/macros.hako` implementing `MacroBoxSpec.expand(json[, ctx]) -> json`.
  - Enable: `NYASH_MACRO_ENABLE=1 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako`
- Built‑in Rust MacroBox variant (bring‑up/dev only):
  - Enable: `NYASH_MACRO_BOX_ENABLE=SelfhostMinMacro`
  - This variant performs the same literal passthrough on the Rust side.

Dev flags summary
- `NYASH_MACRO_SELFHOST_MIN=1` registers the Rust SelfhostMinMacro (bring‑up path).
- `NYASH_ENABLE_CALL_MACRO=1` enables the call! smoke; transformation is experimental.
