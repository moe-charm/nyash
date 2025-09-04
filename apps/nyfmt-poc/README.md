# nyfmt PoC Examples (Documentation Only)

This directory hosts small snippets demonstrating reversible formatting goals. No runtime behavior changes or formatter are included in‑tree yet.

Examples to explore:
- pipeline-compact.nyash: pipeline style vs canonical call nesting
- safe-access-default.nyash: `?.` and `??` sugar vs explicit conditionals
 - coalesce-range-roundtrip.nyash: `??` and `a..b` triad (Before/Canonical/Round‑Trip)
 - compound-assign-roundtrip.nyash: `+=` triad (Before/Canonical/Round‑Trip)

Enable PoC smoke hints:
```bash
NYFMT_POC=1 ./tools/nyfmt_smoke.sh
```

Notes: The real formatter prototype lives out of tree during early PoC. This folder documents intent and testable round‑trip expectations.
