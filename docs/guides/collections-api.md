# Collections API — Unified Semantics (Phase 15.7+)

This guide documents the unified API across StringBox, ArrayBox, and MapBox, and clarifies the default fallback for Map.keys()/values().

## Unified methods
- `.size()` → IntegerBox
- `.isEmpty()` → BoolBox
- `.toJSON()` → StringBox (impl dependent)

### StringBox
- `.size()`
- `.substring(start, end)`
- `.charAt(index)`
- `.indexOf(needle)`

### ArrayBox
- `.get(index)` → Box | null
- `.set(index, value)` → null
- `.push(value)` → null
- `.size()` / `.isEmpty()`

### MapBox
- `.get(key)` → Box | null (missing returns null)
- `.set(key, value)` → null
- `.delete(key)` → null
- `.has(key)` → BoolBox
- `.keys()` → ArrayBox (default: see fallback below)
- `.values()` → ArrayBox (default: see fallback below)
- `.call(key, args:ArrayBox=[])` → Box | null (sugar for calling stored Callable)

## Map.keys()/values() — Default fallback (standardized)

By default, the VM normalizes `Map.keys()` and `Map.values()` to return ArrayBox even when the underlying plugin exports string forms (`keysS()/valuesS()`).

Behavior:
- If the plugin provides `keys()`/`values()` returning ArrayBox, that result is used as-is.
- Otherwise, the VM calls `keysS()`/`valuesS()` and splits by `\n` to produce ArrayBox. This fallback is always ON in Phase 15.7+.

Notes:
- Plugins may later implement Array-returning `keys()/values()` directly. The VM fallback is designed to be removed after parity is confirmed (no behavior change expected for user code).
- A temporary environment `HAKO_MAP_KEYS_VALUES_FALLBACK` existed during migration; it is no longer required and has no effect when unset. Profiles may still set it for explicitness, but the fallback is now default.

## Map.call (P1)

`Map.call(key, args:ArrayBox=[])` resolves the value at `key` and, if it is callable, invokes it synchronously. Missing keys return null; non-callables raise a type error. `Map.callAsync(...)` is planned in later phases.

## Migration summary
- `.size()` unified across all boxes (`.length()` deprecated; remains as an alias where applicable).
- `Map.get(missing)` returns null.
- Mutators `set()/clear()/push()/reverse()/sort()` return null.
- `keys()/values()` normalize to ArrayBox via default fallback when necessary.
