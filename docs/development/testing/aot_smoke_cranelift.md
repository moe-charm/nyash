# Cranelift AOT Smoke (Windows‑first)

Purpose
- Validate the Cranelift‑based AOT pipeline end‑to‑end:
  1) Build `nyash` with `cranelift-jit` feature.
  2) Emit an object via `NYASH_AOT_OBJECT_OUT` while running `--backend cranelift`.
  3) Link the object with NyRT into a runnable binary (via LinkerBox or helper scripts).
  4) Run the binary and assert output.

Prerequisites
- Build flags: `cargo build --release --features cranelift-jit`
- Windows:
  - Prefer MSVC `link.exe` (Developer Command Prompt or properly set env).
  - Fallback: `lld-link` in `PATH`.
  - PowerShell available for `tools/aot_smoke_cranelift.ps1`.
- Unix (optional): system linker (`ld`), or `lld`/`mold`, and `tools/aot_smoke_cranelift.sh`.

Environment toggles
- `NYASH_CLIF_ARRAY_SMOKE=1`: run array smoke (simple Result check).
- `NYASH_CLIF_ARRAY_RET_SMOKE=1`: run “return value” array smoke.
- `NYASH_CLIF_ECHO_SMOKE=1`: run echo smoke (stdin → stdout).
- `NYASH_CLIF_VINVOKE_SMOKE=1`: run variable‑length invoke smoke (plugins required).
- `NYASH_CLIF_VINVOKE_RET_SMOKE=1`: run vinvoke return/size smokes (plugins required).
- `NYASH_DISABLE_PLUGINS=1`: disable plugin‑dependent smokes.
- `NYASH_LINK_VERBOSE=1`: print final link command.

Pseudo run
- Script: `tools/aot_smoke_cranelift.sh` / `tools/aot_smoke_cranelift.ps1`
- Typical invocation: `./tools/aot_smoke_cranelift.sh release`

Pseudo output (example)
```
[clif-aot-smoke] building nyash (release, feature=cranelift-jit)...
[clif-aot-smoke] emitting object via --backend cranelift ...
[clif-aot-smoke] OK: object generated: /ABS/path/target/aot_objects/core_smoke.obj (1536 bytes)

[clif-aot-smoke][win] linking app_clif.exe using link.exe
[clif-aot-smoke][win] entry=nyash_main subsystem=CONSOLE runtime=nyrt.lib
[clif-aot-smoke] running app_clif.exe ...
[clif-aot-smoke] output: Result: 3
[clif-aot-smoke] OK: core smoke passed

[clif-aot-smoke] skipping array smoke (set NYASH_CLIF_ARRAY_SMOKE=1 to enable)
[clif-aot-smoke] skipping echo smoke (set NYASH_CLIF_ECHO_SMOKE=1 to enable)
[clif-aot-smoke] skipping vinvoke smokes (set NYASH_CLIF_VINVOKE_SMOKE=1 / NYASH_CLIF_VINVOKE_RET_SMOKE=1)
```

What the script does (intended)
- Build:
  - `cargo build --release --features cranelift-jit`
- Emit object:
  - Ensure stable output dir: `mkdir -p target/aot_objects`
  - `NYASH_AOT_OBJECT_OUT="$PWD/target/aot_objects/core_smoke.obj" ./target/release/nyash --backend cranelift apps/hello/main.nyash > /dev/null || true`
  - Validate file exists and non‑zero size.
- Link:
  - Windows: PowerShell `tools/aot_smoke_cranelift.ps1 -Mode release`
  - Unix: `tools/aot_smoke_cranelift.sh release`
- Run and verify:
  - `./app_clif[.exe]` → expect a line including `Result:`.

Windows specifics
- Prefer MSVC `link.exe`; auto‑fallback to `lld-link` if present.
- If neither available, fail with a helpful message to open a Developer Command Prompt or install LLVM lld.
- Use `.obj` extension for emitted object; still accept `.o` if emitted by a GNU toolchain.

Exit codes
- 0: all enabled smokes passed
- 1: object missing/empty, or unexpected program output
- 2: toolchain missing (no Cranelift build or no linker)

Future alignment with LinkerBox
- This smoke is the acceptance test for LinkerBox’s AOT path on Cranelift:
  - Same entrypoint (`nyash_main`), runtime linkage (`nyrt.lib`/`libnyrt.a`), and CLI env (`NYASH_LINKER`, `NYASH_LINK_FLAGS`, `NYASH_LINK_VERBOSE`).
  - When LinkerBox becomes default, keep CLI stable and swap implementation behind it.

