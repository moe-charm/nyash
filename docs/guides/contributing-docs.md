# Contributing Docs — Small, Linked, 3‑Layer

Status: Stable  |  Updated: 2025‑09‑21  |  Scope: Docs structure/policy

TL;DR
- Keep docs small. Use 3 layers: Overview → Reference → Details.
- No duplication: overview links to the single canonical reference.
- Every page shows Status/Updated/Scope and has a short summary.

Layers
- Overview (design one‑pager)
  - What/Why/How in bullets, ≤1 page; links to Reference/Details/Guides.
- Reference (docs/reference/)
  - Canonical spec: invariants, API, acceptance rules. Precise and stable.
- Details (docs/design/ or docs/development/…)
  - Background, alternatives, rationale. Optional; link from overview only.

Authoring Rules
- One canonical spec per topic (in reference/). Others must link to it.
- Each directory has a README.md that points to its key one‑pagers.
- Cross‑links go under “See also” (≤3 items, relative paths).

One‑pager Template
- Title / Status / Updated / Scope
- TL;DR (3–5 lines)
- What (spec bullets)
- How (integration points, ownership boundaries)
- Links (Reference / Details / Guides)
- Notes (constraints / future work)

Examples
- Using→Loader overview: docs/development/design/legacy/using-loader-integration.md
- Mini‑VM roadmap: docs/development/roadmap/phases/phase-17-loopform-selfhost/MINI_VM_ROADMAP.md
