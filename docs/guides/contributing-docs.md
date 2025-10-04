# Contributing Docs — Small, Linked, 3‑Layer

Status: Stable  |  Updated: 2025‑10‑04  |  Scope: Docs structure/policy

TL;DR
- Keep docs small. Use 3 layers: Overview → Reference → Details.
- No duplication: overview links to the single canonical reference.
- Every page shows Status/Updated/Scope and has a short summary.

Layers
- Overview (design one‑pager)
  - What/Why/How in bullets, ≤1 page; links to Reference/Details/Guides.
- Reference (docs/reference/)
  - Canonical spec: invariants, API, acceptance rules. Precise and stable.
- Details (docs/design/, docs/architecture/, or docs/development/…)
  - Background, alternatives, rationale. Optional; link from overview only.
  - **Stable** (completed): docs/design/, docs/architecture/
  - **Active** (in-progress): docs/development/design/, docs/development/architecture/

Authoring Rules
- One canonical spec per topic (in reference/). Others must link to it.
- Each directory has a README.md that points to its key one‑pagers.
- Cross‑links go under "See also" (≤3 items, relative paths).

Document Placement (Stable vs Active)
- **New design/architecture docs** → Start in `docs/development/design/` or `docs/development/architecture/`
- **Completed & stable docs** → Move to `docs/design/` or `docs/architecture/`
- **Reference specs** → Always in `docs/reference/` (never move)
- **Archived docs** → Move to `docs/archive/` when superseded

One‑pager Template
- Title / Status / Updated / Scope
- TL;DR (3–5 lines)
- What (spec bullets)
- How (integration points, ownership boundaries)
- Links (Reference / Details / Guides)
- Notes (constraints / future work)

Examples
- **Stable design** (completed): docs/design/using-and-dispatch.md
- **Active architecture** (in-progress): docs/development/architecture/mir-callee-revolution.md
- **Active design** (in-progress): docs/development/design/extern-vs-boxcall.md
- **Reference spec** (canonical): docs/reference/language/LANGUAGE_REFERENCE_2025.md
- **Roadmap** (planning): docs/development/roadmap/phases/phase-17-loopform-selfhost/MINI_VM_ROADMAP.md
