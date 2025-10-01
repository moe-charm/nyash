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
- NYASH_JSON_SCHEMA_V1=1      # enable JSON v1 (mir_call) emission (shape/dev only)
- NYASH_LLVM_DOWNGRADE_V1=1   # when set, force v1→v0 downgrade for harness emit
 - Downgrade extern fallback: when NYASH_LLVM_DOWNGRADE_V1=1 and a v1 Global callee is not defined
   in the current module, the harness emitter maps it to a legacy `externcall` in v0. This keeps
   compile-only runs green while VM/AOT remain Fail‑Fast (unresolved Global is an error at exec).

PHI Sanitize (grouping at block head)
- Harness compile enforces LLVM invariant「PHI はブロック先頭でグループ化」
  - The builder performs a light IR‑text sanitize before verification when `NYASH_LLVM_USE_HARNESS=1` (or `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`):
    - Drop malformed empty PHIs
    - Group all PHIs at the top of each basic block (preserving relative order)
  - Motivation: resolver/legacy finalize may synthesize late PHIs for localized values; sanitize normalizes ordering without changing semantics.

Status
- Harness path is stable for object emission and PHI diagnostics.
- Parity-by-execution with LLVM remains optional until NyKernel minimal ABI is finalized.

Smokes (compile-only)
- PHI invariants (STRICT=1):
  - tools/smokes/v2/profiles/quick/llvm/phi_if_merge_compile_ok.sh
  - tools/smokes/v2/profiles/quick/llvm/phi_loop_compile_ok.sh
- v1→v0 downgrade gate:
  - tools/smokes/v2/profiles/quick/llvm/harness_v1_downgrade_call_compile_ok.sh
  - tools/smokes/v2/profiles/quick/llvm/harness_v1_downgrade_global_extern_compile_ok.sh
