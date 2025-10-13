# Naming & Extensions Policy (Hakorune)

This project adopts "Hakorune" as the primary brand and "Hako ABI" as the canonical ABI name. Historical "Nyash" references remain as compatibility aliases for a time-boxed period.

## Brand & ABI
- Preferred brand: Hakorune (CLI: `hakorune`, alias `hako`).
- ABI name: Hako ABI (formerly Nyash ABI). Old references remain as commentary/aliases only.

## Backends (summary)
- `--backend nyvm` → Hakorune VM (Ny implementation). Engine selector: `HAKO_NYVM_ENGINE={hakorune|mini}` (default `hakorune`).
- `--backend vm|rust` → Rust VM line.
- `--backend llvm` → LLVM line (llvmlite harness / AOT).

See also: `docs/guides/cli-backends-and-tools.md`.

## File extensions
- Default: `.hako`
- Compatibility: `.nyash` is accepted but deprecated. New sources and docs must use `.hako`.

### Migration plan
1) Documentation and new samples use `.hako` exclusively.
2) Test/smoke additions use `.hako`.
3) Existing `.nyash` files will be migrated in small batches with mechanical reference updates.
4) A deprecation warning appears when running a `.nyash` file; warning lives for at least one release window.

### Rationale
- Reduce ambiguity between legacy naming (“nyash”) and modern branding (“Hakorune”).
- Make it clearer that backends and tools are switched through the unified Hakorune CLI.
