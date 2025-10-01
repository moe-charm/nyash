Collections Boxes — Minimal Semantics (Array/Map)

Scope
- Define the minimal, shared semantics for ArrayBox and MapBox used across VM, Plug‑ins, and Kernel (AOT) paths.
- Purpose: reduce divergence and enable a thin, testable core for collection behavior.

ArrayBox
- Construction: `new ArrayBox()` creates an empty array.
- Methods (MVP):
  - `push(value)` → adds to the end; returns "ok" (string) by convention.
  - `get(index:i64)` → returns element at index or `NullBox` if out of bounds.
  - `set(index:i64, value)` →
    - if `index < len`: replaces element; returns "ok".
    - if `index == len`: appends; returns "ok".
    - else: returns error string (implementation-defined message).
  - `len()`/`length()` → returns current length as i64.
  - `toString()` → implementation‑defined, stable goal is `ArrayBox(size=N)`.
- Notes:
  - Numeric operations and equality follow Box semantics; mixed types use string fallback when ordered.
  - Methods should not panic on invalid input; prefer `NullBox` or error string.

MapBox
- Construction: `new MapBox()` creates an empty map (string keys).
- Methods (MVP):
  - `set(key:string, value)` → inserts or replaces; returns "ok".
  - `get(key:string)` → returns value or `NullBox` if missing.
  - `has(key:string)` → returns bool (0/1 as i64 in MIR path).
  - `size()`/`len()` → returns number of entries as i64.
  - `toString()` → implementation‑defined, stable goal is `MapBox(size=N)`.
- Notes:
  - Keys are strings by policy in MVP; non‑string keys should be coerced via `toString()` or rejected consistently.

Core Policy
- Behavior is defined once here and shared by VM built‑ins, Plug‑ins, and Kernel shims.
- Error handling is fail‑fast for programmer errors in dev builds; at runtime, return neutral values (null/"ok") rather than panicking.

Testing
- Quick smokes validate minimal behavior only (push/get/len for Array; set/get/size for Map).
- Parity across VM/Plug‑in/Kernel is maintained by golden tests referencing this spec; extended methods may be tested separately.

