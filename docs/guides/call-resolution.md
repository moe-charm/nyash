# Call Resolution — Strict by Structure

Purpose: eliminate ambiguous name binding by splitting responsibilities:

- Generation: `CallNameNormalizerBox` produces canonical names `Box.method/Arity`.
- Resolution: `ModuleFunctionResolverBox` resolves canonical names strictly.

Policy

- Dotted names must resolve by exact match. If arity is omitted, append `/argc`.
- Tail fallback is OFF by default. Opt in with `NYASH_VM_GLOBAL_TAIL_FALLBACK=1`.
- Bare (non‑dotted) names are treated as legacy global calls.

Builder

- Always build static names via `CallNameNormalizerBox::static_name(Box, method, arity)`.
- Reject invalid identifiers (ASCII `[A-Za-z_][A-Za-z0-9_]*`).
- For static calls, resolve with `ModuleFunctionResolverBox::resolve_strict`.
- Do not emit legacy method‑only unique candidates; keep a single legacy switch.

VM

- Global dotted names: require exact canonical `Class.method/Arity`.
- Optional tail fallback (class prefix / alias‑alias) only when the env is ON.
- Dev aid: `NYASH_VM_REENTER_TRACE=1`, `NYASH_VM_REENTER_LIMIT=N` to diagnose cycles.

Migration Notes

- `JsonFragBox.block0segment` alias is removed. Use `block0_segment`.
- If a legacy artifact still produces alias‑alias `X_X.method`, normalize at the source.

Flags

- `NYASH_VM_GLOBAL_TAIL_FALLBACK=1` — enable dotted tail fallback (legacy only).
- `NYASH_VM_REENTER_TRACE=1` / `NYASH_VM_REENTER_LIMIT=N` — reentrancy diagnostics.

Tests (targeted)

- `tools/smokes/v2/profiles/quick/core/json_v0_const_ret_vm.sh`
- `tools/smokes/v2/profiles/quick/core/json_v0_if_return_phi_vm.sh`
- `tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_multi_compare_gt_last_ret_vm.sh`
