# apps/selfhost/ — Deprecated (migrated to selfhost/)

This directory is kept temporarily during the migration to the new top‑level `selfhost/` layout.

- New locations:
  - Shared MIR helpers: `selfhost/shared/mir/`
  - Compiler pipeline v2: `selfhost/compiler/pipeline_v2/`
  - Mini‑VM boxes: `selfhost/vm/boxes/`, VM scripts in `selfhost/vm/`

Action items
- Prefer module aliases (selfhost.*) over file‑path usings.
- Update remaining references before removing this directory in a future sprint.

