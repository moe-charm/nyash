LLVM Harness-First Policy (Phase 15)

Summary
- Prefer the Python/llvmlite harness for LLVM checks in development and quick smokes.
- Keep Rust VM as the default execution path for day-to-day parity and semantics.
- AOT/executable linking via NyKernel is optional and guarded; object-level compile is sufficient for quick checks.

Why
- Fast iteration without large native toolchain dependencies.
- Deterministic PHI/SSA diagnostics (JSONL trace) and smaller surface for regressions.
- Decouples IR generation/validation from native linking issues.

How to run (object-level)
- Build:
  - cargo build --release
  - cargo build --release -p nyash-llvm-compiler
- Run harness:
  - PYTHONPATH="$PWD" NYASH_NY_LLVM_COMPILER="$PWD/target/release/ny-llvmc" NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/mir-phi-min/main.nyash
- Expected: “Compiled to tmp/nyash_llvm_run.o” or “object written” messages. No executable is required for quick checks.

Optional: AOT executable
- Build ny kernel (either name is accepted):
  - cargo build --release -p nyash-llvm-compiler
  - cargo build --release -p nyash_kernel  (legacy) or cargo build --release -p hako_kernel (new)
 - Link executable by using ny-llvmc with --emit exe. ny-llvmc auto‑detects `libnyash_kernel.a`/`libhako_kernel.a` in `target/release/`.
 - Quick examples:
   - tools/smokes/v2/profiles/quick/llvm/aot_const_ret_exe.sh
   - tools/smokes/v2/profiles/quick/llvm/aot_compare_branch_exe.sh

Smokes
- Quick profile should rely on harness for compile success (object generation) and IR/trace validation, not executable comparison.
- Integration parity (VM vs LLVM) requires either native backend or full NyKernel linking; keep it optional and gated.

Env flags
- NYASH_LLVM_USE_HARNESS=1    # enable harness path
- NYASH_LLVM_PHI_STRICT=1     # PhiHandler: create-only; wiring unified in finalize
- NYASH_LLVM_TRACE_PHI=1      # PHI JSONL diagnostics (optionally set NYASH_LLVM_TRACE_OUT)

Status
- Harness path is stable for object emission and PHI diagnostics.
- Parity-by-execution with LLVM remains optional until NyKernel minimal ABI is finalized.
