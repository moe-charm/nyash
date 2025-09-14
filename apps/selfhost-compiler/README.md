# Nyash Selfhost Compiler (MVP scaffold)

This is the Phase 15.3 work-in-progress Nyash compiler implemented in Ny.

Layout
- `compiler.nyash`: entry (CompilerBox). Reads `tmp/ny_parser_input.ny`, prints JSON v0.
- `parser/`: lexer/parser/ast (scaffolds; to be filled as we extend Stage‑2)
- `mir/`: builder/optimizer stubs (future; current target is JSON v0 emit)
- `tests/`: Stage‑1/2 samples (TBD)

Run (behind flag)
- `NYASH_USE_NY_COMPILER=1 target/release/nyash --backend vm <program.nyash>`
  - The runner writes the input to `tmp/ny_parser_input.ny` and invokes this program.
  - It captures a JSON v0 line from stdout and executes it via the JSON bridge.

Notes
- Early MVP emits a minimal JSON v0 (currently a placeholder: return 0). We will gradually wire lexer/parser/emitter.
- Keep JSON v0 spec in `docs/reference/ir/json_v0.md`.
