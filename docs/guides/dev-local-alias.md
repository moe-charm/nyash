# Dev Sugar: @name = expr as local declaration

Status: dev-only, pre-expand sugar (no spec change)

Goal
- Speed up local declarations during development without impacting readability in shared code.

Syntax (dev sugar)
- Line-head only:
  - `@name = expr` → `local name = expr`
  - `@name: Type = expr` → `local name: Type = expr`

Rules
- Valid only at line start (leading spaces allowed). Inside expressions it is ignored.
- Declaration-only: not allowed for reassignments; use `name = expr` for assignments.
- Semantics are identical to `local` (scope/cleanup unchanged). Zero runtime cost.

Enablement
- Use the provided pre-expander script for dev: `tools/dev/at_local_preexpand.sh`.
- Example:
  - `tools/dev/at_local_preexpand.sh apps/tests/dev_sugar/at_local_basic.nyash > /tmp/out.nyash`
  - `NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm /tmp/out.nyash`

Style
- Shared/committed code: prefer explicit `local` (nyfmt may normalize @ to `local`).
- Dev/repl/prototype: `@` is acceptable to reduce noise.

Notes
- This is a text pre-expansion; it does not change the parser or MIR.
- The pattern is conservative to avoid collisions with comments and inline usages.

