# Phase 10.11: Builtins → Plugins Migration

## Goals
- Remove builtin Box implementations from execution paths (Interpreter/VM/JIT) to avoid divergence and double sources of truth.
- Provide all functionality via plugins (BID-FFI v1) and/or user-defined boxes.
- Keep backward compatibility guarded behind env flags until migration completes.

## Rationale
- Conflicts like ConsoleBox builtin vs plugin cause unexpected behavior.
- Native build (AOT/EXE) path benefits from uniform plugin boundary.
- One registry, one implementation per Box: simpler, safer.

## Plan (Incremental)
1) Disable Switch (Now)
- Add `NYASH_DISABLE_BUILTINS=1` to skip registering builtin box factory.
- Keep off by default; use in CI lanes and targeted tests.

2) Constructor Delegation (Now → Next)
- Ensure all constructors go through the unified registry, not direct builtin instantiation.
- Done: ConsoleBox; Next: remaining non-basic constructors.

3) Override Policy (Ongoing)
- Use `NYASH_USE_PLUGIN_BUILTINS=1` + `NYASH_PLUGIN_OVERRIDE_TYPES` to prefer plugins for selected types.
- Grow the allowlist as plugins become available.

4) Plugin Coverage (Milestones)
- ConsoleBox (stdout) — done
- Array/Map/String/Integer — in place
- File/Net/Python — in place
- Math/Time/etc. — add `nyash_box.toml` and minimal plugins

5) Remove Builtins (Final)
- Remove builtin factory or move into separate optional crate for legacy runs.
- Update docs, examples, and CI to plugin-only.

## Acceptance Criteria
- `NYASH_DISABLE_BUILTINS=1` + plugin set → examples run green (VM path).
- No direct builtins in interpreter constructors (registry only).
- JIT/AOT compile from MIR uses only plugin invoke shims for Box methods.

## How to Test
```bash
# Strict plugin preference + disable builtins
export NYASH_USE_PLUGIN_BUILTINS=1
export NYASH_PLUGIN_OVERRIDE_TYPES="ArrayBox,MapBox,ConsoleBox,StringBox,IntegerBox"
export NYASH_DISABLE_BUILTINS=1

cargo build --release --features cranelift-jit
./target/release/nyash --backend vm examples/console_demo.nyash
```

## Notes
- Temporary breakages expected when some builtin-only boxes remain. Use the override allowlist tactically.
- Keep `[libraries]` and `[plugins]` configured to ensure provider discovery.
