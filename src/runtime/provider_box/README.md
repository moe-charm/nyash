ProviderBox — Responsibility & Interfaces

Scope
- Single boundary to ensure plugin host init and to create boxes in the order:
  PluginHost → v2 BoxFactoryRegistry → Embedded fallback.

Responsibilities
- Initialization: load config from env path or common defaults (nyash.toml, hako.toml).
- Provider application: apply plugin providers to the v2 registry so `new Box()` resolves.
- Creation: attempt to create boxes via plugin host first, then registry, then unified fallback.

Non‑Responsibilities
- No TLV encoding/decoding (use TlvCodecBox).
- No direct loader calls from VM call‑sites (use PluginHostBox facade for invokes).
- No policy decisions about method resolution (use MethodRegistryBox).

Environment
- HAKO_PLUGIN_POLICY=auto|force  Enable plugin path.
- NYASH_PLUGIN_CONFIG=...        Config file (default nyash.toml → hako.toml → hakorune.toml).
- NYASH_PLUGIN_LOOKUP_LOCAL=1    Probe near‑lib nyash_box.toml/hako_box.toml for specs.

Notes
- Eager‑load core boxes (Array/Map/String) when config is present to register per‑Box invoke.
- ProviderBox is intentionally thin and side‑effect free beyond initialization and provider apply.

