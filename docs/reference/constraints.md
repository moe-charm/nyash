# Nyash Constraints & Temporary Limitations

This is a living index of known constraints. Each entry includes status and references to tests and code. Update this file when a constraint is added or lifted.

Legend
- Status: Stable | Temporary | Resolved | Experimental
- Impact: VM / LLVM / PyVM / Macro / Parser

Entries

## CF-JOIN-0001 — If-join PHI variables limit
- Status: Resolved
- Summary: If-join used to effectively handle at most two same‑name variable assignments per join when emitting PHI groups.
- Impact: LLVM harness (PHI wiring)
- Fix: Finalize‑PHI wiring + join result observation; normalized to handle N variables.
- Tests (v2): use `tools/smokes/v2/run.sh --profile integration` (LLVM PHI invariants covered in integration suite)
- Notes: Keep IR hygiene smokes minimal in CI; more exhaustive coverage can run locally.

## CF-PHI-0002 — Empty PHI sanitize switch
- Status: Temporary
- Summary: Text‑level sanitizer drops empty PHI rows before LLVM parse.
- Impact: LLVM harness only
- Gate: `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`
- Exit criteria: PHI wiring guarantees no empty PHIs across Loop/If/Match; remove sanitize path.
- Tests (v2): covered by `tools/smokes/v2` integration runs; legacy scripts were removed

## CF-LOOP-0006 — Nested bare blocks with break/continue in loops
- Status: Resolved
- Summary: Previously, a `break`/`continue` inside a nested bare block (`{ ... }`) within a loop could bypass loop-aware lowering in certain cases.
- Impact: MIR builder (LoopBuilder vs generic block handling)
- Fix: LoopBuilder now lowers `Program` nodes by recursing through statements with termination checks; `break/continue` inside nested blocks route to the loop header/exit uniformly.
- Tests (v2): covered in `tools/smokes/v2` macro cases (legacy paths removed)

## CF-MATCH-0003 — Scrutinee single evaluation
- Status: Stable
- Summary: Scrutinee is evaluated once and stored in a gensym (e.g., `__ny_match_scrutinee_X`).
- Impact: Parser/Normalizer/All backends
- Tests (v2): goldens remain; execution smokes are under `tools/smokes/v2` (legacy paths removed)
- Notes: Golden comparison may normalize gensym names in the future to reduce brittleness.

## EXC-PFX-0004 — Postfix catch/cleanup precedence
- Status: Stable (Stage‑3 gate for parser acceptance)
- Summary: Postfix attaches to the immediately preceding expression (call/chain) and stops further chaining. Normalizes to a single TryCatch.
- Impact: Parser/Normalizer/All backends
- Gate: `NYASH_PARSER_STAGE3=1` (direct parsing); `NYASH_CATCH_NEW=1` (sugar normalization)
- Tests (v2): see `tools/smokes/v2` and `src/tests/parser_expr_postfix_catch.rs` (legacy paths removed)

## MACRO-CAPS-0005 — Macro sandbox capabilities (io/net/env)
- Status: Stable MVP
- Summary: Macro child runs in a sandbox. Only minimal boxes and console externs allowed. IO/NET require caps; env access controlled via ctx/env.
- Impact: PyVM macro child / Macro runner
- Env: `NYASH_MACRO_CAP_{IO,NET,ENV}`; `NYASH_MACRO_SANDBOX=1`
- Tests: macro goldens/smokes; env‑tag demo (`tools/test/golden/macro/env_tag_string_user_macro_golden.sh`)

How to add an entry
1) Allocate an ID with a prefix domain (CF/EXC/MACRO/RES/…)
2) Fill status, impact, gates, tests
3) Reference PR/commit in the change log (optional)
