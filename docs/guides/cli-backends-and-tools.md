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

## NyVM Pipe (MIR JSON direct)

- Execute MIR(JSON v0) via Hakorune VM Core directly (Gate C thin wiring):
  - From file: `hakorune --nyvm-json-file tmp/mir.json`
  - From stdin: `cat tmp/mir.json | hakorune --nyvm-pipe`
  - The wrapper uses `selfhost/hakorune-vm/hakorune_vm_core.hako` and prints the numeric result line at the end.

## Builder → LLVM (one‑liner)

- With python3 + llvmlite installed, you can emit LLVM IR from a MIR JSON file:
  - `python3 tools/llvmlite_harness.py --in tmp/mir.json --emit-ll --out tmp/out.ll`
  - To generate `tmp/mir.json` from the in‑repo builder quickly:
    - `hakorune -c $'using "selfhost/shared/mir/block_builder_box.hako" as B; using "apps/lib/json_native/stringify.hako" as J; static box Main { main() { print(J.stringify_map(B.binop(2,3,"Add"))); return 0 } }' | tail -n1 > tmp/mir.json`

## Tools resolution

- Inspect tools with:
  - `hakorune --which <tool>` → prints resolved path and origin
  - `hakorune --doctor tools` → quick status for `plugin-tester`, `llvm-harness`, `ny-llvmc`
- Resolution order (minimal): dist/bin → workspace (tools/*) → hako.toml → user config → ENV → PATH. Missing items are reported.

Notes
- Plugin‑on smokes SKIP gracefully when artifacts are missing (preflight + precheck).
- Mini‑VM is frozen (dev/education only). Prefer the Hakorune VM line for nyvm.
