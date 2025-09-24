# Nyash Fixture Plugin

Minimal, deterministic plugin for smoke tests. Provides `FixtureBox` with:

- Methods
  - `birth` (id=0): creates an instance; returns instance id as raw u32 (4 bytes)
  - `echo` (id=1): returns the input string (TLV tag=6)
  - `get`  (id=2): returns constant string "ok"
  - `fini` (id=0xFFFF_FFFF): destroys the instance

- TypeBox FFI symbol: `nyash_typebox_FixtureBox`
- Legacy entry (compat): `nyash_plugin_invoke`
- Spec file: `nyash_box.toml` (type_id=101, method ids)

Build

```
cargo build --release -p nyash-fixture-plugin
```

Resulting artifacts (by platform):
- Linux: `target/release/libnyash_fixture_plugin.so`
- macOS: `target/release/libnyash_fixture_plugin.dylib`
- Windows: `target/release/nyash_fixture_plugin.dll`

Copy the built file to the project plugin folder (platform name preserved):
- Linux: `plugins/nyash-fixture-plugin/libnyash_fixture_plugin.so`
- macOS: `plugins/nyash-fixture-plugin/libnyash_fixture_plugin.dylib`
- Windows: `plugins/nyash-fixture-plugin/nyash_fixture_plugin.dll`

Use in smokes
- Profile: `tools/smokes/v2/run.sh --profile plugins`
- Test: Fixture autoload is auto-detected and run when the platform file is present
  - The smoke script auto-detects extension: `.so` (Linux), `.dylib` (macOS), `.dll` (Windows)

Notes
- On Windows, plugin filenames do not start with `lib`.
- The plugins smoke uses `using kind="dylib"` autoload; it is safe by default and only enabled when `NYASH_USING_DYLIB_AUTOLOAD=1` is set (the runner handles this).
