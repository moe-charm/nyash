# Nyash AOT-Plan (Phase 15.1) — Scripts Skeleton

This folder will contain Nyash scripts that analyze a project (following `using` imports) and emit `aot_plan.v1.json` per docs/development/design/legacy/aot-plan-v1.md.

Phase 15.1 scope:
- Keep scripts minimal and deterministic
- Do not invoke native toolchains
- Output a single plan JSON for small smokes

Placeholder files included:
- `analyze.ny` — entry script (skeleton)
- `samples/mini_project/` — a tiny sample project with `using`
- `samples/plan_v1_min.json` — a minimal plan JSON used by importer tests
