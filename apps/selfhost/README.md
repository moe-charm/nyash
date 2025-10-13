# Self‑Hosting Apps (Ny‑only)

Purpose
- Keep self‑hosting Ny scripts isolated from general `apps/` noise.
- Enable fast inner loop without touching Rust unless necessary.

Conventions
- Entry scripts live under this folder; prefer minimal dependencies.
- Use `--using-path selfhost:apps` when resolving modules.
- Prefer VM (`--backend vm`) for speed and stability.

Quickstart
- Run minimal sample: `make dev` (uses `apps/selfhost-minimal/main.nyash`)
- Watch changes: `make dev-watch`
- Run parser Ny project: `./tools/dev_selfhost_loop.sh --std -v --backend vm -- apps/selfhost/ny-parser-nyash/main.nyash`

Guidelines
- Keep files small and composable; avoid cross‑project coupling.
- If moving an existing `apps/*` item here, update docs/scripts accordingly.
- For namespace usage, pass `--using-path selfhost:apps`.
