Plugins — Developer Quick Guide (Hako ABI)

Build
- Prefer building all plugins via tester:
  - `tools/plugin-tester/target/release/plugin-tester build-all`

Smoke Integration
- Dynamic detection checks artifacts under `plugins/*/libnyash_*.{so,a}`.
- Smokes auto-builds missing plugins when `SMOKES_AUTO_BUILD_PLUGINS=1`.
- `plugin-on` profile enables plugin priority and disables builtin core boxes.

Priority
- `NYASH_USE_PLUGIN_BUILTINS=1` and `NYASH_PLUGIN_OVERRIDE_TYPES` allow plugins to own core types (String/Array/Map).

ABI Naming
- “Hako ABI” is the preferred name for the TypeBox interface (formerly “Nyash ABI”). The binary symbols may still use `nyash_` prefixes for compatibility.

Troubleshooting
- If you see "Missing dynamic plugins: …", run the tester or enable auto-build:
  - `SMOKES_AUTO_BUILD_PLUGINS=1 tools/smokes/v2/run.sh --profile quick-selfhost`
