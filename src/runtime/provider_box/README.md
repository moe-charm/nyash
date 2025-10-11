ProviderBox — Thin boundary for plugin/config/embedded resolution

Responsibility
- Ensure plugin host is initialized and attempt creation through Plugin → Registry → Embedded in that order.
- Keep initialization logic and fallbacks out of VM handlers.

Inputs/Outputs
- Input: box_type, args[]
- Output: Box or RuntimeError

Guards
- Deterministic mode blocks IO/NET boxes.
- plugin-only boxes do not fall back to embedded.

ENV behavior
- Respects plugin-on gates. With `HAKO_PLUGIN_POLICY=auto` (and `NYASH_USE_PLUGIN_BUILTINS=1` for core collections), String/Array/Map are served by dynamic plugins; otherwise builtin fallbacks apply. Public APIs remain unchanged across VM/LLVM.
