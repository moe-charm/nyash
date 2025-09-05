# Phase 15 — Box Stacking Roadmap (Living)

This roadmap is a living checklist to advance Phase 15 with small, safe boxes. Update continuously as we progress.

## Now (ready/green)

- [x] v0 Ny parser (Ny→JSON IR v0) with wrappers (Unix/Windows)
- [x] Runner JSON v0 bridge (`--ny-parser-pipe`) → MIR → MIR-Interp
- [x] E2E + roundtrip practical recipes (Windows/Unix)
- [x] Docs path unify (phase-15 under roadmap tree)
- [x] Direct bridge (design + skeleton; feature-gated)
- [x] AOT P2 stubs (CraneliftAotBox/LinkerBox) + RUN smoke wiring

## Next (small boxes)

1) Ny config loader (Ny-only)
   - FileBox + TOMLBox to read `nyash.toml` → Map (env/tasks/plugins/box_types)
   - Deliver as `apps/std/ny-config.nyash` with simple API: `read_root()`, `load_toml()`, `get_env()/get_tasks()`
2) Ny script plugins enumeration
   - Add `[ny_plugins]` to `nyash.toml` for pure Nyash plugins
   - Runner opt-in hook: `NYASH_LOAD_NY_PLUGINS=1`/`--load-ny-plugins` to include/register in order
3) Direct bridge (v0) rollout
   - `--parser ny`/`NYASH_USE_NY_PARSER=1` routes to in-proc bridge (subset: return/int/+ - * /, parens)
   - Keep JSON as debug dump (`NYASH_DUMP_JSON_IR=1`)
   - Expand smokes + parity checks with rust path
4) AOT P2 (stub→first run)
   - Emit `.obj/.o` → link → run; measure time/size; log instructions

## Later (incremental)

- v1 Ny parser (let/if/call) behind `NYASH_JSON_IR_VERSION=1`
- JSON v1 bridge → MirBuilder (back-compat v0)
- 12.7 sugars normalized patterns in bridge (?. / ?? / range)
- E2E CI-lite matrix (no LLVM) for v0/v1/bridge roundtrip
- Ny script plugin examples under `apps/plugins-scripts/`

## Operational switches

- Parser path: `--parser {rust|ny}` or `NYASH_USE_NY_PARSER=1`
- JSON dump: `NYASH_DUMP_JSON_IR=1`
- Load Ny plugins: `NYASH_LOAD_NY_PLUGINS=1` / `--load-ny-plugins`
- AOT smoke: `CLIF_SMOKE_RUN=1`

## Recipes / Smokes

- JSON v0 bridge: `tools/ny_parser_bridge_smoke.sh` / `tools/ny_parser_bridge_smoke.ps1`
- E2E roundtrip: `tools/ny_roundtrip_smoke.sh` / `tools/ny_roundtrip_smoke.ps1`

## Stop criteria (Phase 15)

- v0 E2E green (parser pipe + direct bridge)
- v1 minimal samples pass via JSON bridge
- AOT P2: emit→link→run stable for constant/arith
- Docs/recipes usable on Windows/Unix

## Notes

- JSON is a temporary, safe boundary. We will keep it for observability even after the in-proc bridge is default.
- Favor smallest viable steps; do not couple large refactors with new features.

## Ny Plugins → Namespace (Plan)

- Phase A (minimal): Add a shared `NyModules` registry (env.modules.{set,get}).
  - Map file path → namespace (project‑relative, separators → `.`, trim extension).
  - R5 hook: if a Ny plugin returns an exports map/static box, register it under the derived namespace.
  - Guard: reject reserved prefixes (e.g., `nyashstd.*`, `system.*`).
- Phase B (scope): Optionally run `[ny_plugins]` in a shared Interpreter to share static definitions.
  - Flag: `NYASH_NY_PLUGINS_SHARED=0` to keep isolated execution.
  - Logs: `[ny_plugins] <ns>: REGISTERED | FAIL(reason)`.
- Phase C (language bridge): Resolve `using foo.bar` via `NyModules`, then fallback to file/package resolver (nyash.link).
  - Keep IDE‑friendly fully qualified access; integrate with future `nyash_modules/`.
