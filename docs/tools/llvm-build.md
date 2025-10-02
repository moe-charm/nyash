# LLVM Build Quick Guide (llvmlite harness)

Purpose
- Build and run the LLVM (llvmlite) harness line locally for AOT object emit and parity checks.

Requirements
- System LLVM 18 (`llvm-config-18`) on PATH
- Python 3 + `llvmlite` installed (`pip install llvmlite`)

Build steps
- Build harness compiler and core with LLVM feature:
```
cargo build --release -p nyash-llvm-compiler
cargo build --release --features llvm
```

Run examples
- Harness-first (emit object + run via harness):
```
NYASH_LLVM_USE_HARNESS=1 \
NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc \
NYASH_EMIT_EXE_NYRT=target/release \
./target/release/hakorune --backend llvm apps/tests/phi_loop_simple.nyash
```
- Emit an object only:
```
NYASH_LLVM_USE_HARNESS=1 \
NYASH_LLVM_OBJ_OUT=$PWD/target/aot_objects/demo.o \
./target/release/hakorune --backend llvm apps/tests/phi_loop_simple.nyash
```

Notes
- If harness is unavailable, smoke scripts SKIP gracefully.
- Use `NYASH_LLVM_DUMP_IR=tmp/ir.ll` to dump the IR text for inspection.
- PHI safety: keep `NYASH_LLVM_SANITIZE_EMPTY_PHI=1` for development.
