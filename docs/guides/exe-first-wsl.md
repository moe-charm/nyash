# EXE‑First Quickstart (WSL/Ubuntu)

This guide prioritizes building and running the Nyash parser as a native executable on WSL (Ubuntu). It uses the LLVM harness (llvmlite) and the NyRT static runtime.

Prerequisites
- Rust toolchain (stable): `curl https://sh.rustup.rs -sSf | sh`
- Build tools: `sudo apt update && sudo apt install -y build-essential git python3 python3-pip`
- llvmlite: `pip3 install --user llvmlite`
- LLVM 18 (for `llvm-config-18` used by the Rust build + tools):
  - Ubuntu (with apt.llvm.org):
    - `sudo apt install -y wget gnupg lsb-release`
    - `wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && sudo ./llvm.sh 18`
    - This installs `llvm-18` and `llvm-18-dev` (provides `llvm-config-18`).

Verify
- `rustc --version`
- `python3 -c "import llvmlite, sys; print('llvmlite', llvmlite.__version__)"`
- `llvm-config-18 --version`

Build Parser EXE (bundle)
- `tools/build_compiler_exe.sh`
- Result: `dist/nyash_compiler/` containing `nyash_compiler`, `nyash.toml`, and the FileBox plugin.

Smoke (Parser EXE → JSON)
- `echo 'return 1+2*3' > dist/nyash_compiler/tmp/sample.ny`
- `(cd dist/nyash_compiler && ./nyash_compiler tmp/sample.ny > sample.json)`
- `head -n1 dist/nyash_compiler/sample.json` should start with `{` and contain `"kind":"Program"`.

End‑to‑End (JSON → execute via bridge)
- `./tools/exe_first_smoke.sh`
  - Builds the EXE bundle, runs parser → JSON, and executes via the bridge to verify exit code `7`.

MIR Builder (optional, EXE)
- Build: `cargo build --release --features llvm`
- EXE from JSON: `./target/release/ny_mir_builder --in dist/nyash_compiler/sample.json --emit exe -o app_out`
- Run: `./app_out` (exit `7` expected for `return 1+2*3`).

Runner with EXE‑First Parser
- `NYASH_USE_NY_COMPILER=1 NYASH_USE_NY_COMPILER_EXE=1 ./target/release/nyash --backend vm tmp/sample.nyash`
- Smoke: `./tools/exe_first_runner_smoke.sh`

Troubleshooting
- `llvm-config-18: not found`
  - Ensure apt.llvm.org installation worked (see above), or install the distro’s `llvm-18-dev` package.
- `ModuleNotFoundError: llvmlite`
  - `pip3 install --user llvmlite` and re‑run the build/smoke.
- Link errors (`cc` not found or missing libs)
  - `sudo apt install -y build-essential`
  - If you have a custom toolchain, export `CC` to point at your compiler.
- Plugin resolution
  - The EXE bundle includes a minimal `nyash.toml` and plugin paths under `dist/nyash_compiler/plugins/`.

Notes
- The EXE‑first path is the delivery priority. PyVM remains a development aid for semantics parity.
- Windows support is evolving; WSL is the recommended environment for now.

