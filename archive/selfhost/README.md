Self‑Hosting (Compiler & Smokes)

Purpose
- Provide one place to discover all self‑hosting bits without moving files.
- Avoid large refactors; keep existing paths stable and CI green.

Where things live
- Compiler (Ny): `apps/selfhost-compiler/`
- Mini‑VM for MIR v0: `apps/selfhost/vm/boxes/mir_vm_min.nyash`
- Dev samples: `apps/dev/` (e.g., `mir_cfg_branch_smoke.nyash`)
- Smokes (v2 runner): `tools/smokes/v2/` (filter by prefix `selfhost_*`)

Quick run
- Preferred: enable Selfhost profile (tight mode + compiler-track + artifacts):
  - `source tools/dev_env.sh selfhost`
- Quick selfhost smokes:
  - `tools/selfhost_smokes.sh quick`
- Integration (VM ↔ LLVM harness) selfhost smokes:
  - `tools/selfhost_smokes.sh integration`

Notes
- No files were physically moved; this directory is an index for humans.
- Use test filter `--filter "selfhost_*"` with `tools/smokes/v2/run.sh` to target selfhost cases.
 - To revert env toggles: `source tools/dev_env.sh reset`
