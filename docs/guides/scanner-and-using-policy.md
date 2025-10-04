# Scanner And Using Policy

Purpose: eliminate fragile string scanning and make `using` predictable across profiles.

## Principles

- Structure-first: parse structure with scanners, stringify at the edge.
- Dual-key search: when handling embedded JSON fragments, support both plain and escaped key forms.
- Limit raw `indexOf`: only allow for lightweight key existence probes, not for structural boundaries.

## Standard Boxes

- `StringScanBox` (apps/selfhost/vm/boxes/string_scan.hako)
  - `find_unescaped(text, ch, pos)` – skip escaped occurrences.
  - `scan_string_end(text, start)` – returns closing quote index or -1.

- `JsonScanBox` (apps/selfhost/vm/boxes/json_scan.hako)
  - `seek_obj_end(text, start)` / `seek_array_end(text, start)` – escape-aware end scanners.
  - `find_key_dual(text, plain, escaped, pos)` – dual-key search helper.

## Raw Strings

- Supported: `r"..."` and `r#"..."#` (hash can repeat). Contents are not unescaped.
- Use raw strings when embedding JSON fragments to avoid double-escaping.

## Using Policy

- Quick/CI profiles: file-path `using` is discouraged/forbidden. Prefer logical names resolved by `[modules]` or packages in `nyash.toml`/`hako.toml`.
- Tests may provide `NYASH_MODULES` to inject a minimal mapping for E2E.

## Lint (dev)

- Plan: add a dev-only lint to flag raw `indexOf` used at structural boundaries.
