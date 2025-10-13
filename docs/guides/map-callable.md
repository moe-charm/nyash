# Map.call / Callable — P1 Spec (Draft)

Purpose: Provide a small, safe sugar to call a callable value stored in MapBox.

## API (P1)
- `Map.call(key, args:ArrayBox=[]) -> Box | null`
  - Behavior:
    1) `v = map.get(key)`
    2) If `v == null`: return `null`
    3) If `v` is callable: perform a synchronous call with given args and return its result
    4) Otherwise: Fail-Fast with a TypeError("map.call: value is not callable")
- `Map.callAsync(key, args:ArrayBox=[])` (P2+)
  - Not implemented in P1. Returns InvalidMethod (or documented as future work)

## Callable definition
- Accept VM-known callables: FunctionBox/Closure/CallableBox/etc.
- P1 limits to synchronous path only (consistent across VM/LLVM).

## Errors / Nulls
- Missing key or value `null` -> return `null` (unified collection semantics)
- Non-callable value -> TypeError(Fail-Fast)
- Errors in callee propagate normally per backend rules.

## Implementation plan (phased)
1. P1 (VM sugar only): Implement in VM method routing — `Map.call` delegates to `get` then host `call`.
   - No plugin changes; plugins only provide `get/set/delete/keys/values`.
2. P2: Add `callAsync` and a Future path (optional) once scheduler hooks are ready.
3. P3: Optional plugin fast-path (if needed). Keep VM sugar as canonical entry.

## Tests (minimum)
- call success: set("f", f); call("f", [41]) -> 42
- missing key: call("missing", []) -> null
- non-callable: set("x", 123); call("x", []) -> TypeError

## Notes
- This spec follows unified null semantics in collections: `get` returns `null` for missing.
