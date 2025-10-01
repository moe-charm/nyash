# Async Task System (Structured Concurrency) — Overview

Goal: A safe, structured, and portable async task system that runs end‑to‑end across Nyash code → MIR → VM → JIT/EXE.

- Default is safe: tasks are scoped to an owning group; when the owner ends, children cancel and join.
- Everything is Box: TaskGroup and Future are Boxes; user APIs are Box methods; MIR uses BoxCall.
- No new MIR ops required: use BoxCall/PluginInvoke consistently; safepoints are inserted around await.
- Deterministic exits: parent exit triggers cancelAll → joinAll on children (LIFO), eliminating leaks.

This folder contains the spec, phase plan, and test plan:

- SPEC.md: User API, Box contracts, MIR/VM/JIT mapping, ABI, error semantics.
- PLAN.md: Phased rollout (P1–P3), acceptance gates and checklists.

