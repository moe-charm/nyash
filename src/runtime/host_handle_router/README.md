HostHandleRouterBox

Responsibility
- Centralize HostHandle slot dispatch to avoid spreading slot numbers and decode/encode rules.
- Allow host_api.rs to remain a thin C-ABI boundary.

Inputs/Outputs
- Input: (handle, selector_id, TLV args)
- Output: TLV result or error code

Guards
- No direct imports of builtin Array/Map implementations; use HostHandle indirection.
