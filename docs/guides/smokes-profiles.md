# Smoke Profiles — Quick, Integration, Full

Purpose
- Keep everyday checks fast (quick), validate backend parity in a scoped way (integration), and aggregate broader coverage when needed (full).

Profiles
- `quick`: small, fast checks for core paths. Expected runtime: ~1–2 minutes.
- `integration`: VM ↔ LLVM parity set. Expected runtime: ~5–10 minutes.
- `full`: Aggregates quick + integration + plugins + suites/* (if present). Expected runtime: 15–30 minutes depending on env.
  - `integration-core`: Core-only parity (no plugins). Includes short-circuit, type ops, equality, and basic compare.

How to run
- `tools/smokes/v2/run.sh --profile quick`
- `tools/smokes/v2/run.sh --profile integration`
- `tools/smokes/v2/run.sh --profile full`
 - Dual parser (opt‑in): set `SMOKES_PARSER_MODE=rust|hako|both` (default `rust`). Example:
   - `SMOKES_PARSER_MODE=both tools/smokes/v2/run.sh --profile quick-selfhost --filter 'parser_facade_*|selfhost_min_json_header_vm.sh'`
- HostHandleRouter boundary suite (plugins profile):
  - `tools/smokes/v2/run.sh --profile plugins --filter 'hosthandle_boundary_*'`
  - To observe return type mismatch (-14) boundary with a test hook:
    - `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1 tools/smokes/v2/run.sh --profile plugins --filter hosthandle_return_type_mismatch_vm.sh`
 - Parser facade (quick-selfhost, opt-in):
   - `tools/smokes/v2/run.sh --profile quick-selfhost --filter 'parser_facade_*'`

Environment flags (minimal)
- `SMOKES_FORCE_LLVM=1`: Force LLVM path in parity tests (skip detection).
- `SMOKES_PROFILE_ENV=<name>`: Loads `tools/smokes/v2/configs/<name>.env` for defaults.
- `SMOKES_DEFAULT_TIMEOUT=<sec>`: Per-test timeout.
- `NYASH_USING=1`: Using resolver ON (default for smokes).

Dynamic plugin set
- `SMOKES_REQUIRED_PLUGINS` — required plugin keys (space or comma separated). Defaults to core set.
  - Example: `SMOKES_REQUIRED_PLUGINS="stringbox arraybox mapbox setbox" tools/smokes/v2/run.sh --profile plugins`
  - Runner maps keys → crates and builds only the required subset (`cargo build -p ...`). Missing artifacts are warned and tests may SKIP.

Parity harness availability
- Parity scripts call a shared gate `require_llvm_or_skip`.
  - If LLVM feature or Python harness(llvmlite) is unavailable, tests are SKIPped.
  - Set `SMOKES_FORCE_LLVM=1` to always run the LLVM path (useful in CI with harness).

Notes
- Noise filtering is conservative; user output lines aren’t stripped. If differences surface, prefer tightening filters locally in the script.
- For environment variables beyond the above, see `docs/guides/env-variables.md`.

Module Resolution (Selfhost)
- Prefer module resolution via `hako.toml [modules]` and workspace `hako_module.toml [exports]`.
- Quoted module using is supported: `using "selfhost.shared.mir.builder" as BlockBuilderBox;`
  - The resolver first looks up the module name in `[modules]`/workspace exports, then falls back to file lookup only if not matched.
- Development ENV `NYASH_MODULES` is being phased out; it remains available for temporary, local overrides but should not be relied on in new tests.
- Quick‑selfhost profile no longer injects builder/schema into `NYASH_MODULES` by default; tests that need them should either:
  - add an explicit `using "selfhost.shared.mir.builder" as BlockBuilderBox;`, or
  - define entries under `[modules]` / workspace exports.


Selfhost Opt‑In (gated)
- Purpose: keep quick/CI green while allowing developers to run heavier selfhost checks locally.
- Flags:
  - `SMOKES_SELFHOST_ENABLE=1`: enable selfhost quick tests (alias/pipeline/oop/etc.). Default OFF → tests SKIP.
  - `SMOKES_SELFHOST_M2M3_ENABLE=1`: enable selfhost Mini‑VM M2/M3 tests (JSON→Mini‑VM eval). Default OFF → tests SKIP.
- Rationale: these suites depend on local modules, plugin availability, and evolving emit paths. Gating avoids incidental reds.
- Example:
  - `SMOKES_SELFHOST_ENABLE=1 tools/smokes/v2/run.sh --profile quick-selfhost`
  - `SMOKES_SELFHOST_M2M3_ENABLE=1 tools/smokes/v2/run.sh --profile quick-selfhost --filter 'selfhost_mir_*'`

Plugin‑On Strict (dev)
- Strict plugin semantics tests require dynamic plugins (.so) to be present.
- We preflight with plugin tester when available:
  - `tools/plugin-tester/target/release/plugin-tester build-all`
  - If plugin artifacts are missing, strict tests SKIP (not FAIL) to keep quick profile green.
- You can force a rebuild locally to run them:
  - `tools/plugin-tester/target/release/plugin-tester build-all`


CI Guidance (recommended)
- Defaults (fast + stable):
  - `quick` + `integration-core`
  - Avoid enabling plugin-on strict or selfhost suites in default CI to keep signal clean.
- Opt-in (developers / nightly):
  - Selfhost: set `SMOKES_SELFHOST_ENABLE=1` (and `SMOKES_SELFHOST_M2M3_ENABLE=1` for Mini‑VM M2/M3)
  - Plugin-on strict: ensure dynamic plugin artifacts exist or run `tools/plugin-tester/target/release/plugin-tester build-all` first.
- Rationale: isolates evolving surfaces (plugins/selfhost) from core regressions while preserving easy local verification.
