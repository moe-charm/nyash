
# Async Model — nowait / await (Phase 15.7)

Intent
- Keep MIR as SSOT: Await/FutureNew/FutureSet instructions with safepoint invariants.
- Lowering differences are handled per backend; semantics remain fixed.

Backends
- Rust VM (vm):
  - Await: native handler (Future.get() or pass-through if non-Future).
  - nowait: builder lowers to env.future.spawn_instance; VM returns a resolved Future (pseudo-async). Future scheduler hookup is reserved.
- LLVM (llvmlite harness):
  - Await/Future*: rewritten to env.future.* via NYASH_REWRITE_FUTURE=1.
  - env.future.await: thin special-case in Python builder; IR remains well-typed.
  - env.future.spawn_instance: thin stub; IR only.
- Hakorune VM (nyvm):
  - Runner selects --backend mir with HAKO_NYVM_ENGINE=hakorune.
  - The bridge compiles MIR JSON and runs via Hakorune core; semantics align with VM/LLVM path (pseudo-async for nowait).

Flags
- HAKO_CALLABLE_ASYNC=1: enable builtin async for CallableBox.callAsync (experimental job-queue).
- NYASH_REWRITE_FUTURE=1: force rewrite Await/Future* to env.future.* (set automatically for LLVM mode).

Verification
- Verifier enforces Safepoint before/after Await.
- Effects: Await is READ+Async.

Roadmap
- Connect TaskGroup.spawn and Future.get() tick for cooperative scheduling (flag guarded).
- Cranelift: lower Await natively; LLVM keeps function-call rewrite.
