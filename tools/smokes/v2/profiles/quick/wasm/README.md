Quick/WASM — wasm harness smokes (gated)

Purpose
- Keep WASM-related quick checks isolated. This avoids coupling with VM/LLVM smokes and simplifies merges.

Gating (default SKIP)
- Set either of the following to enable tests in this folder:
  - `SMOKES_ENABLE_WASM=1`
  - or `NYASH_WASM_USE=1`

Conventions
- Always source `lib/test_runner.sh` and run `require_env` + `preflight_plugins`.
- Tests must be short (< 1s) and self-contained. If a harness/binary is missing, SKIP with a clear message.

Notes
- This folder is safe to keep in both main and wasm-development branches; merging is typically conflict-free.

