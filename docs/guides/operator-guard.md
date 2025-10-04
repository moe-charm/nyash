# Operator Box Guard (Compare/Add) — Boundary and Policy

Purpose
- Centralize adoption/interception of operator boxes (e.g., `CompareOperator.apply/3`).
- Keep hot paths native by default; adopt only under explicit flags.

Design
- Module: `src/backend/mir_interpreter/operator_guard.rs`
- Entry hook: `MirInterpreter::operator_guard_intercept_entry(func_name, arg_vals)` is called at the top of `exec_function_inner` before any frame/regs setup.
- Current behavior: Always intercept `CompareOperator.apply/*` and evaluate via VM native `eval_cmp` (root‑fix phase). This avoids re‑entry and frame corruption during bring‑up.

Flags (future)
- `NYASH_OPERATOR_BOX_COMPARE_ADOPT=1` — enable CompareOperator adoption once parity is ensured.
- `NYASH_BUILDER_OPERATOR_BOX_COMPARE_CALL=1` — builder lowering to operator calls (kept OFF during root‑fix).

Tests
- `tools/smokes/v2/profiles/quick/core/vm_compare_semantics_vm.sh` — native compare semantics.
- `tools/smokes/v2/profiles/quick/core/jsonscan_seek_array_end_vm.sh` — regression where operator re‑entry previously corrupted params.

Policy
- Default: native compare; operator boxes are optional/flag‑guarded.
- Observation must not cause re‑entry; use guard or events, not nested calls.

## Non‑Reentry Policy (Mini‑VM)
- VM エントリで OperatorGuard が演算・比較の観測/採用判定を一元化。
- 下位ハンドラでは再度の観測/再入は行わない（副作用の重複を防ぐ）。
- Compare/Arithmetic/Unary はガード配下でネイティブ評価に委譲。
