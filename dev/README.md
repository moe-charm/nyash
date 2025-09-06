Nyash Dev Areas

This folder contains isolated development workspaces that do not interfere with the main source tree. Use these for experiments and focused bring‑up.

Areas

- selfhosting/: JIT self‑hosting pipeline experiments (Ny → MIR → MIR‑Interp → VM/JIT). Includes quickstart notes and scripts references.
- cranelift/: Cranelift JIT/AOT bring‑up and AOT link experiments; smokes and env toggles.

Notes

- Keep experiments and artifacts inside each subfolder. Avoid modifying the core `src/` unless changes are ready to graduate.
- Prefer scripts under `tools/` and add thin wrappers here if needed.

