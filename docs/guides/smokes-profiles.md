# Smoke Profiles — Quick, Integration, Full

Purpose
- Keep everyday checks fast (quick), validate backend parity in a scoped way (integration), and aggregate broader coverage when needed (full).

Profiles
- `quick`: small, fast checks for core paths. Expected runtime: ~1–2 minutes.
- `integration`: VM ↔ LLVM parity set. Expected runtime: ~5–10 minutes.
- `full`: Aggregates quick + integration + plugins + suites/* (if present). Expected runtime: 15–30 minutes depending on env.

How to run
- `tools/smokes/v2/run.sh --profile quick`
- `tools/smokes/v2/run.sh --profile integration`
- `tools/smokes/v2/run.sh --profile full`

Environment flags (minimal)
- `SMOKES_FORCE_LLVM=1`: Force LLVM path in parity tests (skip detection).
- `SMOKES_PROFILE_ENV=<name>`: Loads `tools/smokes/v2/configs/<name>.env` for defaults.
- `SMOKES_DEFAULT_TIMEOUT=<sec>`: Per-test timeout.
- `NYASH_USING=1`: Using resolver ON (default for smokes).

Parity harness availability
- Parity scripts call a shared gate `require_llvm_or_skip`.
  - If LLVM feature or Python harness(llvmlite) is unavailable, tests are SKIPped.
  - Set `SMOKES_FORCE_LLVM=1` to always run the LLVM path (useful in CI with harness).

Notes
- Noise filtering is conservative; user output lines aren’t stripped. If differences surface, prefer tightening filters locally in the script.
- For environment variables beyond the above, see `docs/guides/env-variables.md`.
