# Nyash Smoke Tests v2 — Guide

Overview
- Entry: `tools/smokes/v2/run.sh` — unified runner for quick/integration/full.
- Profiles:
  - `quick` — fast developer checks.
  - `integration` — VM↔LLVM parity, basic stability.
  - `full` — comprehensive matrix.

Dev Mode (defaults)
- In v2 smokes, the `quick` profile exports `NYASH_DEV=1` by default.
  - This enables CLI `--dev`-equivalent defaults inside Nyash:
    - AST using ON (SSOT + AST prelude merge)
    - Operator Boxes in observe mode (no adoption)
    - Minimal diagnostics; output parity is preserved
- You can also run manually with `nyash --dev script.nyash`.

Common commands
- Quick suite (auto `NYASH_DEV=1`):
  - `tools/smokes/v2/run.sh --profile quick`
- Focus JSON smokes:
  - `tools/opbox-json.sh` (Roundtrip/Nested, plugins disabled, generous timeout)
- One-off program (VM):
  - `target/release/nyash --backend vm --dev apps/APP/main.nyash`

Key env knobs
- `NYASH_DEV=1` — enable dev defaults (same effect as `--dev`).
- `SMOKES_DEFAULT_TIMEOUT` — per test timeout seconds (default 15 for quick).
- `SMOKES_PLUGIN_MODE=dynamic|static` — plugin mode for preflight (auto by default).
- `SMOKES_FORCE_CONFIG=rust_vm_dynamic|llvm_static` — force backend config.
- `SMOKES_NOTIFY_TAIL` — lines to show on failure tail (default 80).

Notes
- Dev defaults are designed to be non-intrusive: tests remain behavior‑compatible.
- To repro outside smokes, either pass `--dev` or export `NYASH_DEV=1`.

Heavy JSON probes
- Heavy JSON tests (nested/roundtrip/query_min) run a tiny parser probe first.
- The probe's stdout last non-empty line is trimmed and compared to `ok`.
- If not `ok`, the test is SKIP (parser unavailable), not FAIL. This avoids
  false negatives due to environment noise or optional dependencies.
