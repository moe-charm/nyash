# Nyash Refactor Roadmap (Pre–Self-Hosting)

This document lists large modules, proposes safe splits/commonization, and outlines the MIR13 cleanup plan.

## Large Modules to Split

Targets are chosen by size and cohesion. Splits are incremental and build-preserving; move code in small steps and re-export in `mod.rs`.

- `src/mir/verification.rs` (~965 loc)
  - Split into: `mir/verification/{mod.rs,basic.rs,types.rs,control_flow.rs,ownership.rs}`.
  - First move leaf helpers and pass-specific checks; keep public API and `pub use` to avoid churn.

- `src/mir/builder.rs` (~930 loc)
  - Split into: `mir/builder/{mod.rs,exprs.rs,stmts.rs,decls.rs,control_flow.rs}`.
  - Extract expression/statement builders first. Keep tests (if any) colocated.

- `src/mir/instruction.rs` (~896 loc)
  - Near-term: introduce `mir/instruction/{mod.rs,core.rs,ops.rs,calls.rs}` without changing the enum surface.
  - Medium-term: migrate to MIR13 (see below) and delete legacy variants.

- `src/mir/optimizer.rs` (~875 loc)
  - Split passes into: `mir/optimizer/{mod.rs,constant_folding.rs,dead_code.rs,inline.rs,type_inference.rs}`.
  - Keep a simple pass runner that sequences the modules.

- `src/runner/mod.rs` (~885 loc)
  - Extract modes into `runner/modes/{vm.rs,jit.rs,mir_interpreter.rs,llvm.rs}` if not already, and move glue to `runner/lib.rs`.
  - Centralize CLI arg parsing in a dedicated module.

- `src/backend/vm_instructions/boxcall.rs` (~881 loc)
  - Group by box domain: `boxcall/{array.rs,map.rs,ref.rs,weak.rs,plugin.rs,core.rs}`.
  - Long-term: most of these become `BoxCall` handlers driven by method ID tables.

## MIR13 Cleanup Plan

A large portion of pre-MIR13 variants remain. Current occurrences:

- ArrayGet: 11, ArraySet: 11
- RefNew: 8, RefGet: 15, RefSet: 17
- TypeCheck: 13, Cast: 13
- PluginInvoke: 14, Copy: 13, Debug: 8, Print: 10, Nop: 9, Throw: 12, Catch: 13, Safepoint: 14

Phased migration (mechanical, testable per phase):

1) Introduce shims
   - Add `BoxCall` helpers covering array/ref/weak/map ops and plugin methods.
   - Add `TypeOp::{Check,Cast}` modes to map legacy `TypeCheck/Cast`.

2) Replace uses (non-semantic changes)
   - Replace within: `backend/dispatch.rs`, `backend/mir_interpreter.rs`, `backend/cranelift/*`, `backend/wasm/codegen.rs`, `mir/printer.rs`, tests.
   - Keep legacy variants in enum but mark Deprecated for a short period.

3) Tighten verification/optimizer
   - Update `verification.rs` to reason about `BoxCall/TypeOp` only.
   - Update optimizer patterns (e.g., fold Copy → Load/Store; drop Nop/Safepoint occurrences).

4) Delete legacy variants
   - Remove `ArrayGet/Set, RefNew/Get/Set, PluginInvoke, TypeCheck, Cast, Copy, Debug, Print, Nop, Throw, Catch, Safepoint`.
   - Update discriminant printer and state dumps accordingly.

Use `tools/mir13-migration-helper.sh` to generate per-file tasks and verify.

## Commonization Opportunities

- Backend dispatch duplication
  - `backend/dispatch.rs`, `backend/vm.rs`, and Cranelift JIT lowerings handle overlapping instruction sets. Centralize instruction semantics interfaces (traits) and keep backend-specific execution and codegen in adapters.

- Method ID resolution
  - `runtime/plugin_loader_v2` and backend call sites both compute/lookup method IDs. Provide a single resolver module with caching shared by VM/JIT/LLVM.

- CLI/runtime bootstrap
  - Move repeated plugin host init/logging messages into a small `runtime/bootstrap.rs` with a single `init_plugins(&Config)` entry point used by all modes.

## Suggested Order of Work

1. Split `mir/verification` and `mir/builder` into submodules (no behavior changes).
2. Add `BoxCall` shims and `TypeOp` modes; replace printer/dispatch/codegen uses.
3. Update verification/optimizer for the unified ops.
4. Delete legacy variants and clean up dead code.
5. Tackle `runner/mod.rs` and `backend/vm_instructions/boxcall.rs` splits.

Each step should compile independently and run `tools/smoke_vm_jit.sh` to validate VM/JIT basics.

