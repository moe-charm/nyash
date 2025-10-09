MIR Verification — Legacy Forbid and Invariants (Phase 19)

Scope
- Summarizes the key verifier checks that enforce the unified MIR shape and catch regressions early.

Legacy instructions (forbidden)
- TypeCheck / Cast → TypeOp
- BarrierRead / BarrierWrite → Barrier
- Print → Call + Extern("env.console.log")
- PluginInvoke → BoxCall/Method
- ArrayGet / ArraySet → BoxCall("get"/"set") [when Core‑13 or `NYASH_MIR_ARRAY_BOXCALL=1`]
- RefGet / RefSet → BoxCall("getField"/"setField") [when `NYASH_MIR_REF_BOXCALL=1`]

Behavior
- By default, the verifier rejects functions that still contain legacy ops.
- Set `NYASH_VERIFY_ALLOW_LEGACY=1` to bypass during bring‑up (not recommended for CI).

Other invariants (selection)
- Box Compare(Eq/Ne) forbidden: Box equality must be lowered to a call (either `.equals/1` or `Extern("nyrt.ops.op_eq")`).
- Static box fields forbidden: Disallow `getField/setField` when the receiver is a constant class name (namespace only policy).
- PHI inputs coverage (dev): With `NYASH_VERIFY_PHI_STRICT=1`, every PHI must cover all reachable predecessors; empty PHIs are invalid.

Dev tips
- Enable MIR verification in the VM with `NYASH_VM_VERIFY_MIR=1` to get early errors near execution.
- Use `tools/ny_doctor.sh` for a quick environment sanity check.

