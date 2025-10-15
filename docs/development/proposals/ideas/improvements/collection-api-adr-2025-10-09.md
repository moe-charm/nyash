# ADR: Collections API Unification — Phase 1 (Gated rollout)

Status: accepted (phase‑2 defaultized)
Date: 2025‑10‑09

Context
- Current `MapBox.get()` returns `StringBox("Key not found: …")` on missing keys, which forces string‑error checks and leaks into control‑flow.
- `ArrayBox` exposes `.length()` (alias: `.size()` via interpreter), and `StringBox` lacked a first‑class `.size()`/`.isEmpty()` pair in core paths.
- We want a unified, minimal collection surface: `.size()`, `.isEmpty()`, `get(...) -> Box|null` where applicable, without breaking existing code.

Decision
- Defaultize `MapBox.get(missing) -> null` (flag removed; behavior is now default).
- Add `.size()` and `.isEmpty()` convenience across core collections:
  - StringBox: `.size()` (alias to length), `.isEmpty()`
  - ArrayBox: `.isEmpty()` (interpreter already supports `.size()`)
  - MapBox: `.isEmpty()`
- Extend VM interpreter handlers to route these methods consistently.

Rationale
- Unifies ubiquitous checks: `if coll.isEmpty() { … }` and `n = coll.size()` across Array/Map/String.
- Removes string‑based error patterns from data‑path once the flag is enabled project‑wide.
- Gated rollout honors current freeze policy (no default semantics change) and allows per‑profile opt‑in.

Consequences
- Migration flag removed; docs updated in `docs/config/env.md`.
- Minimal code change size; public spec now aligns with intuitive Map semantics.
- Follow‑up phases can deprecate status‑string returns from mutators (`set/clear`) and promote lints.

Migration Notes
- Short‑term: prefer `map.has(k)` before `map.get(k)` in compatibility mode.
- Remove legacy string‑error checks (e.g., `RegexFlow.find_from(value, "Key not found:", …)`).
- Replace `arr.length()` → `arr.size()` gradually; `.length()` remains as an alias for now.

Verification
- Add a smoke test that asserts `MapBox.get(missing) == null` when the flag is ON.
