Testing Guide — Runtime/Plugins/Smokes (Phase‑31)

Overview
- Unit: prefer `NYASH_DISABLE_PLUGINS=1 cargo test -q` for quick green runs.
- Plugins-only: `cargo test -q --features plugins -- --ignored` to run plugin-dependent tests.
- Smokes: use `tools/smokes/v2/run.sh --profile {plugins|quick}` or call individual scripts under `tools/smokes/v2/profiles/...`.

Representative Smokes (plugins)
- `tools/smokes/v2/profiles/plugins/callable_async_plugin_vm.sh` — CallableBox.callAsync → Future 表示の安定
- `tools/smokes/v2/profiles/plugins/set_bad_arity_vm.sh` — SetBox の arity ガード（Fail‑Fast）
- `tools/smokes/v2/profiles/plugins/plugin_map_len_vm.sh` — len/length エイリアスの整合
- `tools/smokes/v2/profiles/plugins/map_values_array_element_vm.sh` — Map.values → Array.size の連鎖確認
- `tools/smokes/v2/profiles/plugins/hosthandle_boundary_suite_vm.sh` — HostHandleRouter の境界（-1/-11/-13/-14）

HostHandle -14 Boundary Check
- Enable with `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1` (alias `NYASH_HOSTHANDLE_TEST_RET_MISMATCH=1`).
- VM prints `hosthandle-test rc=-14` on stdout for String.len path (HostSlot and Extern).
- The smoke captures stdout to a temp file before grepping to avoid PIPE-loss.

Module Paths — Callable/Future (meta)
- New modules: `src/runtime/meta/{callable,future}/`.
- Preferred imports: `runtime::meta::{callable::callable_box::CallableBox, future::future_box::FutureBox}`.
- Legacy paths `runtime::{callable_box,future_box}` remain via re-export (will be removed in Phase‑32).

Quick Commands
- Unit (no plugins): `NYASH_DISABLE_PLUGINS=1 cargo test -q`
- Plugins only: `cargo test -q --features plugins -- --ignored`
- One smoke: `bash tools/smokes/v2/profiles/plugins/plugin_map_len_vm.sh`

