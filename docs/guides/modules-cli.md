# Modules CLI — Dir-as-NS Visibility (Dev Aid)

This page documents the lightweight CLI utilities for inspecting module resolution when using Dir-as-NS (directory → namespace) discovery.

## Commands

- `hakorune --list-modules`
  - Lists discovered modules in precedence order: `[workspace]` → `[override]` → `[auto]` (Dir-as-NS).
  - Duplicate namespaces are suppressed by precedence. Workspace entries show their origin tag: `hako_module`, `module`, or `module_hako`.

- `hakorune --modules-show <namespace>`
  - Prints a single mapping line for the specified namespace with its origin.
  - Example: `[auto] selfhost.compiler.pipeline_v2.using_resolver → ./apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako`

- `hakorune --modules-resolve <file>`
  - Computes the Dir-as-NS namespace for a file under `apps/` and prints it.
  - Errors if the file is not located beneath the `apps/` root.

## Dir-as-NS normalization rules (v2.2)

- Directory component: replace `-` with `.` (e.g., `selfhost-compiler` → `selfhost.compiler`).
- File name: strip `.hako`, then strip optional `_box` suffix (e.g., `json_minify_box.hako` → `json_minify`).
- Join with `.`. Example:
  - `apps/selfhost-compiler/pipeline_v2/json_minify_box.hako` → `selfhost.compiler.pipeline_v2.json_minify`

## Recommended workflow

- Development
  - Enable auto discovery: `NYASH_USING_DIR_NS=1`
  - Optionally auto-register boxes in VM: `NYASH_VM_AUTO_REGISTER_DIR_NS=1`
  - Inspect with `--list-modules`, drill down with `--modules-show` / `--modules-resolve`.

- Production/CI
  - Keep `NYASH_USING_DIR_NS=0`.
  - Declare modules under `hako.toml` `[modules]` (workspace/overrides/aliases).
  - `--list-modules` remains useful to validate configuration, but should not rely on `[auto]` entries.

## Notes

- `--list-modules` anchors the root at `NYASH_ROOT` (or CWD) and reads `hako.toml`/`nyash.toml` when present.
- Conflict detection is performed for workspace manifests; conflicts are printed as diagnostics.
- All utilities are side-effect-free and return after printing (no execution of program files).
