# Include-only Dependency Tree (Phase 0)

Goal
- Build a dependency tree using only Ny (no Array/Map), scanning source text for `include "..."`, and output stable JSON.

Scope (Phase 0)
- Only `include` is supported. `using/module/import` are out of scope.
- Runner bridge: `NYASH_DEPS_JSON=<path>` is read and logged only (no behavior change).

Tool
- `apps/selfhost/tools/dep_tree_min_string.hako`
  - Recursively reads source files, scans for `include "path"` outside of strings and comments.
  - Comments: `//` and `#` (line comments) are ignored.
  - Strings: `"..."` with `\"` escapes are honored.
  - Cycles: detected via a simple stack string; when detected, the child appears as a leaf node with empty `includes`/`children`.

Output format
```
{ "version": 1,
  "root_path": "<entry>",
  "tree": {
    "path": "<file>", "includes": ["..."], "children": [ <node> ]
  }
}
```

Acceptance criteria
- Running `make dep-tree` produces `tmp/deps.json` whose first non-empty line starts with `{` and conforms to the format above.
- The scanner does not pick includes inside strings or comments.
- Cycles do not crash or loop; the repeated node is represented as a leaf.

Examples
- Root: `apps/selfhost/smokes/dep_smoke_root.nyash` (includes `dep_smoke_child.nyash`)
- Cycle: `apps/selfhost/smokes/dep_smoke_cycle_a.nyash` ↔ `dep_smoke_cycle_b.nyash`

Validation (examples)
- `echo apps/selfhost/smokes/dep_smoke_root.nyash | ./target/release/nyash --backend vm apps/selfhost/tools/dep_tree_min_string.hako`
- `make dep-tree`

