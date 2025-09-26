# PHI and SSA in Nyash

Overview
- Nyash lowers high-level control flow (If/Loop/Match) to MIR and backends that rely on SSA form.
- We prioritize IR hygiene and observability while keeping runtime cost at zero.

Design points
- PHI hygiene: no empty PHIs; PHIs at block head only.
- JoinResult hint: when both branches assign the same variable, we emit a MIR hint for diagnostics.
- Loop carriers: loops may expose a carrier observation (≤ N variables, where N is unconstrained by design; smokes emphasize common cases).

Normalization
- If: may optionally wrap into LoopForm under a conservative gate (dev only). Semantics remain unchanged.
- Match: scrutinee evaluated once, guard fused; normalized to nested If‑chain in macro/core pass.

Testing
- LLVM smokes: fixed small cases ensure no empty PHIs and head placement.
- MIR smokes: trace `scope|join|loop` to validate shaping without peeking into IR details.

Roadmap
- Remove text-level sanitization once finalize‑PHI is trustworthy across Loop/If/Match.
- Expand goldens to cover nested joins and multi‑carrier loops while keeping CI light.

