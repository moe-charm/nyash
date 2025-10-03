Title: Static box: first argument lost (critical)
Status: Open (Docs + Plan committed)
Date: 2025-10-03

Summary
- Symptom: Calling static box methods drops the first argument at runtime. The same occurs with pipe `|>` (left value is not passed as arg0).
- Impact: Any API designed as `static box` with a state argument (e.g., `start_module(st)`) fails; Builder pipelines produce null/empty output; downstream VM always returns 0.

Minimal Reproduction (Ny)
```nyash
static box Foo {
  id(x) { return x }
  add(a,b) { return a + b }
}

static box Main {
  main() {
    // Expect: 42
    print(Foo.id(42))
    // Expect: 12
    print(Foo.add(5, 7))
    // Pipe: Expect 42
    print(42 |> Foo.id)
    // Pipe: Expect 12
    print(5 |> Foo.add(7))
    return 0
  }
}
```

Observed
- All four cases print 0 or void (first arg lost / left pipe value not injected).

Root Cause (hypothesis)
- Static method call marshalling discards the first positional argument (and/or fails to prepend the pipe LHS to the args list). The receiver-less static path likely shares logic with instance/receiver handling and overwrites/omits arg0.

Plan (surgical fix)
1) Add four quick smokes under `tools/smokes/v2/profiles/quick/core/`:
   - `core_static_id_call_vm.sh` (Foo.id(42) → 42)
   - `core_static_add_call_vm.sh` (Foo.add(5,7) → 12)
   - `core_pipe_static_id_vm.sh` (42 |> Foo.id → 42)
   - `core_pipe_static_add_vm.sh` (5 |> Foo.add(7) → 12)
   All tests avoid plugins; output as plain integer.
2) Fix VM argument marshalling:
   - Normal call: args = [a,b,...] (no receiver injection)
   - Pipe call: args = [lhs, ...args]
   - Ensure static path never mixes receiver into args.
3) Dev trace (temporary): log argc and the first two arg values at the call dispatch site; remove after green.
4) Re-run quick; ensure all four pass; remove dev traces.

Temporary Workaround (in repo)
- Added instance-style builder `apps/selfhost/common/json/mir_builder2.hako` that keeps internal state (`me.st`) and does not rely on passing a builder state via args. Use this for emit paths while the VM bug is being fixed.

Risks
- None to stable quick profile once the fix is in; smokes are isolated and plugin-free.

Rollback
- Disable the new smokes if needed; revert arg marshalling patch (single site) if regression observed.

