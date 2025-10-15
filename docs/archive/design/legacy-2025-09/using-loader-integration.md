# Using → Loader Integration (Minimal)

Goal
- Keep `using` simple: strip lines and resolve names to paths/aliases.
- Add the minimal integration so userland boxes referenced via `using` are actually available at compile/run time.

Scope (pause‑safe)
- Parser stays Phase‑0: keep `nyashstd` restriction; do not widen grammar.
- Integration is a runner step (pre‑parse): resolve and register modules; do not change language semantics.

Design
- Strip `using` lines when `NYASH_ENABLE_USING=1` (already implemented).
- For each `using ns [as alias]?`:
  - Resolve `ns` → path via: [modules] → aliases → using.paths (apps/lib/.) → context dir.
  - Register mapping in `modules_registry` as `alias_or_ns -> path` (already implemented).
- Minimal loader hook (defer heavy linking):
  - Compile/execute entry file as today.
  - Userland boxes are accessed via tools/runners that read from `modules_registry` where needed (e.g., PyVM harness/tests).

Notes
- Entry thin‑ization (Mini‑VM) waits until loader reads userland boxes on demand.
- Keep docs small: this note serves as the canonical link; avoid duplicating details in other pages.

Preprocessing invariants (runner)
- `using` lines are stripped and resolved prior to parse; dependencies are inlined before `Main` so names are available without changing language semantics.
- Line‑head `@name[:T] = expr` is normalized to `local name[:T] = expr` as a purely textual pre‑expand (no semantic change). Inline `@` is not recognized; keep `@` at line head.
- These steps are pause‑safe: they do not alter AST semantics; they only simplify authoring and module wiring.

Links
- Runner pipeline: src/runner/pipeline.rs
- Using strip/resolve: src/runner/modes/common_util/resolve.rs
- Env: NYASH_ENABLE_USING, NYASH_USING_STRICT, NYASH_SKIP_TOML_ENV
