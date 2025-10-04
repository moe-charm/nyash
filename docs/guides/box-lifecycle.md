# Box Lifecycle — auto‑birth Policy (2025‑10)

Purpose: remove ambiguity around object initialization. “new means usable.”

Policy (Strict, Uniform)
- Default is auto‑birth: `new TypeBox(args)` automatically invokes `birth(args)` right after allocation. Applies to ALL boxes, including plugins.
- Advanced path with `unborn()`: `TypeBox.unborn().withXxx(...).birth(...)` suppresses auto‑birth until `birth` is explicitly called.
- `birth()` is idempotent and returns Result:
  - Calling `birth()` twice is a no‑op (success).
  - On failure, the `new` expression fails (no silent fallback).
- Verifier/Lint:
  - Any operation on an unborn instance (set/get/call) is an immediate error.

Implementation Roadmap (small, reversible steps)
1) Lowering: Builder emits `birth` immediately after `NewBox` (except `unborn()` path).
2) Plugin Loader: synthesize a no‑op `birth` for legacy plugins lacking it (warn for one release, then silent).
3) Verifier: enforce unborn-forbid rules (dev: detailed diagnostics; prod: concise).
4) Docs: keep this guide as the single source of truth, link from README.

Parser / Surface Syntax
- Dot-call `birth()` is accepted on instances for the unborn path:
  - Example: `local alice = Life.unborn(); alice.birth(); alice.name()`
- Auto‑birth path remains sugar: `local alice = new Life("Alice")`.

Fail‑Fast Error (stable text)
- Operations on unborn instances fail with the message containing:
  - `operation on unborn instance (call birth() first)`
- Tests grep this substring; keep it stable across versions.

Migration Notes
- Existing code like `new MapBox(); regs.birth()` continues to work (redundant second `birth` is a no‑op).
- Prefer the simple form: `local regs = new MapBox()`.
- For pre‑configuration needs, switch to the explicit path:

```nyash
local regs = MapBox.unborn()
    .withPolicy(:deterministic, :symbolKeys, :ordered)
    .birth()?  # Result-returning birth
```

Plugin Requirements
- All plugin boxes must expose a `birth()` method. When absent, the loader provides a no‑op stub during migration.
- `birth` should be safe for repeated invocation (idempotent).

Testing Notes
- Smokes validate auto‑birth/unborn and plugin no‑op birth:
  - `tools/smokes/v2/profiles/quick/core/userbox_unborn_failfast_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/userbox_unborn_then_birth_ok_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/plugin_no_birth_noop_vm.sh` (dev fixture; skips if missing)

## Auto‑Birth と unborn
- 既定は auto‑birth（ が自動で  を呼ぶ）。
- 上級者向けに  を許可。
-  は冪等（多重呼び出しは no‑op）。

## Auto‑Birth と unborn
- 既定は auto‑birth（`new` が自動で `birth` を呼ぶ）。
- 上級者向けに `TypeBox.unborn().withXxx(...).birth()` を許可。
- `birth()` は冪等（多重呼び出しは no‑op）。
