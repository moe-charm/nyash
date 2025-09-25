# Plugin ABI (by-id / tagged) — Snapshot

This summarizes the ABI surfaces used by LLVM in Phase 15. Details live in NyRT (`crates/nyrt`).

## Fixed-arity by-id shims
- Integer-dominant: `i64 @nyash_plugin_invoke3_i64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, i64 a1, i64 a2, i64 a3, i64 a4)`
- Float-dominant: `f64 @nyash_plugin_invoke3_f64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, f64 a1, f64 a2, f64 a3, f64 a4)`

## Tagged shims (mixed types)
- Fixed (<=4 args): `i64 @nyash_plugin_invoke3_tagged_i64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, i64 a1, i64 t1, i64 a2, i64 t2, i64 a3, i64 t3, i64 a4, i64 t4)`
- Vector (N args): `i64 @nyash.plugin.invoke_tagged_v_i64(i64 type_id, i64 method_id, i64 argc, i64 recv_h, i8* vals, i8* tags)`

Tag codes (minimal):
- 3=int, 5=float, 8=handle(ptr). Others are reserved/experimental.

## Return mapping (LLVM lowering)
- If destination is annotated as Integer/Bool → keep i64 as integer.
- If destination is String/Box/Array/Future/Unknown → cast i64 handle to opaque pointer for SSA flow; do not `inttoptr` where a C string is expected.

## Notes
- These ABIs are used by both built-ins (nyrt) and plugins for consistency.
- The LLVM backend is the reference; other backends will be aligned later.

