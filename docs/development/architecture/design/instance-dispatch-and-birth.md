# Instance Dispatch & Birth Invariants (Phase 15)

Status: Adopt (Go). Scope: VM/Builder/Smokes. Date: 2025-09-27.

## Goals
- Unify user-instance method calls to functions for stability and debuggability.
- Make constructor flow explicit: NewBox(Box) → Global("Box.birth/N").
- Eliminate BoxCall-on-Void crashes (stringify, accessors) without changing prod semantics.

## Decisions
1) Instance→Function rewrite (default ON)
- Builder always lowers `me.m(a,b)` to `Box.m/2(me,a,b)`.
- Env override: `NYASH_BUILDER_REWRITE_INSTANCE=0|1` (default 1).

2) VM BoxCall policy
- User-defined InstanceBox: BoxCall is disallowed in prod; dev may fallback with one-line WARN.
- Plugin/Builtin boxes: BoxCall allowed as before (ABI contract).

3) NewBox→birth invariant
- Builder: After `NewBox(Box)`, emit `Global("Box.birth/N")` when present (arity excludes `me`).
- VM: No implicit birth; run what Builder emits.
- Dev assert: `birth(me==Void)` forbidden; WARN+metric when hit.

4) Void stringify safety valve (dev)
- VM: `stringify(Void)` yields `"null"` (to match `toString(Void)`); one-line WARN+metric.
- Remove once hits converge to zero.

## Smokes & Probes
- Heavy JSON smokes use a probe that prints `ok`. Runner compares the last non-empty line exactly to `ok` (trimmed). Noise-safe, portable.

## Acceptance
- Quick: JSON apps green; user-instance BoxCall hits=0; stringify-void hits=0.
- Heavy: nested/roundtrip PASS where parser is available.

