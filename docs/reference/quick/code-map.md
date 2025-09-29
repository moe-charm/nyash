# Code Map (Backend VM) — Quick Guide

Purpose: Make it obvious where to look without reading everything.

## Backend Modules (VM)
- `src/backend/vm.rs`: Core VM struct, execute loop, storage, control-flow.
- `src/backend/vm_instructions.rs`: Instruction handlers called from `execute_instruction`.
- `src/backend/vm_values.rs`: Value-level ops (binary/unary/compare), boolean coercions.
- `src/backend/vm_boxcall.rs`: Box method dispatch (`call_box_method_impl`), BoxCall debug logger。
- `src/backend/vm_phi.rs`: Loop/phi utilities (LoopExecutor)。
- `src/backend/vm_stats.rs`: Stats/diagnostics printing（JSON/Text, envで制御）。

## MIR Pipeline (where to check next)
- `src/mir/printer.rs`: `--mir-verbose(--effects)`の出力。
- `src/mir/verification.rs`: SSA/支配/CFG/merge-phi + WeakRef/Barrier 最小検証＋Strict Barrier診断。
- `src/mir/optimizer.rs`: DCE/CSE/順序調整 + 未lowering検知（is/as 系）。

## Useful env flags
- `NYASH_VM_DEBUG_BOXCALL=1`: BoxCallの受け手/引数/結果型をstderrに出力。
- `NYASH_VM_STATS=1` (`NYASH_VM_STATS_JSON=1`): 実行統計を表示（JSON可）。
- `NYASH_VERIFY_BARRIER_STRICT=1`: Barrierの軽い文脈診断を有効化。
- `NYASH_OPT_DIAG_FAIL=1`: Optimizerの未lowering検知でエラー終了（CI向け）。

Last updated: 2025-08-25

