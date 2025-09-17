# Parser/Bridge: Unary and ASI Alignment (Stage‑2)

Context
- Rust parser already parses unary minus with higher precedence (parse_unary → factor → term) but PyVM pipe path did not reflect unary when emitting MIR JSON for the PyVM harness.
- Bridge（JSON v0 path）is correct for unary by transforming to `0 - expr` in the Python MVP, but Rust→PyVM path uses `emit_mir_json_for_harness` which skipped `UnaryOp`.
- ASI in arguments split over newlines is not yet supported in Rust (e.g., newline inside `(..., ...)` after a comma in a chained call), while Bridge/Selfhost cover ASI for statements and operators.

Proposed minimal steps
- Unary for PyVM harness:
  - Option A (preferred later): extend `emit_mir_json_for_harness[_bin]` to export a `unop` instruction and add PyVM support. Requires schema change.
  - Option B (quick): legalize unary `Neg` to `Const(0); BinOp('-', 0, v)` before emitting, by inserting a synthetic temporary. This requires value id minting in emitter to remain self‑consistent, which we currently do not have. So Option B is non‑trivial without changing emitter capabilities.
  - Decision: keep Bridge JSON v0 path authoritative for unary tests; avoid relying on Rust→PyVM for unary until we add a `unop` schema.

- ASI inside call arguments (multi‑line):
  - Keep as NOT SUPPORTED for Rust parser in Phase‑15. Use single‑line args in tests.
  - Selfhost/Bridge side already tolerate semicolons optionally after statements; operator‑continuation is supported in Bridge MVP.

Tracking
- If we want to support unary in the PyVM harness emitter:
  - Add `unop` to tools/pyvm_runner.py and src/llvm_py/pyvm/vm.py (accept `{op:"unop", kind:"neg", src: vid, dst: vid}`)
  - Teach emitters to export `UnaryOp` accordingly (`emit_mir_json_for_harness[_bin]`).

Status
- Bridge unary: OK（ny_stage2_bridge_smoke includes unary）
- Rust→PyVM unary: not supported in emitter; will stay out of CI until schema update
- ASI in args over newline: not supported by Rust parser; keep tests single‑line for now
