Box Factory — Priority & Policy (Phase 15.5 → 15.7)

Purpose
- Unify creation of Boxes from three sources: Builtin, Plugin, User.

Priority Policy
- Default (dev): StrictPluginFirst → Plugin > User > Builtin
- Compatible: CompatPluginFirst → Plugin > Builtin > User
- Legacy: BuiltinFirst → Builtin > User > Plugin (avoid; migration only)

Control
- Env: `NYASH_BOX_FACTORY_POLICY={strict_plugin_first|compat_plugin_first|builtin_first}`
- Plugin overrides for reserved core types: set both
  - `NYASH_USE_PLUGIN_BUILTINS=1`
  - `NYASH_PLUGIN_OVERRIDE_TYPES="StringBox,ArrayBox,MapBox"`

Reserved Core Types
- StringBox, IntegerBox, BoolBox, FloatBox, NullBox, ArrayBox, MapBox, ResultBox, MethodBox
- When `NYASH_USE_PLUGIN_BUILTINS=1` and type is listed in `NYASH_PLUGIN_OVERRIDE_TYPES`, plugins may provide these types.

Status
- Builtin implementations for String/Array/Map removed; use plugins (`nyash-*-plugin`).
- VM convenience handlers removed; BoxCall routes via User/Plugin path.

