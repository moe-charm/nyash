Dev Mode and Defaults (Plan)

Overview
- Goal: make Nyash simple to run like other languages. Defaults should be obvious; experiments should be opt‑in.
- Two modes:
  - Production (default): quiet, stable; only stable features on.
  - Development (`--dev`): safe dev defaults on; experiments enabled in observe mode; helpful diagnostics.

Current
- Use `--dev` to enable development defaults:
  - `nyash --dev script.nyash`
  - Enables AST using + Operator Boxes (observe) by default. Output remains stable.
- Dev shortcuts remain available:
  - `./tools/opbox-json.sh` – JSON Roundtrip/Nested with Operator Boxes
  - `./tools/opbox-quick.sh` – Quick suite with Operator Boxes
- Using guard:
  - Duplicate `using` (same file imported twice or alias rebound) is a hard error with file:line hints.
  - Fix by removing/consolidating duplicates.

Defaults mapping
- Production (default)
  - using: ON (SSOT). AST merge only when nyash.toml defines the module (no implicit).
  - Operator Boxes: OFF (no behavior change from legacy).
  - Traces: OFF.
- Development (`--dev`)
  - using: AST merge ON (SSOT + AST prelude enabled by default).
  - Operator Boxes: observe ON (stringify/compare/add calls visible; results not adopted → output is stable).
  - Traces: OFF by default; can enable selectively via `NYASH_TRACE` (to be introduced) or legacy flags.
  - Duplicate `using`: error (with line numbers).

Flag consolidation (mid‑term)
- `NYASH_OPERATOR_BOXES=all|none|comma-list` → expands to legacy flags.
- `NYASH_TRACE=all|vm,box,print,if,loop` → expands to legacy trace flags.
- Old flags remain; new keys take precedence; final resolved state is logged when `NYASH_CLI_VERBOSE=1`.

nyash.toml profiles (long‑term)
- Example:
  [profiles.json-dev]
  operator_boxes = ["stringify", "compare", "add"]
  using_ast = true
  trace = []

  [profiles.debug]
  operator_boxes = "all"
  trace = ["vm", "box", "if", "loop"]
  verbose = true

Priority
- CLI `--profile` > explicit env > nyash.toml defaults > built‑in defaults.

Acceptance
- `nyash script.nyash` runs with stable defaults.
- `nyash --dev script.nyash` enables AST using + Operator Boxes (observe) and passes JSON Roundtrip/Nested.
- Smokes use `--dev` path when appropriate; profiles remain as a convenience.
