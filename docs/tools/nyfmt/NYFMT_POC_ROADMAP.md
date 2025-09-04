# nyfmt Reversible Formatter – PoC Roadmap

Status: Proposal (docs only)

Goal: A reversible code formatter for Nyash that enables round‑trip transforms (format → parse → print → original) for ANCP/Phase 12.7 sugar while preserving developer intent.

## PoC Scope (Phase 1)
- Define reversible AST surface subset (no semantics changes).
- Implement pretty‑printer prototype in Rust or script (out of tree), constrained to subset.
- Add examples demonstrating round‑trip invariants and failure modes.

Round‑trip invariants (subset)
- Pipeline: `lhs |> f(a)` ⇄ `f(lhs,a)`
- Safe Access: `a?.b` ⇄ `peek a { null => null, else => a.b }`
- Default: `x ?? y` ⇄ `peek x { null => y, else => x }`
- Range: `a .. b` ⇄ `Range(a,b)`
- Compound Assign: `x += y` ⇄ `x = x + y` (var/field target)

## VSCode Extension Idea
- Commands:
  - "Nyfmt: Format (reversible subset)"
  - "Nyfmt: Verify Round‑Trip"
- On‑save optional gate with env flag `NYFMT_POC=1`.
- Diagnostics panel lists non‑reversible constructs.

## Examples and Smokes
- Place minimal examples under `apps/nyfmt-poc/`.
- Add a smoke script `tools/nyfmt_smoke.sh` that:
  - echoes `NYFMT_POC` and current subset level
  - prints instructions and links to `ANCP-Reversible-Mapping-v1.md`
  - shows Before/Canonical/Round‑Trip triads from examples

## Non‑Goals
- Changing Nyash runtime/semantics.
- Enforcing formatting in CI (PoC is opt‑in).
