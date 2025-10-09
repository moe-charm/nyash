# ADR: Collections API Unification — Phase 1 (Gated rollout)

Status: accepted (phase‑1 in progress)
Date: 2025‑10‑09

Context
- Current `MapBox.get()` returns `StringBox("Key not found: …")` on missing keys, which forces string‑error checks and leaks into control‑flow.
- `ArrayBox` exposes `.length()` (alias: `.size()` via interpreter), and `StringBox` lacked a first‑class `.size()`/`.isEmpty()` pair in core paths.
- We want a unified, minimal collection surface: `.size()`, `.isEmpty()`, `get(...) -> Box|null` where applicable, without breaking existing code.

Decision
- Add a migration flag `HAKO_MAP_GET_NULL=1` (alias: `NYASH_MAP_GET_NULL=1`) that changes `MapBox.get(missing)` to return `null` (NullBox). Default OFF for compatibility.
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
- New env: `HAKO_MAP_GET_NULL` documented in `docs/config/env.md`.
- Minimal code change size; no public spec change by default.
- Follow‑up phases can deprecate status‑string returns from mutators (`set/clear`) and promote lints.

Migration Notes
- Short‑term: prefer `map.has(k)` before `map.get(k)` in compatibility mode.
- When enabling `HAKO_MAP_GET_NULL=1`, remove string‑error checks (e.g., `RegexFlow.find_from(value, "Key not found:", …)`).
- Replace `arr.length()` → `arr.size()` gradually; `.length()` remains as an alias for now.

Verification
- Add a smoke test that asserts `MapBox.get(missing) == null` when the flag is ON.

