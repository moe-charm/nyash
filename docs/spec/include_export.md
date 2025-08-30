Nyash Include/Export Specification (Phase 1)

Overview
- Goal: Simple, box-first module system for code split and reuse.
- Principle: One file exports one module (Box). include(path) returns that Box.

Syntax
- Include expression:
  local Math = include "lib/math.nyash"
  local r = Math.add(1, 2)

Rules
- One static box per file: The included file must define exactly one static box. Zero or multiple is an error.
- Expression form: include(...) is an expression that evaluates to the included module’s Box instance.
- Caching: The same file is evaluated at most once per run (subsequent includes return the cached module).
- Path resolution:
  - Relative: include("./x.nyash") resolves relative to the current project root.
  - Roots (nyash.toml):
    [include.roots]
    std = "./stdlib"
    lib = "./lib"
    Then include("std/string") -> ./stdlib/string.nyash, include("lib/utils") -> ./lib/utils.nyash
  - Extension/index: If extension is omitted, .nyash is appended. If the path is a directory, index.nyash is used.

Backends
- Interpreter: include is executed at runtime (implemented via execute_include_expr). Returns the Box instance.
- VM/AOT: include is handled by the MIR builder by reading and parsing the target file and lowering its static box into the same MIR module (no new MIR op added). The include expression lowers into a new Box() for the included static box type.
- No new MIR instruction was added; existing instructions (Const/NewBox/BoxCall/etc.) are used.

Limitations (Phase 1)
- Cycles: Circular includes are not yet reported with a dedicated error path. A follow-up change will add active-load tracking and a clear error message with the cycle path.
- Export box: Reserved for Phase 2. For now, the single static box in the file is the public API.

Examples
- See examples/include_math.nyash and examples/include_main.nyash.

Rationale
- Keeps MIR spec stable and relies on build-time module linking for VM/AOT.
- Aligns with Everything-is-Box: modules are Boxes; methods/fields are the API.

