# Phase 15.1 — Self-Hosting AOT-Plan to MIR13 (Nyash-first)

Scope: Extend Phase 15 (Self-Hosting Ny → MIR → VM/JIT) with a small, safe subphase that keeps VM/JIT primary, and prepares a future AOT pipeline by introducing:

- Nyash scripts that perform AOT preflight analysis across `using` trees
- A stable AOT-Plan JSON schema (functions, externs, exports, type hints, link units)
- A compiler-side importer that lowers AOT-Plan → MIR13 skeletons (no object code yet)

Avoid: deep AOT emission/linking, cross-platform toolchain work, or scope creep beyond planning and MIR13 import.

## Phase 15 Goals (context)

- VM-first correctness; JIT as compiler-only, not required for AOT deliverables
- Minimal policy: read-only JIT-direct default, deny mut hostcalls unless whitelisted
- Observability: compile/runtime JSONL events; counters for lower-time fallbacks
- Short-circuit lowering and core collection ops kept consistent

## Phase 15.1 Objectives

- AOT-Plan via Nyash scripts (self-hosted analysis):
  - Parse sources with `using` resolution; build function inventory and extern references
  - Compute minimal “link unit” groups (per file or per module) pragmatically
  - Produce `aot_plan.v1.json`

- MIR13 importer:
  - Read AOT-Plan → create MIR13 functions with signatures and extern stubs
  - Leave bodies empty or minimal where applicable; execution stays VM/JIT

- Smokes:
  - `plan: using` on 2–3 small Nyash projects; output deterministic JSON
  - Import the plan and run via VM to confirm pipeline integrity (no AOT emit)

## Deliverables

- `tools/aot_plan/` Nyash scripts and helpers
- `docs/design/aot-plan-v1.md` (lightweight schema)
- Compiler entry to import AOT-Plan → MIR13 (feature-gated)
- 3 smokes + 1 golden JSON sample

## Out of Scope (Phase 15.1)

- Object emission, linkers, archive/rpath, platform toolchains
- Non-trivial inliner/optimizer passes dedicated to AOT

## Milestones

1) AOT-Plan schema v1
- Minimal fields: functions, externs, exports, units, types(optional)
- Golden JSON example committed

2) Nyash analyzer (self-hosted)
- Walk `using` graph; collect symbols and extern refs
- Output `aot_plan.v1.json`

3) Importer to MIR13
- Map functions → MIR13 signatures and extern call placeholders
- Feature-gate import; maintain VM/JIT run with consistency

4) Smokes + Observability
- 3 projects → stable plan JSON; importer round-trip builds MIR
- Emit `jit::events` low-volume markers: `aot_plan.import`, `aot_plan.analyze`

## Risk & Guardrails

- Risk: scope creep to AOT emit → Guard: no obj/link in 15.1
- Risk: importer expands semantics 
  → Guard: stub bodies only; effects mask conservative; VM reference behavior unchanged
- Risk: plan schema churn → Guard: v1 frozen; add `extensions` map for future keys

---

## Consultation Notes (Gemini / Claude)

Prompts used:
- Gemini: "Phase 15 self-hosting goals (VM-first/JIT-compiler-only). Propose a 2-week 15.1 scope to add Nyash-driven AOT preflight that outputs a stable AOT plan JSON, plus a MIR13 importer—no object emission. Include milestones, acceptance criteria, and guardrails to prevent scope creep. Keep implementation incremental and observable."
- Claude: "Given an existing Ny parser and using-resolution, design a minimal AOT planning pipeline as Ny scripts that produces a plan.json (functions/externs/exports/units). Define the MIR13 importer requirements and tests ensuring VM/JIT behavior remains canonical. Provide risks, do-not-do list, and minimal telemetry." 

Key takeaways aligned into this document:
- Keep 15.1 as a planning/import phase with strong do-not-do
- Make plan JSON stable and small; let importer be skeletal
- Add clear smokes and counters; avoid new runtime semantics

## Acceptance Criteria

- `tools/aot_plan` can analyze a small project with `using` and emit deterministic JSON
- Importer can read that JSON and construct MIR13 module(s) without panics
- VM runs those modules and matches expected string/number results for trivial bodies
- Events present when enabled; counters reflect plan/import activity; no AOT emit performed
