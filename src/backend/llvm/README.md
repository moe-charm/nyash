# ⚠️ DEPRECATED: Legacy Rust/inkwell LLVM backend

This directory contains the historical Rust/inkwell-based LLVM backend.

- Development focus has shifted to the Python/llvmlite backend under `src/llvm_py/`.
- Keep this code for reference only. Do not extend or modify for current tasks.
- The LLVM build pipeline now prefers the llvmlite harness (`NYASH_LLVM_USE_HARNESS=1`).

If you need to reduce build time locally, consider using the harness path and
building only the core crates. See `tools/build_llvm.sh` and `tools/compare_harness_on_off.sh`.

