# MIR Builder Modules (Current Split)

This note summarizes the current split of the MIR builder to keep `builder.rs` thin and maintainable.

Paths:
- `src/mir/builder.rs`: Thin hub for MIR building; owns state (generators, maps) and high‑level coordination.
- `src/mir/builder/stmts.rs`: Statement builders
  - `build_print_statement`, `build_block`, `build_if_statement`, `build_loop_statement`,
    `build_try_catch_statement`, `build_throw_statement`, `build_local_statement`,
    `build_return_statement`, `build_nowait_statement`, `build_await_expression`, `build_me_expression`.
- `src/mir/builder/ops.rs`: Expression ops
  - `build_binary_op`, `build_unary_op`, `convert_binary_operator`, `convert_unary_operator`.
- `src/mir/builder/utils.rs`: Shared utilities
  - `resolve_include_path_builder`, `builder_debug_enabled`, `builder_debug_log`, `infer_type_from_phi`.
- Calls: `src/mir/builder/builder_calls.rs`
  - `build_function_call`, `build_method_call`, `build_from_expression`,
    `lower_method_as_function`, `lower_static_method_as_function`,
    `parse_type_name_to_mir`, `extract_string_literal`.

Notes:
- `builder.rs` now stays < 1,000 LOC by delegating to the above modules.
- No behavior change intended; only mechanical movement. jit‑direct smokes remain green.
- Debug logs remain gated by `NYASH_BUILDER_DEBUG=1`.

Run checks:
- Build (JIT): `cargo build --release --features cranelift-jit`
- jit‑direct smokes:
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-phi-min/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-multi/main.nyash`

