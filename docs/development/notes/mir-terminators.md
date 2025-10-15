# MIR Terminator Normalization (Builder & Emitter)

Goal: ensure every basic block ends with a proper terminator (ret/jump/branch/throw) to avoid runtime failures ("unterminated block").

What we did:
- Hako Mini‑VM path
  - BlockBuilderBox now runs a light `_ensure_terminators()` pass on all emitted modules.
  - LocalSSA copy insertion respects the first terminator and never pushes across it.
- Rust MIR path
  - finalize_module() appends a minimal return for any block that lacks a terminator. For non-void functions it returns the last defined value or `0` as a conservative fallback, then updates CFG.

Dev guard (temporary):
- Historical: a temporary guard existed to map unterminated blocks to `return void`, but it has been removed after builder/emitters guaranteed block terminators.

Rollout policy:
- Keep the pass thin and local; no behavior change for already well‑formed MIR.
- Remove the dev guard once rc suites remain green for a few cycles without it.
