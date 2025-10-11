# hako.toml — Modules Spec (Proposed v2)

Goal
- Keep it simple, predictable, and overridable.
- Default: convention over configuration (auto‑discover modules under `apps/`).
- Override: `hako.toml` refines/aliases specific entries without fighting the default.

Design (Option C + Overrides)
- Auto‑discovery
  - Scan `apps/**/*.hako` and register namespaces by path:
    - `selfhost/vm/boxes/mir_vm_min.hako` → `selfhost.vm.boxes.mir_vm_min`
    - `apps/hakorune/vm/boxes/inst_scan.hako` → `hakorune.vm.boxes.inst_scan`
  - This becomes the implicit [modules] table unless overridden.

- hako.toml extensions
  - `[modules.aliases]`: short names that map to discovered namespaces.
    ```toml
    [modules.aliases]
    vm = "selfhost.vm"
    json = "selfhost.common.json_adapter"
    ```
  - `[modules.overrides]`: explicit path for specific keys (takes precedence).
    ```toml
    [modules.overrides]
    selfhost.vm.entry = "selfhost/vm/boxes/mini_vm_entry.hako"
    hakorune.vm.entry = "apps/hakorune/vm/boxes/hakorune_vm_entry.hako"
    ```
  - `[modules.options]`: discovery options（既定ON、除外強化）
    ```toml
    [modules.options]
    enable_discovery = true
    roots = ["apps"]
    # ヒューリスティック除外（glob無しの簡易実装）
    exclude = [
      "**/archive/**",    # アーカイブ除外
      "**/_*/*",          # _で始まるディレクトリ配下
      "**/test_*",        # テストファイル除外
      "**/example_*"      # サンプル除外
    ]
    ```

Resolution order
1. Start from auto‑discovered namespaces when `enable_discovery = true`.
2. Apply `[modules.overrides]` to replace specific keys → path.
3. Apply `[modules.aliases]` to inject additional keys that redirect to other keys/namespaces.

Workspace (module manifests)
- Preferred file name: `hako_module.toml` (legacy `module.toml` accepted for compatibility)
- Resolver wiring: also accepts `module.hako` as a fallback manifest when TOML is absent (same priority as shown by `--list-modules`).
- Declare per‑module boundary and public exports only.
  ```toml
  # selfhost/vm/hako_module.toml
  [module]
  name = "selfhost.vm"
  version = "1.0.0"

  [exports]
  entry = "boxes/mini_vm_entry.hako"
  mir_min = "boxes/mir_vm_min.hako"
  ```
  hako.toml
  ```toml
  [modules.workspace]
  members = [
    "selfhost/vm/hako_module.toml",
    "apps/hakorune/vm/hako_module.toml",
  ]
  ```

Examples
- Minimal (no hako.toml entries required):
  - `using selfhost.vm.boxes.mir_vm_min as MirVmMin`
  - `using hakorune.vm.boxes.inst_scan as InstScanBox`

- Friendly aliases (opt‑in):
  ```toml
  [modules.aliases]
  selfhost.vm.entry = "selfhost.vm.boxes.mini_vm_entry"
  hakorune.vm.entry = "hakorune.vm.boxes.hakorune_vm_entry"
  ```

Migration plan
- Phase 1 (additive):
  - Implement discovery behind `enable_discovery` (default true).
  - Keep existing explicit `[modules]` entries; treat as overrides.
  - Add tooling（dry‑run）: `--list-modules` で検出/override/alias を表示（移行の可視化）
- Phase 2 (documentation):
  - Recommend path‑based namespaces for new code.
  - Move repetitive entries to aliases.
- Phase 3 (cleanup):
  - Reduce hand‑written `[modules]` to only overrides/aliases.

Notes
- This mirrors Go’s path as namespace and Rust’s Cargo override patterns.
- It reduces boilerplate while keeping escape hatches for edge cases.

Labels in list-modules
- `[workspace:hako_module]` — exported via `hako_module.toml`
- `[workspace:module]` — legacy `module.toml`
- `[workspace:module_hako]` — parsed from `module.hako` (preview‑only)
- `[override]` — hako.toml overrides
- `[auto]` — auto discovery under roots (e.g., `apps/`)

Dry‑run (preview)
```bash
hako --list-modules
# [workspace:hako_module] selfhost.vm.entry → selfhost/vm/boxes/mini_vm_entry.hako
# [workspace:module_hako] demo.modhako.hello → apps/examples/module_hako_demo/hello_box.hako
# [override] selfhost.tools.dep_tree_core → apps/selfhost/tools/dep_tree_core.hako
# [auto] selfhost.vm.boxes.mir_vm_min → selfhost/vm/boxes/mir_vm_min.hako
```


Strict checks (gated)
- Enable `NYASH_USING_CHECKS_STRICT=1` to fail fast on:
  - `cycle` (workspace dependency cycles)
  - `missing_dep` (workspace dependency missing or incompatible `^major`)
  - `conflict` (same namespace exported by multiple paths)
- `unresolved`/`ambiguous` also honor strict behavior at runtime via `NYASH_USING_STRICT=1` and emit unified diagnostics JSON.
