# Self-Hosting Front MVP — Ny → JSON v0

Status: drafting (Phase 15.9 entry)

## Purpose
- Deliver a Ny-written front-end that lowers Stage-3 Ny source into JSON v0 accepted by the existing MIR/VM/LLVM lines.
- Keep Core (`src/`) untouched; all implementation resides under `apps/selfhost-compiler/`.
- Fail fast: unsupported syntax must abort with a precise diagnostic, never fall back to Rust-side parser.

## Scope (Day 1–2)
- Expressions: integer literal, boolean literal, null, binary ops (`+ - * / %`, comparisons `== != < <= > >=`), logical `&& ||`, grouped `(...)`.
- Statements: `return`, `local`, `assignment`, `if/else`, `loop` (while-style), `break`, `continue`.
- Calls: direct function call (`foo()`), method call sugar (`obj.method()`), builtin boxes (Array/Map/String minimal) deferred to Day 3–4.
- Metadata: `meta.usings` populated from `ParserBox.extract_usings` (already implemented).

## Interfaces
| box | responsibility |
| --- | --- |
| `ParserBox` | Tokenize/parse Ny source into Stage-1 AST JSON. Already handles Stage-2 constructs; ensure Stage-3 gating by `stage3_enable`. |
| `EmitterBox` | Convert Stage-1 AST JSON to JSON v0 Program. Must guarantee single-line output when `NYASH_JSON_ONLY=1`. |
| `MirEmitterBox` | Out of scope for Day 1–2 (stub remains). |

### Contracts
- `ParserBox.parse_program2(src)` returns a `Program` JSON string with `version/kind/body/meta` keys.
- `EmitterBox.emit_program(ast_json, usings_json)` injects metadata and *must* avoid altering the AST when `ast_json` already conforms to JSON v0.
- `Main.main(args)` (`apps/selfhost-compiler/compiler.hako`) orchestrates: read source → parse → emit → print.

## Deliverables
1. **Documentation**
   - This file (living roadmap).
   - Update `CURRENT_TASK_SELFHOST.md` with checklists and timeline (done).
   - README snippet explaining how to run the front MVP via `hakorune` flags.
2. **Code**
   - Extend `ParserBox` where gaps exist (Stage-3 gating, diagnostics).
   - Implement JSON v0 normalization helpers (Box-first: `emit/json_program_box.hako` TBD).
   - Optional: add `apps/selfhost-compiler/boxes/json_program_box.hako` as a thin aggregator to keep emitter lean.
3. **Tests**
   - Quick smokes: `tools/smokes/v2/profiles/quick/selfhost/selfhost_min_const_ret_vm.sh` etc.
   - Harness comparator: `tools/smokes/v2/run_selfhost_min.sh` to run VM/LLVM and diff `Result:` lines.

## Fail-Fast Checklist
- Parser errors include position info (gpos) and stop compilation.
- Unsupported statements/expressions raise `StageUnsupported` error (box-based), not silent drop.
- JSON builder ensures key order (`version`, `kind`, `body`, `meta`) for deterministic output.

## Box Layout Proposal
```
apps/selfhost-compiler/
├── boxes/
│   ├── parser_box.hako        # existing Stage-1 parser
│   ├── emitter_box.hako       # metadata injector
│   ├── json_program_box.hako  # (new) JSON v0 construction helpers
│   └── error_box.hako         # diag helpers (fail-fast)
├── emitter/
│   └── json_v0.nyash          # refactored to delegate to JsonProgramBox
└── docs/
    └── front_mvp.md           # this file
```

## Execution Examples
Emit-only (JSON header):
```
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_MIN_JSON=1 \
NYASH_NY_COMPILER_EMIT_ONLY=1 \
NYASH_JSON_ONLY=1 \
./target/release/hakorune --backend vm apps/tests/selfhost_min/const_ret.hako
```
Expected output: `{"version":0,...}` single-line JSON.

Execute via VM (Result comparison):
```
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_EMIT_ONLY=0 \
NYASH_SELFHOST_READ_TMP=1 \
./target/release/hakorune --backend vm apps/tests/selfhost_min/const_ret.hako
```
Expected output: `Result: 42`

## Timeline (rolling)
- **Day 1**: Docs + const/return + binop. Add selfhost_min_const_ret smoke.
- **Day 2**: if/else, loop sum smoke, branch diagnostics.
- **Day 3**: Calls/BoxCall minimal, DP integration reuse.
- **Day 4**: externcall string primitives, verify via `run_llvm_extended.sh`.
- **Day 5**: CLI polish + docs cleanup + bench harness alignment.

## Open Questions
- Should Stage-3 acceptance (`break`, `continue`, `throw`) be gated by CLI flag? (Currently via `--stage3`.)
- JSON v0 normalization: do we normalize binary op names (`+` → `Add`) or keep literal symbols? (Existing Stage-1 uses symbols; MIR builder already maps them.)
- How much of `ParserBox` should move into dedicated modules for maintainability? (Deferred; focus on MVP shipping.)
