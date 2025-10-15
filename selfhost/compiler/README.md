selfhost/compiler — Selfhost compiler pipeline

Responsibilities
- Parse/normalize/emit MIR.
- Enforce Fail‑Fast (SSA ensure_cond/ensure_calls) at boundaries.

Forbidden
- No runtime execution or VM specific logic.
- No plugin loading; pure emit only.
