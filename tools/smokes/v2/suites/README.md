Suites — logical grouping of smokes

This folder organizes tests by purpose, independent from profiles:

- core: language/core behaviors (backend-agnostic)
- mir: MIR-level constructs (phi/merge/reachability)
- vm: VM-specific smokes (vm:rust / vm:hakorune)
- llvm: LLVM harness/AOT/IR related smokes
- plugins: dynamic/static plugin E2E
- experimental: PoC/legacy; gated by env (off by default)

Profiles (quick/integration/full) select subsets across suites. The runner now accepts
`--profile full` to aggregate quick+integration+plugins and any suites present.

