# Hakorune CLI — Backends and Tools

This guide documents how to select backends from a single `hakorune` CLI and how tool paths are resolved.

## Backends

- Select backend with `--backend {nyvm|rust|llvm}` (aliases permitted):
  - `nyvm` → Hakorune VM (Ny implementation). Env `HAKO_NYVM_ENGINE={hakorune|mini}` (default `hakorune`).
    - `mini` selects the legacy Mini‑VM (MIR Interpreter) for dev/rc‑only.
  - `rust` or `vm` → Rust VM/Runtime (practical line).
  - `llvm` → LLVM harness / AOT line.
- Env override: `HAKO_BACKEND` (also accepts `NYASH_BACKEND`). CLI arg takes precedence in help, env is normalized internally.

Examples

- Hakorune VM (nyvm default):
  - `hakorune --backend nyvm apps/APP/main.hako`
  - `HAKO_NYVM_ENGINE=hakorune hakorune --backend nyvm apps/APP/main.hako`
- Mini‑VM (opt‑in):
  - `HAKO_NYVM_ENGINE=mini hakorune --backend nyvm apps/APP/main.hako`
- Rust VM:
  - `hakorune --backend vm apps/APP/main.hako`
- LLVM harness:
  - `hakorune --backend llvm apps/APP/main.hako`

## Tools resolution

- Inspect tools with:
  - `hakorune --which <tool>` → prints resolved path and origin
  - `hakorune --doctor tools` → quick status for `plugin-tester`, `llvm-harness`, `ny-llvmc`
- Resolution order (minimal): dist/bin → workspace (tools/*) → hako.toml → user config → ENV → PATH. Missing items are reported.

Notes
- Plugin‑on smokes SKIP gracefully when artifacts are missing (preflight + precheck).
- Mini‑VM is frozen (dev/education only). Prefer the Hakorune VM line for nyvm.
