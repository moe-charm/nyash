Legacy Smokes (Transition Holding Area)

Purpose
- Provide a temporary, clearly-labeled place for smokes that exist only for
  migration/compatibility during the Rust→.hako transition. Keeps active
  profiles uncluttered while preserving signal.

Policy
- Scope: migration fallbacks, deprecated APIs, or transitional diagnostics.
- Run: never included by default profiles. Invoke manually via scripts here.
- Exit: remove once the new path is stable for ≥1 release cycle.

Initial contents (wrappers)
- Map keys/values string-shim fallback
- Array.size host-slot forcing check (while HostHandleRouter phases in)

How to run
- Individually: run wrapper scripts in this directory.
- Batch: `./run_all_legacy.sh`.

