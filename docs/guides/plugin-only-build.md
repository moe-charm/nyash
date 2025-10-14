# Plugin‑Only Build (Phase 15.6 – Transition Guide)

Purpose
- Build without legacy `src/boxes/` to validate plugin/HostHandleRouter paths.
- Use for local verification; CI remains unchanged until references are cleared.

Terminology (Phase 15.75)
- Rust line: legacy built-ins enabled (current default). Stable dev line.
- Hakorune line: plugin-only runtime (legacy built-ins disabled). Migration/verification line.

How to try (experimental)
- Default features include `legacy-boxes`. Disable them and opt‑in required features:

```
# Plugin‑only (no legacy boxes)
cargo build --release --no-default-features -F cli,plugins,host-anchors
```

Convenient aliases (optional)

```
# .cargo/config.toml (suggested aliases)
[alias]
# Rust line (legacy built-ins ON)
build-rust = "build --release"
run-rust = "run --release"

# Hakorune line (plugin-only)
build-hako = "build --release --no-default-features -F cli,plugins,host-anchors"
run-hako = "run --release --no-default-features -F cli,plugins,host-anchors --"
```

Expected outcomes
- If build fails, remaining `crate::boxes::*` references are present.
- Use the helper to list references:

```
./tools/dev/list_boxes_refs.sh
```

Staging plan
- plugins profile: HostHandleRouter paths ON (Map/Array/String) + Stage‑2 HostHandle arrays.
- quick profile: minimal HostHandle (Array.size) ON; broaden gradually (Map.size/has → get/set → String len/others).
- Once `crate::boxes` references are zero, remove `legacy-boxes` feature and delete `src/boxes/`.

Dual build lines (Phase 15.75)
- Keep two official build lines to reduce confusion:
  - Rust line (default): `cargo build --release` or `cargo build-rust`
  - Hakorune line (verification): `cargo build-hako`
- CI (minimal):
  - Job A: legacy build + quick smokes
  - Job B: plugin‑only build (build‑only) to ensure guard coverage

Frozen toolchain (Phase 15.76)
- Long‑term, switch daily development to a frozen EXE (see `docs/guides/frozen-toolchain.md`).
- The Rust line is kept only to mint the frozen binary and for emergencies.

Notes
- ENV toggles for HostHandleRouter are development‑only and will be removed once unified paths are stable.
- Keep plugin configs explicit (e.g., `NYASH_PLUGIN_CONFIG=hako.toml`) to avoid implicit loads when testing.

Limitations (current)
- `env.future`/`VMValue::Future` are legacy‑only (guarded). Plugin‑only returns explicit errors for legacy externs.
- Builtin arms in method router (`FileBox`/`CallableBox`/`ArrayBox`/`MapBox`) are legacy‑only; plugin‑only uses plugin paths.

Minimal CI example (build‑only)
```
name: plugin-only-build
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build (plugin-only)
        run: cargo build --release --no-default-features -F cli,plugins,host-anchors
```

Smokes (optional, build‑only)

```
# Run build‑only check under plugins profile (optional developer step)
tools/smokes/v2/run.sh --profile plugins --filter plugin_only_build_check.sh

# If initial build is cold and may exceed default per‑test timeout,
# the smoke supports a per‑test header timeout. You can also pass it via CLI:
SMOKES_DEFAULT_TIMEOUT=180 tools/smokes/v2/run.sh --profile plugins --filter plugin_only_build_check.sh
```
