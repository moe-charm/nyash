Layer Guard — selfhost/vm

Scope and responsibility
- Minimal Ny-based executors and helpers for self‑hosting experiments.
- Responsibilities: trial executors (MIR JSON v0), tiny helpers (scan/binop/compare), smoke drivers.
- Forbidden: full parser implementation, heavy runtime logic, code generation.

Imports policy (SSOT)
- Dev/CI: file-using allowed; drivers may embed JSON for tiny smokes.
- Prod: prefer `nyash.toml` mapping under `[modules.selfhost.*]`.

Notes
- MirVmMin covers: const/binop/compare/ret (M2). Branch/jump/phi are later.
- Keep changes minimal and spec‑neutral; new behavior is gated by new tests.
