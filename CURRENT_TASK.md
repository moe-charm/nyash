# Current Task — Phase 15 (Concise)

Focus
- JSON heavy smokes green (VM), stable method resolution.
- Instance→Function rewrite default ON (prod/dev/ci).
- NewBox→birth invariant enforced; eliminate BoxCall-on-Void crashes.

Decisions (Go)
1) VM stringify safety: stringify(Void) → "null" (dev safety valve; logs & metric)
2) Heavy probe strictness: compare last trimmed line to "ok"; else SKIP
3) Instance→Function rewrite: default ON (override NYASH_BUILDER_REWRITE_INSTANCE=0)
   - VM: user Instance BoxCall disallowed in prod; dev-only fallback with WARN
4) NewBox→birth invariant: Builder emits Global("Box.birth/N"); VM has no implicit birth
   - Dev assert: birth(me==Void) forbidden (WARN+metric)

Plan (next patches)
- Implement stringify(Void) guard in VM (handlers/boxes.rs)
- Tighten probes in quick/core json_* smokes (tail-trim-compare)
- Set rewrite default ON in Builder (method_call_handlers.rs)
- Add VM guard for user Instance BoxCall (prod error; dev fallback)
- (Optional) Builder verify for NewBox→birth, VM dev assert hook

Status
- Tokenizer/parse([]): PASS
- Nested/Roundtrip: probe SKIP on this env (expected); direct run OK
- json_query_min (core): still null → fix follows via stringify(Void) + invariant

Acceptance
- quick: json_pp/json_lint/json_query_min PASS; user Instance BoxCall hits=0
- heavy: nested/roundtrip PASS where parser available

References
- docs/design/instance-dispatch-and-birth.md
- tools/smokes/README.md (heavy probes)
