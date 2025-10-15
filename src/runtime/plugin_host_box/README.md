PluginHostBox — Responsibility & Interfaces

Scope
- Facade for VM and runtime layers to interact with the plugin host without importing loader internals.

Responsibilities
- create_box: Instantiate plugin boxes via unified plugin host.
- invoke_instance_method: Call instance methods safely.
- resolve_method_handle: Resolve (type_id, method_id, returns_result) using MethodRegistryBox.

Non‑Responsibilities
- No TLV encoding/decoding (use TlvCodecBox).
- No method id tracing policy (MethodRegistryBox handles trace).
- No direct configuration loading (ProviderBox handles init/apply).

Environment
- Uses whatever ProviderBox initialized (nyash.toml / hako.toml). No direct env of its own.

