# MIR Builder (modularized) — Archived

Status: Archived. The optional feature `mir_modular_builder` and the module `src/mir/builder_modularized/*` have been removed.

Reason:
- The active MIR builder lives under `src/mir/builder/` and `src/mir/builder.rs`.
- CI and default builds never enabled the modular builder feature; it diverged from the main path.

Where to look now:
- Current builder: `src/mir/builder/` and `src/mir/builder.rs`
- MIR cleanup plan/spec: `docs/development/roadmap/phases/phase-11.8_mir_cleanup/{PLAN.md,TECHNICAL_SPEC.md}`

Notes:
- Any historical design details can be recovered from git history. This page exists as a breadcrumb for past references.
