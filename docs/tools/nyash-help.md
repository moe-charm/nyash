# `nyash --help` Snapshot (HakoRune CLI alias `hrn` available)

Captured: 2025-08-23
Source: Built-in clap help from the `nyash` binary (aka HakoRune)

```
🦀 HakoRune Programming Language (aka Nyash) — Everything is Box in Rust! 🦀

Usage: nyash [OPTIONS] [FILE]

Arguments:
  [FILE]  Nyash file to execute

Options:
      --debug-fuel <ITERATIONS>  Set parser debug fuel limit (default: 100000, 'unlimited' for no limit) [default: 100000]
      --dump-mir                 Dump MIR (Mid-level Intermediate Representation) instead of executing
      --verify                   Verify MIR integrity and exit
      --mir-verbose              Show verbose MIR output with statistics
      --backend <BACKEND>        Choose execution backend: 'interpreter' (default), 'vm', or 'llvm' [default: interpreter]
      --compile-wasm             Compile to WebAssembly (WAT format) instead of executing
      --compile-native           Compile to native AOT executable using wasmtime precompilation
      --aot                      Short form of --compile-native
  -o, --output <FILE>            Output file (for WASM compilation or AOT executable)
      --benchmark                Run performance benchmarks across all backends
      --iterations <COUNT>       Number of iterations for benchmarks (default: 10) [default: 10]
      --vm-stats                 Enable VM instruction statistics (equivalent to NYASH_VM_STATS=1)
      --vm-stats-json            Output VM statistics in JSON format
  -h, --help                     Print help
  -V, --version                  Print version
```

関連: CLIオプション早見表は `docs/tools/cli-options.md`
