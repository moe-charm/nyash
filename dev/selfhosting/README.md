Self‑Hosting Dev (JIT / VM)

Focus: Ny → MIR → MIR‑Interp → VM/JIT quick loops to validate semantics and bootstrap paths.

Quickstart

- Core build (JIT):
  - `cargo build --release --features cranelift-jit`
- Core smokes (plugins disabled):
  - `NYASH_CLI_VERBOSE=1 ./tools/jit_smoke.sh`
- Roundtrip (parser pipe + json):
  - `./tools/ny_roundtrip_smoke.sh`
- Plugins smoke (optional gate):
  - `NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh`
- Using/Resolver E2E sample (optional):
  - `./tools/using_e2e_smoke.sh` (requires `--enable-using`)
- Bootstrap c0→c1→c1' (optional):
  - `./tools/bootstrap_selfhost_smoke.sh`

Flags

- `NYASH_DISABLE_PLUGINS=1`: stabilize core path
- `NYASH_LOAD_NY_PLUGINS=1`: enable nyash.toml ny_plugins
- `NYASH_ENABLE_USING=1`: using/namespace enable
- `NYASH_SKIP_TOML_ENV=1`: suppress [env] mapping in nyash.toml

Tips

- For debug, set `NYASH_CLI_VERBOSE=1`.
- Keep temp artifacts under this folder (`dev/selfhosting/_tmp/`) to avoid polluting repo root.

