# Frozen Toolchain Quickstart

Purpose
- Use the frozen Hakorune binary for fast, reproducible local development.

Quick usage
- Linux:
  - `./dist/hako-frozen-v1-linux-x64 apps/selfhost-minimal/main.hako`
- Windows (MSVC):
  - `hako-frozen-v1-win-x64-msvc.exe apps\selfhost-minimal\main.hako`

Notes
- Prefer running with `NYASH_MACRO_ENABLE=1` to enable syntactic macros.
- For extern_c AOT flows, see `docs/guides/frozen-toolchain.md`.

