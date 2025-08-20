# Examples: Plugin BoxRef Return (v2.2)

- File: `plugin_boxref_return.nyash`
- Purpose: Demonstrates a plugin method returning a Box (BoxRef/Handle), and passing Box as an argument.

How to run (after full build):
- Ensure `nyash.toml` includes FileBox with methods:
  - `copyFrom = { method_id = 7, args = [ { kind = "box", category = "plugin" } ] }`
  - `cloneSelf = { method_id = 8 }`
- Build the plugin: `cd plugins/nyash-filebox-plugin && cargo build --release`
- Run the example: `./target/release/nyash docs/examples/plugin_boxref_return.nyash`

Expected behavior:
- Creates two FileBox instances (`f`, `g`), writes to `f`, copies content to `g` via `copyFrom`, then closes both.
