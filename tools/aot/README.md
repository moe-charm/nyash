# AOT helpers (extern_c)

Scripts in this folder assist the extern_c → AOT path:

- `emit_object_via_extern_c.sh <mir.json> <out.o>`
  - Ensures `llvm_compile_mir_to_object` is allowed and calls it via a tiny Hakorune program.
  - Respects `HAKO_FFI_LIB_PATHS` (defaults to `$(git root)/target/release`).
  - Expects the cdylib `libllvm_backend` to be built:
    - `cargo build --release -p llvm_backend`

- `link_with_clang.sh -o out_exe obj1.o [obj2.o ...] [--nyrt /path/to/libnyrt.a] [--extra "<flags>"]`
  - Minimal linker wrapper for native executables (dev only).
  - Adds conservative defaults on Linux (`-ldl -lpthread -lm`).
  - Use `--nyrt` to include a static runtime archive when available.

- `emit_ll_via_extern_c.sh <mir.json> <out.ll>`
  - Calls `llvm_compile_mir_to_ll` via extern_c to emit LLVM IR.
  - Useful for inspection, IR diffs, or custom toolchains.

- `doctor_frozen_v1.sh`
  - End-to-end sanity for the mint pipeline.
  - Emits MIR JSON from `examples/simple_return.hako`, produces `.o` via extern_c, links and runs.
  - Requires `python3`+`llvmlite`. `clang` is needed for the link step; if not found, the script reports and exits non‑zero.

Notes
- This is a convenience wrapper for development. For production pipelines consider project‑specific scripts or Makefiles.
- See `docs/guides/frozen-toolchain.md` for the end‑to‑end workflow.
