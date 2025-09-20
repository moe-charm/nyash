# Macro System (Phase 16) — Quickstart

Status: MVP available behind environment gates (default OFF). This page describes how to enable and use the initial features (derive, test runner, expansion dump) and the developer‑facing Pattern/Quote API preview.

## Enabling/disabling macros

- Default: ON
- Disable: `NYASH_MACRO_DISABLE=1` or `NYASH_MACRO_ENABLE=0|false|off`
- Debug trace: `NYASH_MACRO_TRACE=1`
- CLI shortcut for debug: `--expand` (prints pre/post AST, and sets trace)

## Derive (MVP)

- Provided derives: `Equals`, `ToString`
- Control via env:
  - `NYASH_MACRO_DERIVE=Equals,ToString` (default when unset)
  - `NYASH_MACRO_DERIVE_ALL=1` (force all derives)
- Behavior:
  - Injects `equals(__ny_other)` and `toString()` into Box declarations when not already present.
  - Public-only: derives operate on public fields only (Philosophy: Box independence / loose coupling).
  - No changes to MIR instruction set; expansion happens on AST, then compiled as usual.

Example
```
NYASH_MACRO_ENABLE=1 ./target/release/nyash --backend vm apps/APP/main.nyash
```

## Test runner (MVP)

- CLI: `--run-tests` → enable macro engine and inject a test harness main
- Discovery targets:
  - Top‑level functions with names `test_*`
  - Box static/instance methods `test_*`
- Filtering: `--test-filter SUBSTR` or `NYASH_TEST_FILTER=SUBSTR`
- Entry policy when a main exists:
  - `--test-entry wrap` → rename original `main` to `__ny_orig_main`, run tests then call original
  - `--test-entry override` → replace entry with test harness only
  - Force apply even if a main exists: `NYASH_TEST_FORCE=1`
- Return policy in wrap mode:
  - `--test-return {tests|original}` or `NYASH_TEST_RETURN`
  - `tests` (default): harness returns number of failed tests
  - `original`: return the original main result (tests still run)
- Parameterized tests (MVP): `NYASH_TEST_ARGS_DEFAULTS=1` injects integer `0` defaults for each parameter (static/instance)
- Exit code: number of failed tests (0 = green)

Examples
```
# run all tests in a file
./target/release/nyash --run-tests apps/tests/my_tests.nyash

# filter + wrap entry + default arg injection
NYASH_MACRO_ENABLE=1 NYASH_TEST_ARGS_DEFAULTS=1 \
./target/release/nyash --run-tests --test-filter http --test-entry wrap apps/tests/my_tests.nyash
```

## Expansion dump

```
./target/release/nyash --expand --dump-ast apps/tests/ternary_basic.nyash
```
Shows pre/post expansion AST (debug only).

## Core Normalization (always-on when macros enabled)

Certain language sugars are normalized before MIR across all runners when the macro gate is enabled. These are not user macros and do not require any registration:

- for sugar → Loop
  - `for(fn(){ init }, cond, fn(){ step }, fn(){ body })`
  - Emits: `init; loop(cond){ body; step }`
  - `init/step` also accept a single Assignment or Local instead of `fn(){..}`.

- foreach sugar → Loop
  - `foreach(array_expr, "x", fn(){ body_with_x })`
  - Expands into an index-based loop with `__ny_i`, and substitutes `x` with `array_expr.get(__ny_i)` inside body.

Normalization order (within the core pass):
1) for/foreach → 2) match(PeekExpr) → 3) loop tail alignment (carrier-like ordering; break/continue segments supported).

Notes:
- Backward-compat function names `ny_for` / `ny_foreach` are also accepted but `for` / `foreach` are preferred.
- This pass is part of the language pipeline; it is orthogonal to user-defined macros.

## Developer API (preview)

- Pattern/Quote primitives are available to bootstrap macro authorship.
- TemplatePattern:
  - Bind placeholders using `$name` inside a template AST.
  - OrPattern: alternation of templates.
  - Variadic `$...name` supported at any position inside call/array argument lists; binds the variable-length segment to a pseudo list (`ASTNode::Program`).
- Unquote:
  - Replace `$name` with the bound AST.
  - Splice `$...name` into call/array argument lists.
  - Array/Map nodes participate in pattern/unquote (map keys must match literally; values can bind via `$name`).

## MacroCtx (PoC)

User macros executed via the PyVM sandbox receive a second argument `ctx` in `expand(json, ctx)`:

- Shape (string): `{ "caps": { "io": bool, "net": bool, "env": bool } }`
- Policy: all caps default to false. The sandbox disables plugins and exposes a minimal Box API.
- Do not print to stdout from macros: the child process stdout is reserved for the expanded JSON.
  - For diagnostics use stderr in the future; for now prefer silent operation or trace via the parent process.

Example (identity):
```
static box MacroBoxSpec {
  name() { return "MacroCtxDemo" }
  expand(json, ctx) { return json }
}
```

### JSON test args (advanced)

For `--run-tests`, you can supply per-test arguments and instance construction details via `NYASH_TEST_ARGS_JSON`.

Shapes:
- Simple list (as before): `{ "test_name": [1, "s", true], "Box.method": [ 0, 1 ] }`
- Detailed object:
  - `{ "Box.method": { "args": [ ... ], "instance": { "ctor": "new|birth", "args": [ ... ], "type_args": ["T", "U"] } } }`
- Typed values inside args:
  - `{ "i": 1 }`, `{ "f": 1.2 }`, `{ "s": "x" }`, `{ "b": true }`, `null`
  - Arrays/Maps as value: `{ "array": [1, 2, 3] }`, `{ "map": { "k": 1, "k2": {"s":"v"} } }`
  - Call/method/var literals: `{ "call": "fn", "args": [...] }`, `{ "method": "m", "object": {"var":"obj"}, "args": [...] }`, `{ "var": "name" }`

Diagnostics:
- When JSON cannot be mapped or arity mismatches, warnings are printed with `[macro][test][args]` and the specific test may be skipped unless `NYASH_TEST_ARGS_DEFAULTS=1` is set.

Notes
- Textual `quote(code)` is supported as a convenience via the existing parser, but `$name` placeholders should be built in AST templates directly.
- This API will evolve; treat as experimental until the stable `macro_box` interface is finalized.

## Expansion Safety (Depth/Cycle)

- The macro engine applies a bounded number of expansion passes and stops on fixpoint.
- Environment tuning:
  - `NYASH_MACRO_MAX_PASSES` (default 32)
  - `NYASH_MACRO_CYCLE_WINDOW` (default 8) — detect cycles across recent states
  - `NYASH_MACRO_TRACE=1` — pass-by-pass logging
