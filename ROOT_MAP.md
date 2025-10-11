Nyash Project — Root Navigation Map (Tight Mode Friendly)

Purpose
- Reduce time/tokens to find things by giving one-page pointers.
- Non-destructive: no massive file moves; keep existing paths stable.

Top Areas
- Core runtime/compiler (Rust): `src/`
- Python LLVM harness & PyVM: `src/llvm_py/`
- Selfhost compiler (Nyash): `apps/selfhost-compiler/`
- Examples & small apps: `apps/`
- Tools & scripts: `tools/`
- Docs (reference/guides): `docs/`
- Selfhost index (no moves): `selfhost/`

High-frequency Entrypoints
- CLI runner: `src/main.rs`
- MIR builder (Rust): `src/mir/`
- LLVM harness orchestrator: `src/llvm_py/llvm_builder.py`
- PyVM exec core: `src/llvm_py/pyvm/`
- Selfhost compiler main: `apps/selfhost-compiler/compiler.hako`
- Selfhost emit (MIR v0): `apps/selfhost-compiler/boxes/mir_emitter_box.nyash`

Tight Search Tips
- Use ripgrep with .ignore: `rg -n -m 50 "pattern" path/`
- Avoid root-wide scans; prefer a subfolder (e.g. `apps/selfhost-compiler` / `src/llvm_py`).
- Need a quick tree? `ls apps/selfhost-compiler` or `rg --files apps/selfhost-compiler`.

Noise Reduced
- Root-level compiled artifacts (files starting with `app`) are hidden from rg via `.ignore`.
- `target/`, `.git/`, IDE folders are hidden.

Common Tasks
- Build (LLVM): `cargo build --release --features llvm`
- Build (JIT): `cargo build --release --features cranelift-jit`
- Selfhost quick (MIR v0): `NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 ./target/release/nyash apps/dev/selfhost_compiler_min_cmp.nyash`
- Harness IR dump (dev): `NYASH_LLVM_DUMP_IR=tmp/nyash_harness.ll ...`
- Collect artifacts (opt-in):
  - Copy CLI: `make artifacts-nyash` → `artifacts/bin/nyash`
  - Copy root apps: `make artifacts-apps` → `artifacts/apps/`
  - Clean: `make artifacts-clean`
- Move root app* physically (safe):
  - Move + symlink: `make artifacts-move`
  - Remove symlinks: `make artifacts-unlink`
  - Restore files: `make artifacts-restore`
- Place smoke outputs under artifacts:
  - `APP_BIN_DIR=artifacts/apps bash tools/llvm_smoke.sh`
  - `APP_BIN_DIR=artifacts/apps bash tools/smoke_aot_vs_vm.sh`
  - `APP_BIN_DIR=artifacts/apps bash tools/pyvm_vs_llvmlite.sh`
- Dev profile defaults:
  - `source tools/dev_env.sh pyvm` → sets `APP_BIN_DIR=artifacts/apps` (overridable) and enables tight mode (disable with `DEV_TIGHT=0`).
- Selfhost-only smokes shortcut:
  - `APP_BIN_DIR=artifacts/apps tools/selfhost_smokes.sh quick|integration`

Housekeeping
- Prefer new docs under `docs/` and small dev-only scripts under `tools/`.
- Large intermediate outputs → put under `tmp/` or `tools/tmp/` (gitignored).
