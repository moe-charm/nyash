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

## Codex Async Workflow (Background Jobs)
- Purpose: run Codex tasks in the background and notify a tmux session on completion.
- Script: `tools/codex-async-notify.sh`
- Defaults: posts to tmux session `codex` (override with env `CODEX_DEFAULT_SESSION` or 2nd arg); logs to `~/.codex-async-work/logs/`.

Usage
- Quick run (sync output on terminal):
  - `./tools/codex-async-notify.sh "Your task here" [tmux_session]`
- Detached run (returns immediately):
  - `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "Your task" codex`
- Tail lines in tmux notification (default 60):
  - `CODEX_NOTIFY_TAIL=100 ./tools/codex-async-notify.sh "…" codex`

Concurrency Control
- Cap concurrent workers: set `CODEX_MAX_CONCURRENT=<N>` (0 or unset = unlimited).
- Mode when cap reached: `CODEX_CONCURRENCY_MODE=block|drop` (default `block`).
- De‑duplicate same task string: `CODEX_DEDUP=1` to skip if identical task is running.
- Example (max 2, dedup, detached):
  - `CODEX_MAX_CONCURRENT=2 CODEX_DEDUP=1 CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "Refactor MIR 13" codex`

Keep Two Running
- Detect running Codex exec jobs precisely:
  - Default counts by PGID to treat a task with multiple processes (node/codex) as one: `CODEX_COUNT_MODE=pgid`
  - Raw process listing (debug): `pgrep -af 'codex.*exec'`
- Top up to 2 jobs example:
  - `COUNT=$(pgrep -af 'codex.*exec' | wc -l || true); NEEDED=$((2-${COUNT:-0})); for i in $(seq 1 $NEEDED); do CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "<task $i>" codex; done`

Notes
- tmux notification uses `paste-buffer` to avoid broken lines; increase tail with `CODEX_NOTIFY_TAIL` if you need more context.
- Avoid running concurrent tasks that edit the same file; partition by area to prevent conflicts.
 - If wrappers spawn multiple processes per task (node/codex), set `CODEX_COUNT_MODE=pgid` (default) to count unique process groups rather than raw processes.
