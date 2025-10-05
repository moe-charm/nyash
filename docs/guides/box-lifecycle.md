# Box Lifecycle — auto‑birth (C++‑style) and unborn/in_birth (2025‑10)

Purpose: remove ambiguity around object initialization. “new means usable.”

Policy (Strict, Uniform)
- Default is auto‑birth: `new TypeBox(args)` automatically invokes `birth(args)` as part of construction.
- Advanced path with `unborn()`: `TypeBox.unborn().withXxx(...).birth(...)` suppresses auto‑birth until `birth` is explicitly called.
- `birth()` is idempotent and returns Result:
  - Calling `birth()` twice is a no‑op (success).
  - On failure, the construction fails (no silent fallback).
- Verifier/Lint:
  - Any operation on an unborn instance (set/get/call) is an immediate error.

MIR/VM Semantics (C++‑style constructor)
- MIR `NewBox` carries an optional `auto_birth: Option<String>` where the value is `"Class.birth/N"`.
- VM executes `NewBox` and, if `auto_birth` is present (or discoverable by global function table), calls `birth(me, args...)` immediately.
- Lifecycle states:
  - unborn → in_birth (during `birth`) → born (on success) / unborn (on failure)
  - During `in_birth`, methods on the same instance are allowed; re‑entrancy of `birth` is an error; second `birth` after success is a no‑op.

Parser / Surface Syntax
- Dot‑call `birth()` is accepted on instances for the unborn path:
  - Example: `local alice = Life.unborn(); alice.birth("Alice"); alice.name()`
- Auto‑birth sugar remains: `local alice = new Life("Alice")`.

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

Plugin Policy
- If a plugin defines `birth`, it is called as constructor (`method_id = 0` recommended).
- If `birth` is absent, construction proceeds without calling it (no‑op) to maintain compatibility.
- `birth` implementations should be idempotent.

Testing Notes
- Smokes validate auto‑birth/unborn and plugin no‑birth:
  - `tools/smokes/v2/profiles/quick/core/userbox_unborn_failfast_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/userbox_unborn_then_birth_ok_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/plugin_no_birth_noop_vm.sh` (dev fixture; skips if missing)

実装ポリシー（2025‑10 更新）
- Builder: `NewBox { auto_birth: Some("Class.birth/N") }` を既定で付与（関数が見つかる場合）。
- VM: `NewBox` 実行時に `auto_birth` を直ちに呼び出す。`in_birth` 状態を導入し、成功時のみ `born` を確定。
- Parser: `obj.birth(...)`（ドット呼び）を受理（unborn 経路のE2E）。
- Contracts: `NYASH_CHECK_CONTRACTS=1` 既定ON。unbornの操作は禁止。`birth()` は冪等。

注意（dev モード）
- bring‑up 便宜の挙動差が混じるため、core の値検証系スモークは dev=OFF（`SMOKES_USE_DEV=0`）を既定とする。
