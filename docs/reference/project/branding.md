# Project Branding — HakoRune (aka Nyash)

Status: Adopted (non‑breaking)

- Official name: HakoRune
- CLI short name: `hrn` (nickname: `hako`)
- Backward‑compat: existing `nyash` naming remains valid across code, env, and config.

## Compatibility rules

- Environment variables
  - Official brand prefix: `HAKO_*`
  - Aliases are accepted automatically: `HAKO_*`, `HAKU_*`, and `HRN_*` mirror into `NYASH_*` when the latter is unset.
  - Example: `HAKO_CLI_VERBOSE=1` is equivalent to `NYASH_CLI_VERBOSE=1`.
  - Root path alias: `NYASH_ROOT` falls back to `HAKO_ROOT`/`HAKU_ROOT`/`HRN_ROOT`.

- Configuration file
  - Preferred: `hako.toml`
  - Loader searches in order: `hako.toml` (CWD) → `nyash.toml` → `hakorune.toml` → `$NYASH_ROOT/hako.toml` → `$NYASH_ROOT/nyash.toml` → `$NYASH_ROOT/hakorune.toml`.
  - Kernel (exe‑side) also checks neighbors in the same order.

- Per‑plugin box specs
  - Preferred: `hako_box.toml` (under the plugin directory)
  - Compatibility: `nyash_box.toml` is still accepted when `hako_box.toml` is absent.

- CLI
  - Binary name remains `nyash` in this branch. For local use, create an alias: `ln -s ./target/release/nyash hrn`.
  - Future packaging may ship `hrn` as an additional binary name.

## Rationale

- Box‑First philosophy aligned: “Hako” (箱) + “Rune” (記号/核)。
- Non‑breaking adoption maintains scripts and docs while enabling the new brand.
