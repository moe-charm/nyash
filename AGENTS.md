# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Nyash core (MIR, backends, runner modes). Key: `backend/`, `runner/`, `mir/`.
- `crates/nyrt/`: NyRT static runtime for AOT/LLVM (`libnyrt.a`).
- `plugins/`: First‑party plugins (e.g., `nyash-array-plugin`).
- `apps/` and `examples/`: Small runnable samples and smokes.
- `tools/`: Helper scripts (build, smoke).
- `tests/`: Rust and Nyash tests; historical samples in `tests/archive/`.
- `nyash.toml`: Box type/plug‑in mapping used by runtime.

## Build, Test, and Development Commands
- Build (JIT/VM): `cargo build --release --features cranelift-jit`
- Build (LLVM AOT): `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
- Quick VM run: `./target/release/nyash --backend vm apps/APP/main.nyash`
- Emit + link (LLVM): `tools/build_llvm.sh apps/APP/main.nyash -o app`
- Smokes: `./tools/llvm_smoke.sh release` (use env toggles like `NYASH_LLVM_VINVOKE_RET_SMOKE=1`)

## Coding Style & Naming Conventions
- Rust style (rustfmt defaults): 4‑space indent, `snake_case` for functions/vars, `CamelCase` for types.
- Keep patches focused; align with existing modules and file layout.
- New public APIs: document minimal usage and expected ABI (if exposed to NyRT/plug‑ins).

## Testing Guidelines
- Rust tests: `cargo test` (add targeted unit tests near code).
- Smoke scripts validate end‑to‑end AOT/JIT (`tools/llvm_smoke.sh`).
- Test naming: prefer `*_test.rs` for Rust and descriptive `.nyash` files under `apps/` or `tests/`.
- For LLVM tests, ensure LLVM 18 is installed and set `LLVM_SYS_180_PREFIX`.

## Commit & Pull Request Guidelines
- Commits: concise imperative subject; scope the change (e.g., "llvm: fix argc handling in nyrt").
- PRs must include: description, rationale, reproduction (if bug), and run instructions.
- Link issues (`docs/issues/*.md`) and reference affected scripts (e.g., `tools/llvm_smoke.sh`).
- CI: ensure smokes pass; use env toggles in the workflow as needed.

## Security & Configuration Tips
- Do not commit secrets. Plug‑in paths and native libs are configured via `nyash.toml`.
- LLVM builds require system LLVM 18; install via apt.llvm.org in CI.
- Optional logs: enable `NYASH_CLI_VERBOSE=1` for detailed emit diagnostics.
