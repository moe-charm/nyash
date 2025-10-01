Builders Layer — Responsibilities and Boundaries

Scope
- Block/function/instruction lowering from MIR(JSON) to llvmlite IR.
- No global orchestration, no object emission, no target policy.

Responsibilities
- function_lower.py: per‑function orchestration (blocks creation, prepasses, finalize).
- block_lower.py: per‑block lowering, including PHI collection and placement via PhiHandler.
- instruction_lower.py: instruction dispatch; keeps hot path minimal.
- phi_handler.py: PHI creation at block head; no wiring in STRICT mode.

Non‑Responsibilities
- Emitting objects or running the verifier (handled by top‑level builder/target).
- Creating PHIs during finalize; wiring only is delegated to phi_wiring.

Design Guards
- PHI nodes are only created in PhiHandler (block head). Any additional PHIs
  synthesized elsewhere must be behind a development flag and documented.
- Finalization is wire‑only (phi_wiring.finalize_phis). Creation during finalize
  is disabled by default.

Env Flags (development)
- NYASH_LLVM_PHI_STRICT=1: PhiHandler creates placeholders only; finalize wires.
- NYASH_LLVM_PHI_VERIFY=1: run phi_wiring.verify after finalize; STRICT raises.

