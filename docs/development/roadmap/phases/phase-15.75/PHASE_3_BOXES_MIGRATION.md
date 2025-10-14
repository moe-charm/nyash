# Phase 3 — Boxes Migration (Legacy → Plugins/HostHandle)

Goal
- Retire `src/boxes/` progressively by moving core functionality to plugins + HostHandleRouter.
- Keep both lines green: legacy (default) and plugin-only (verification). No default behavior change.

Strategy (structure-first)
- Guard legacy with `#[cfg(feature="legacy-boxes")]` (already in place for many modules).
- Prefer plugin/HostHandle paths when `legacy-boxes` is OFF.
- Split mixed modules (router/extern/array helpers) into `{builtin,plugin}` pieces with a small hub facade.

Acceptance
- `cargo build --release` (legacy) + quick smokes: PASS
- `cargo build --release --no-default-features -F cli,plugins,host-anchors` (plugin-only): PASS (build-only)
- plugins profile (hosthandle) representative smokes: PASS; quick stays minimal

Playbook (small steps)
1) Inventory remaining `crate::boxes::*` refs
   - `./tools/dev/list_boxes_refs.sh`
2) Gate or replace in highest-fanout sites first
   - runner, extern loader, MIR handlers (legacy-only paths behind cfg)
   - type_registry: factory functions prefer plugin when legacy OFF (done)
3) Add/adjust smokes (plugins profile) when touching routing surfaces
4) Re-run plugin-only build; iterate until refs ~0
5) Flip: default OFF for `legacy-boxes` (short-lived branch), then delete `src/boxes/`

Order of work (suggested)
- Runner/extern loader/array helpers (mostly done)
- MIR handlers legacy branches (cfg + stable diagnostics)
- Box operators/static ops (already cfg-gated for FloatBox)
- Residual: tests that import legacy boxes (gate under cfg)

Tools/Docs
- `docs/guides/plugin-only-build.md` — commands/aliases/CI stub
- `tools/dev/list_boxes_refs.sh` — refs report

Notes
- Keep ENV-based HostHandle toggles minimal and documented; remove after parity is stable.
- Prefer deleting unreachable branches once the split modules own the code paths.
