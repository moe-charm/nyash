# Phase 15 — Box Stacking Roadmap (Living)

This roadmap is a living checklist to advance Phase 15 with small, safe boxes. Update continuously as we progress.

## Now (ready/green)

- [x] v0 Ny parser (Ny→JSON IR v0) with wrappers (Unix/Windows)
- [x] Runner JSON v0 bridge (`--ny-parser-pipe`) → MIR → MIR-Interp
- [x] E2E + roundtrip practical recipes (Windows/Unix)
- [x] Docs path unify (phase-15 under roadmap tree)
- [x] Direct bridge (design + skeleton; feature-gated)
- [x] AOT P2 stubs (CraneliftAotBox/LinkerBox) + RUN smoke wiring
- [x] JIT‑only baseline stabilized (core smokes green; plugins optional)
- [x] Roundtrip (Case A/B) aligned; Case A re‑enabled via parser pipe
- [x] using/namespace (gated) + nyash.link minimal resolver
- [x] NyModules + ny_plugins regression suite (Windows path normalization/namespace derivation)
- [x] Standard Ny scripts scaffolds added (string/array/map P0) + examples + jit_smoke

## Next (small boxes)

1) Standard Ny std impl (P0→実体化)
   - Implement P0 methods for string/array/map in Nyash (keep NyRT primitives minimal)
   - Enable via `nyash.toml` `[ny_plugins]` (opt‑in); extend `tools/jit_smoke.sh`
2) Ny compiler MVP (Ny→MIR on JIT path)
   - Ny tokenizer + recursive‑descent parser (current subset) in Ny; drive existing MIR builder
   - Flag path: `NYASH_USE_NY_COMPILER=1` to switch rust→ny compiler; rust parser as fallback
   - Add apps/selfhost-compiler/ and minimal smokes
3) Bootstrap loop (c0→c1→c1’)
   - Use existing trace/hash harness to compare parity; add optional CI gate
4) Plugins CI split (継続)
   - Core always‑on (JIT, plugins disabled); Plugins as optional job (strict off by default)
5) LLVM Native EXE Generation
   - LLVM backend object → executable pipeline completion
   - Separate `nyash-llvm-compiler` crate (reduce main build weight)
   - Input: MIR (JSON/binary) → Output: native executable
   - Link with nyrt runtime (static/dynamic options)
   - Integration: `nyash --backend llvm --emit exe program.nyash -o program.exe`

## Later (incremental)

- v1 Ny parser (let/if/call) behind `NYASH_JSON_IR_VERSION=1`
- JSON v1 bridge → MirBuilder (back-compat v0)
- 12.7 sugars normalized patterns in bridge (?. / ?? / range)
- E2E CI-lite matrix (no LLVM) for v0/v1/bridge roundtrip
- Ny script plugin examples under `apps/plugins-scripts/`
- Expand std Ny impl (String P1: trim/split/startsWith/endsWith; Array P1: map/each/filter; Map P1: values/entries/forEach)
- using/nyash.link E2E samples under `apps/` (small project template)
- Tighten Plugins job: migrate samples to Core‑13; re‑enable strict diagnostics

## Operational switches

- Parser path: `--parser {rust|ny}` or `NYASH_USE_NY_PARSER=1`
- JSON dump: `NYASH_DUMP_JSON_IR=1`
- Load Ny plugins: `NYASH_LOAD_NY_PLUGINS=1` / `--load-ny-plugins`
- AOT smoke: `CLIF_SMOKE_RUN=1`

## Recipes / Smokes

- JSON v0 bridge: `tools/ny_parser_bridge_smoke.sh` / `tools/ny_parser_bridge_smoke.ps1`
- E2E roundtrip: `tools/ny_roundtrip_smoke.sh` / `tools/ny_roundtrip_smoke.ps1`

## Stop criteria (Phase 15)

- v0 E2E green (parser pipe + direct bridge) including Ny compiler MVP switch
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
