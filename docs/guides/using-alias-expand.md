# UsingAliasExpand — Local alias expansion (nested alias)

Purpose
- Support local, file‑scoped nested alias like:
  - `using A as X;`
  - `using X.B as Y` → expands to `A.B` before resolution

Location
- `src/runner/modes/common_util/resolve/alias_expand.rs`
- Integrated in:
  - `strip/collect.rs` (record local alias for namespace targets)
  - `pipeline.rs` (expand head alias before resolve)

Notes
- Dev/CI oriented; tests run with `NYASH_USING_AST=1` for stable behavior.
- Expansion is conservative: only non‑path targets (namespaces) are recorded; file paths are not aliased.

