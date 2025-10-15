# Rune Host (Skeleton)

Status: Minimal bridge (default OFF). We stop here for now. The box is in place and a VM extern is ready, but the box currently ships with a tiny built‑in fallback so tests stay green without wiring providers. No runner/VM‑core changes beyond the extern adapter.

## Responsibility
- Provide a single entry point to evaluate a small "rune" program with context.
- Be disabled-by-default (fail-fast) until a provider is wired.
- Encapsulate provider selection and timeouts behind a box boundary.

## Interface (Box-first)
File: selfhost/vm/boxes/rune_host.hako

- RuneHostBox.is_available() -> Bool
- RuneHostBox.provider_name() -> String
- RuneHostBox.eval(code: String, ctx: MapBox) -> Int|String|Json|Null
  - Current minimal: contains a tiny fallback ("1+2" -> 3, integer literal passthrough). No side‑effects.
  - Extern route prepared (nyrt.rune.eval), but left unused by default to avoid coupling during bring‑up.

## Environment (planned)
- HAKO_RUNE_ENABLE=0|1 (default 0)
- HAKO_RUNE_PROVIDER=mock|wasm|… (default mock)
- HAKO_RUNE_TIMEOUT_MS=2000

Notes
- Box does not yet read env; extern adapter reads HAKO_RUNE_ENABLE/HAKO_RUNE_PROVIDER when we switch the box to extern.
- Keep scope small and box‑local.

## Usage (skeleton)
```
using "selfhost/vm/boxes/rune_host.hako" as RuneHostBox

static box Main {
  main() {
    local ctx = new MapBox()
    local rc = RuneHostBox.eval("1+2", ctx)
    print("result:" + (""+rc))
    return 0
  }
}
```

## Policy
- Box boundary only; no runner/VM-core changes.
- Default OFF; fail-fast on use.
- One smoke (disabled) is provided; providers and enabled path are deferred.

## Next Steps (when unfreezing)
1) Add env reading inside RuneHostBox and a minimal mock provider.
2) Optional wasm provider (sandboxed), behind HAKO_RUNE_PROVIDER=wasm.
3) Add timeouts and error classification; expand return type if needed.


### Decision (Phase pause)
- Stop at Minimal Bridge: box facade + extern adapter in Rust, no provider wiring yet.
- Rationale: keep selfhost stabilization priority; avoid widening core surface area.
- Enablers already in place: env mapping in extern adapter, module key reserved in hako.toml.
