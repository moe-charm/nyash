# Static Plugins — Spec Ingestion & Registration

Purpose
- Make type_id and method slots available even without full `hako.toml`.
- Allow per‑Box invoke pointer registration when statically linked (features).

Mechanism
- Each plugin contains `hako_box.toml` with:
  - `[box]` header: `type_name`, `type_id`, `provider`
  - `[[methods]]` list: `name`, `slot`, `arity`
- At runtime startup, `src/runtime/static_plugins/mod.rs`:
  - `include_str!()` to embed the toml text
  - Register metadata into Loader (box_specs)
  - When features are enabled, register per‑Box `invoke_static` symbol

Notes
- Dynamic runs still prefer dlopen symbols; static specs serve as early metadata and fallback.
- Duplicate registration is skipped (first wins).

Profiles
- quick: dynamic plugins + static specs (metadata only)
- plugins: dynamic plugins (tests always on)
- AOT: static kernel + optional static plugins (invoke pointers exported)
