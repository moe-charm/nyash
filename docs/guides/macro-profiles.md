# Macro Profiles — Simple, Practical Defaults

Profiles simplify CLI/ENV by choosing sensible defaults for macro execution.

Profiles
- lite
  - macros: OFF
  - strict: OFF
  - trace: OFF
- dev (default behavior)
  - macros: ON
  - strict: ON
  - trace: OFF
- ci / strict
  - macros: ON
  - strict: ON
  - trace: OFF

CLI
- `nyash --macro-profile dev file.nyash`
- `nyash --macro-profile ci-fast file.nyash`
- `nyash --macro-profile strict file.nyash`
- または開発向けショートカット: `nyash --dev file.nyash`（macro ON/strict ON ほか開発既定も有効）

ENV mapping (non-breaking; can be overridden)
- lite → `NYASH_MACRO_ENABLE=0`, `NYASH_MACRO_STRICT=0`, `NYASH_MACRO_TRACE=0`
- dev → `NYASH_MACRO_ENABLE=1`, `NYASH_MACRO_STRICT=1`, `NYASH_MACRO_TRACE=0`
- ci-fast/strict → `NYASH_MACRO_ENABLE=1`, `NYASH_MACRO_STRICT=1`, `NYASH_MACRO_TRACE=0`

Recommended ENV (minimal)
- `NYASH_MACRO_ENABLE=1`
- `NYASH_MACRO_PATHS=...`
- `NYASH_MACRO_STRICT=1`
- `NYASH_MACRO_TRACE=0|1`

Deprecated ENV (kept for compatibility for now)
- `NYASH_MACRO_BOX_NY=1` + `NYASH_MACRO_BOX_NY_PATHS=...` → use `NYASH_MACRO_PATHS`
- `NYASH_MACRO_BOX_CHILD_RUNNER` → no longer needed
- `NYASH_MACRO_TOPLEVEL_ALLOW` → default OFF; prefer CLI `--macro-top-level-allow` when necessary

