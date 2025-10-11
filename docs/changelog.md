## Changelog — Milestones and Notable Changes

### 2025-10-11 — M3 VM↔LLVM Parity (minimum)
- quick parity suite green (parity_q_*)
- Added two parity cases: JSON stringify and <=/>= boundary
- Providers print one-line digest (policy/config/loaded/anchors/stage2)
- Anchors dlsym self-check added; policy=force → Fail‑Fast when missing

### 2025-10-09 — M2 Self‑Rebuild
- Selfhost compiler EXE builds and rebuilds its own source (bootstrap path)
- EXE-first and MIR→EXE smokes pass under harness

### 2025-08-09 — Initial Commit
- Project bootstrap

