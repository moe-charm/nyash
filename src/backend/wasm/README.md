# WASM Backend — Status & Plan

Status
- Codegen covers: const/binop/compare/return/print and a few BoxCall stubs (toString/print/equals/clone, console.log).
- Not covered yet: ArrayBox and MapBox method families (get/set/len/push/values ...).
- Plugins are cfg-disabled on wasm32; HostHandle is not applicable.

Plan (Phase‑A)
- Implement ArrayBox minimal semantics in codegen:
  - slots 100(get)/101(set)/102(len)/103(push)/105(clear)
  - linear memory layout: [len, cap, data...]
  - defensive OOB behavior (return 0/null, no panic) to match VM tolerances
- Keep StringBox minimal (const/toString/substring/indexOf), expand in Phase‑B.
- MapBox limited or SKIP in Phase‑A.

Design Guards
- Keep TypeRegistry slot ids as SSOT — do not hardcode divergent ids here.
- No plugin path on wasm32 — enforce via cfg and ProviderBox policy.

Testing
- Add a tiny wasm parity smoke (auto‑SKIP when toolchain missing): len→push→len prints "2".
