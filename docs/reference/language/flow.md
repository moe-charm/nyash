# Flow (Stateless Namespace)

Status
- Default ON (can be disabled with NYASH_ENABLE_FLOW=0). Backends behavior remains unchanged; this only introduces a clearer surface for stateless modules.
- Purpose: replace legacy "static box" for stateless utilities and entry modules with a clearer, safer construct.

Definition
- A flow is a stateless container of methods.
- It does not allow persistent fields or instance lifecycle.

Rules
- Allowed:
  - Method declarations
  - Local variables inside methods (temporary computation)
- Forbidden:
  - Field declarations (persistent state)
  - `birth`/`fini` members
  - `new Flow()` instantiation
  - `me` receiver inside flow methods

Lowering (implementation intent)
- A call `Flow.method(a, b)` lowers to the global function form `Flow.method/2`.
- No BoxCall is generated; runtime dispatch remains simple and consistent across backends.

Typical Usage
```
flow Main {
    main() {
        local cfg = load_config()
        local result = process(cfg)
        print(result)
    }
}

flow MathUtils {
    add(a, b) {
        local r = a + b
        return r
    }
}
```

Errors (Fail‑Fast)
- Declaring a field inside a flow → parse/semantic error: "flow cannot declare fields".
- Using `new Flow()` → invalid construct: "flow cannot be instantiated".
- Referencing `me` inside a flow → invalid receiver: "flow methods have no receiver".

Migration (from legacy static box)
- Replace `static box Name { ... }` with `flow Name { ... }`.
- Move any persistent state to a proper instance/service box, or thread it through parameters.
- Calls remain source-compatible (`Name.method(...)`), lowering becomes explicit global function resolution.

Notes
- Hakorune recommends `flow Main` as the standard entry style. Entry policy is Strict（既定は `Main.main` のみをエントリとみなす）— details: `docs/reference/language/entrypoints.md`。
- Keep flows small and stateless. For stateful/services or plugin-backed providers, use dedicated instance/service boxes.
- Flows are a good fit for entry points, pure helpers, and utility modules.

Env toggle
- Parser acceptance is ON by default. Disable with `NYASH_ENABLE_FLOW=0` when needed.
