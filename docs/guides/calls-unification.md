# Calls Unification (MIR v1)

Purpose
- Unify all call-like instructions into a single `MirCall` with typed `callee`.
- Remove legacy `ExternCall`/`PluginInvoke` JSON/instructions and avoid backend divergence.

Key Points
- Use `MirInstruction::Call { callee: Some(Callee::...) }` for all calls.
- `Callee` variants: `Global`, `ModuleFunction`, `Method`, `Constructor`, `Closure`, `Value`, `Extern`.
- External/host functions are expressed as `Callee::Extern("iface.method")`.
- Box methods use `Callee::Method { box_name, method, receiver, certainty }`.

Verification
- Equality on Box types must not use `Compare(Eq|Ne)`. Use `Callee::Extern("nyrt.ops.op_eq")` instead.
- Legacy ops (`TypeCheck`, `Cast`, `WeakNew`, `WeakLoad`, `BarrierRead`, `BarrierWrite`) must be normalized; verifier rejects them when present.

JSON Emission
- JSON schema v1 emits a single `mir_call` object. No `externcall` or `plugin_invoke` appear in JSON.
- Effects are emitted as `effects: ["IO", ...]` based on the call’s effect mask and thin adapters.

Backends
- LLVM/WASM/VM consume the unified form. Method IDs may be injected post-build (resolver pass). Plugins are bridged via the unified plugin host.

Retired
- `ExternCall` instruction and `PluginInvoke` path are retired and removed. Use unified `MirCall` exclusively.

