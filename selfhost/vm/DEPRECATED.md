# DEPRECATED: selfhost/vm (Mini‑VM sandbox)

This directory hosts the original Mini‑VM used for early self‑hosting.

Policy
- Status: Frozen (no new features).
- Purpose: Dev/education and targeted repros only.
- Successor: selfhost/hakorune-vm (Hakorune VM) — nyvm maps here by default.

Migration
- Prefer using aliases under `selfhost.hakorune-vm.*`.
- Mini‑VM specific smokes remain rc-only and opt-in.

Removal trigger
- When Hakorune VM reaches feature parity and quick/integration remain green for one sprint, Mini‑VM will be retired.
