# Testing Matrix — Mapping Specs to Tests

Purpose
- Map invariants/constraints to the concrete tests (smokes/goldens/unit) that verify them.

Categories
- PHI hygiene (LLVM)
  - ir_phi_empty_check.sh — no empty PHIs
  - ir_phi_hygiene_if_phi_ret.sh — PHIs at block head with if/ret pattern
- MIR hints (VM)
  - hints_trace_smoke.sh — basic scope enter/leave
  - hints_join_result_* — join diagnostics for 2/3 vars
  - hints_scope_trycatch_smoke.sh — try/catch scopes
- Match normalization (VM/goldens)
  - match_literal_basic / literal_three_arms output smokes
  - match_guard_literal_or / type_basic_min goldens
- Exceptions (VM)
  - expr_postfix_catch_cleanup_output_smoke.sh — postfix direct parser
  - loop_postfix_catch_cleanup_output_smoke.sh — combined with loops
- LoopForm break/continue (VM)
  - loopform_continue_break_output_smoke.sh — basic continue/break
  - loop_nested_if_ctrl_output_smoke.sh — nested if inside loop
  - loop_nested_block_break_output_smoke.sh — nested bare block with break

Maintenance
- When adding an invariant or lifting a constraint, update this matrix and link the tests.
