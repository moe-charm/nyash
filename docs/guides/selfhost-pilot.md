Self‑Hosting Pilot — Quick Guide (Phase‑15)

Overview
- Goal: Run Ny→JSON v0 via the selfhost compiler path and execute with PyVM/LLVM.
- Default remains env‑gated for safety; CI runs smokes to build confidence.

Recommended Flows
- Runner (pilot): `NYASH_USE_NY_COMPILER=1 ./target/release/nyash --backend vm apps/examples/string_p0.nyash`
- Emit‑only: `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=1 ...`
- EXE‑first (parser EXE): `tools/build_compiler_exe.sh && NYASH_USE_NY_COMPILER=1 NYASH_USE_NY_COMPILER_EXE=1 ./target/release/nyash --backend vm apps/examples/string_p0.nyash`
- LLVM AOT: `NYASH_LLVM_USE_HARNESS=1 tools/build_llvm.sh apps/... -o app && ./app`

CI Workflows
- Selfhost Bootstrap (always): `.github/workflows/selfhost-bootstrap.yml`
  - Builds nyash (`cranelift-jit`) and runs `tools/bootstrap_selfhost_smoke.sh`.
- Selfhost EXE‑first（optional）
  - crate 直結（ny-llvmc）で JSON→EXE→実行までを最短経路で確認できるよ。
  - 手順（ローカル）:
    1) MIR(JSON) を出力: `./target/release/nyash --emit-mir-json tmp/app.json --backend mir apps/tests/ternary_basic.nyash`
    2) EXE 生成: `./target/release/ny-llvmc --in tmp/app.json --emit exe --nyrt target/release --out tmp/app`
    3) 実行: `./tmp/app`（戻り値が exit code）
  - ワンコマンドスモーク: `bash tools/crate_exe_smoke.sh apps/tests/ternary_basic.nyash`
  - CLI で直接 EXE 出力: `./target/release/nyash --emit-exe tmp/app --backend mir apps/tests/ternary_basic.nyash`
  - Installs LLVM 18 + llvmlite, then runs `tools/exe_first_smoke.sh`.

Useful Env Flags
- `NYASH_USE_NY_COMPILER=1`: Enable selfhost compiler pipeline.
- `NYASH_NY_COMPILER_EMIT_ONLY=1`: Print JSON v0 only (no execution).
- `NYASH_NY_COMPILER_TIMEOUT_MS=4000`: Child timeout (ms). Default 2000.
- `NYASH_USE_NY_COMPILER_EXE=1`: Prefer external parser EXE.
- `NYASH_NY_COMPILER_EXE_PATH=<path>`: Override EXE path.
- `NYASH_SELFHOST_READ_TMP=1`: Child reads `tmp/ny_parser_input.ny` when supported.

Troubleshooting (short)
- No Python found: install `python3` (PyVM / harness).
- No `llvm-config-18`: install LLVM 18 dev (see EXE‑first workflow).
- llvmlite import error: `python3 -m pip install llvmlite`.
- Parser child timeout: raise `NYASH_NY_COMPILER_TIMEOUT_MS`.
- EXE‑first bridge mismatch: re‑run with `NYASH_CLI_VERBOSE=1` and keep `dist/nyash_compiler/sample.json` for inspection.

Notes
- JSON v0 schema is stable but not yet versioned; validation is planned.
- Default backend `vm` maps to PyVM unless legacy VM features are enabled.
