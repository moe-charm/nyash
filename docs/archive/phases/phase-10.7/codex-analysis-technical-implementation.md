# Codex (GPT-5)の技術分析：Python→Nyashトランスパイルの実装詳細

2025-08-30 - OpenAI Codex v0.25.0による技術的深掘り

## Big Picture

- **Purpose**: Generate Nyash source from Python to leverage Nyash's optimizer, keep debugging simple, and "think in Nyash" early without locking into MIR details.
- **Core idea**: Normalize Python to a small CorePy IR, then lower to idiomatic Nyash with a thin "Py runtime shim" in Nyash that preserves Python's dynamic semantics where required.

## AST Conversion

### Two-stage lowering
```
Python AST → CorePy IR → Nyash AST
```

CorePy should be expression-friendly, desugared, and semantics-explicit (e.g., with→try/finally, for→iterator loop, boolean ops→if/else).

### Visitor + environment
Implement a node visitor that carries scope info (locals/free/globals), evaluation order, and source spans. Use Python's `symtable` to seed symbol kinds; validate against your own pass.

### Semantic shims
Lower Python ops to Nyash intrinsics that preserve semantics:
- `py_truthy(x)`
- `py_getattr(o,"a")`
- `py_setattr(o,"a",v)`
- `py_binop("add", a, b)`
- `py_cmp("lt", a, b)`
- `py_iter(x)`
- `py_call(f, args, kwargs)`
- `py_slice(x, i, j, k)`

### Boxes and cells
Model Python variables and closures with Box/Cell objects. Rule of thumb: locals are unboxed unless captured or aliased; promote to Box when needed. Everything-is-Box in Nyash aligns well with Python's mutability/aliasing.

### Control flow
Normalize to a small set: `if`, `block`, `loop`, `break`, `continue`, `try/catch/finally`, `throw`. Lower `and`/`or` with short-circuit temp; turn comprehensions into explicit loops with dedicated inner scope.

## Transpile Quality

### Readability vs optimize
Offer modes. Default emits idiomatic Nyash constructs and meaningful identifiers, comments with source spans, and simple temporaries. "Optimize" mode switches to `py_*` intrinsics fusion and fewer temps.

### Idiomatic Nyash
Prefer Nyash control constructs over procedural labels. Use native blocks for `if/else`, `match` if Nyash has it; only fall back to runtime calls where semantics demand.

### Stable pretty-printer
Round-trip friendly formatter with consistent whitespace, trailing comma rules, and deterministic temp naming (`_t1`, `_t2…`). Keep def/class emitted in declaration-order.

### Debug info
Attach `span(file, line, col)` to every IR node, carry through to Nyash AST, then emit a sidecar source map. Optionally embed lightweight `#line` directives or inline comments per statement in debug builds.

## Python Feature Mapping

### Default args
Evaluate at def-time; store tuple/dict on the function object. At call-time, fill missing with stored defaults. Beware mutable defaults: do not clone; reuse exact object.

### LEGB scoping
Build symbol table with flags for `global`/`nonlocal`. Emit closure "cells" (Boxes) for free vars; functions capture a vector of cells. Globals map to the module dict; builtins fallback when name miss in global.

### for/else, while/else
Introduce `broken=false`. On `break`, set and exit; after loop, `if !broken { else_block }`.

### Comprehensions
Create inner function/scope per comprehension (Python 3 semantics). Assignment targets exist only in that scope. Preserve evaluation order and late binding behavior.

### With statement
Desugar to try/finally per Python spec: evaluate context expr, call `__enter__`, bind target, run body, always call `__exit__`, and suppress exception only if `__exit__` returns truthy.

### Decorators
Evaluate bottom-up at def-time: `fn = decoN(...(deco1(fn)))` then assign back. Keep evaluation order of decorator expressions.

### Generators
Lower to a state machine object implementing Nyash's iterator protocol, with saved instruction pointer, stack slots, and exception injection (`throw`, `close`). Support `yield from` by delegation trampoline.

### Pattern matching (PEP 634)
If supported by Nyash, map directly; else lower to nested guards and extractor calls in a `py_match` helper library.

### Data model
Attribute access honors descriptors; method binding creates bound objects; arithmetic and comparisons dispatch to `__op__`/`__rop__` and rich comparisons; truthiness via `__bool__`/`__len__`.

## Performance Opportunities

### At transpile-time
- Constant fold literals, f-strings (format plan precomputation), simple arithmetic if types are literal.
- Invariant hoisting for loop-invariant comprehensions and attribute chains when no side-effects (guarded).
- Direct calls to Nyash intrinsics for selected builtins (`len`, `isinstance`, `range`) only if not shadowed (prove via symbol table).
- Peephole: collapse nested `py_truthy(py_truthy(x))`, merge adjacent `if` with literal conditions, drop dead temporaries.

### Defer to Nyash compiler
- Inlining across Nyash functions, register allocation, loop unrolling, vectorization, constant propagation at MIR level.
- DCE/CSE once `py_*` helpers are inlined or annotated as pure/idempotent where legal.

### Types as hints
- Consume Python annotations/`typing` to emit specialized fast paths: e.g., `int` → direct Nyash integer ops, else fallback to `py_binop`. Plumb types through IR as optional metadata for MIR to exploit.
- Profile-guided guards: optional mode emits guards around hot calls to enable Nyash JIT/AOT to speculate and deopt to generic `py_*`.

## Error Handling & Debug

### Source maps
Emit a compact mapping (e.g., VLQ JSON) from Nyash line/col → Python original; include segment mappings per expression for precise stepping.

### Exception rewriting
Wrap Nyash runtime entrypoints to translate stack frames via the source map, hiding frames from helpers (`py_*`) unless verbose mode is on.

### Stage diagnostics
- CorePy dump: toggle to print normalized IR with spans.
- Nyash preview: post-format preview with original Python line hints.
- Trace toggles: selective tracing of `py_call`, `py_getattr`, iteration; throttle to avoid noise.

### Friendly messages
On unsupported nodes or ambiguous semantics, show minimal repro, Python snippet, and link to a doc page. Include symbol table excerpt when scoping fails.

## Architecture & DX

### Pass pipeline
```
Parse Python AST → Symbol table → Normalize to CorePy → 
Scope/closure analysis → Type/meta attach → Lower to Nyash AST → 
Optimize (peephole/simplify) → Pretty-print + source map
```

### Runtime shim (`nyash/lib/py_runtime.ny`)
Core APIs:
- `py_call(f, pos, kw, star, dstar)`
- `py_truthy(x)`
- `py_getattr/py_setattr`
- `py_binop(op, a, b)`
- `py_cmp(op,a,b)`
- `py_iter(x)`
- `py_next(it)`
- `py_slice(x,i,j,k)`
- `py_with(mgr, body_fn)`
- `py_raise`
- `py_is`
- `py_eq`

Data model support: descriptor get/set, method binding, MRO lookup, exception hierarchy, StopIteration protocol.

Perf annotations: mark pure or inline candidates; keep stable ABI.

### CLI/flags
Modes:
- `--readable`
- `--optimized`
- `--debug`
- `--emit-sourcemap`
- `--dump-corepy`
- `--strict-builtins`

Caching: hash of Python AST + flags to cache Nyash output, source map, and diagnostics.

Watch/incremental: re-transpile changed modules, preserve source map continuity.

### Tests
- Golden tests: Python snippet → Nyash output diff, with normalization.
- Differential: run under CPython vs Nyash runtime for functional parity on a corpus (unit/property tests).
- Conformance: edge cases (scoping, descriptors, generators, exceptions) and evaluation order tests.

## Pitfalls & Remedies

### Evaluation order
Python's left-to-right arg eval, starred/unpacking, and kw conflict checks. Enforce by sequencing temps precisely before `py_call`.

### Shadowing builtins/globals
Only specialize when proven not shadowed in any reachable scope. Provide `--strict-builtins` to disable specialization unless guaranteed.

### Identity vs equality
`is` is reference identity; avoid folding or substituting.

### Integer semantics
Python's bigints; ensure Nyash numeric layer matches or route to bigints in `py_*`.

## Future Extensibility

### Plugins
Pass manager with hooks (`before_lower`, `after_lower`, `on_node_<Type>`). Allow project-local rewrites and macros, with access to symbol/type info.

### Custom rules
DSL for pattern→rewrite with predicates (types, purity), e.g., rewrite `dataclass` patterns to Nyash records.

### Multi-language
Keeping the Nyash script as a stable contract invites other frontends (e.g., a subset of JS/TypeScript or Lua) to target Nyash; keep `py_*` separate from language-agnostic intrinsics to avoid contamination.

### Gradual migration
As Nyash grows Pythonic libraries, progressively replace `py_*` with native Nyash idioms; keep a compatibility layer for mixed projects.

## Concrete Translation Sketches

### Attribute
```python
a.b
```
→
```nyash
py_getattr(a, "b")
```

### Call
```python
f(x, y=1, *as, **kw)
```
→
```nyash
py_call(f, [x], {"y":1}, as, kw)
```

### If
```python
if a and b:
```
→
```nyash
let _t=py_truthy(a); if _t { if py_truthy(b) { ... } }
```

### For/else
```python
for x in xs:
    if cond:
        break
else:
    else_block
```
→
```nyash
let _it = py_iter(xs); 
let _broken=false; 
loop { 
    let _n = py_next(_it) catch StopIteration { break }; 
    x = _n; 
    ... 
    if cond { _broken=true; break } 
} 
if !_broken { else_block }
```

### With
Evaluate mgr, call `__enter__`, bind val; try body; on exception, call `__exit__(type,e,tb)` and suppress if returns true; finally call `__exit__(None,None,None)` when no exception.

### Decorators
```nyash
let f = <def>; 
f = decoN(...(deco1(f))); 
name = f
```

## Why Nyash Script First

- **Debuggability**: Human-readable Nyash is easier to inspect, diff, and map errors to; source maps stay small and precise.
- **Optimization leverage**: Nyash compiler/MIR can keep improving independently; your Python frontend benefits automatically.
- **Ecosystem fit**: Generates idiomatic Nyash that other tools (formatters, linters, analyzers) can understand; fosters a consistent DX.