This folder contains reproducibility artifacts for Paper A (MIR13 IR design).

Files
- `COLLECT_ENV.sh`: Captures host OS/CPU/toolchain/git info into `ENVIRONMENT.txt`.
- `RUN_BENCHMARKS.sh`: Runs interpreter/VM/JIT/AOT (if available) against sample benchmarks and writes CSVs to `results/`.
- `results/`: Output CSVs (per benchmark and per mode). Merge/plot as needed.

Usage
1) Capture environment
   ./COLLECT_ENV.sh

2) Build (full)
   cargo build --release --features cranelift-jit

3) Run benchmarks
   ./RUN_BENCHMARKS.sh

   Variables:
   - NYASH_BIN: Path to nyash binary (default: target/release/nyash)
   - USE_EXE_ONLY=1: Only measure AOT executables (skips interp/vm/jit)

Notes
- AOT requires `tools/build_aot.sh`. If missing, AOT is skipped.
- If `hyperfine` is not installed, a simple timing fallback is used.

