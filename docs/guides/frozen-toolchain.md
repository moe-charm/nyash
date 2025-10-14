# Frozen Toolchain — Freeze the Bootstrap Compiler (Phase 15.76+)

Goal
- Eliminate dual maintenance of two parsers by freezing the Rust line.
- Develop only in Hakorune (.hako) while using a frozen EXE for day‑to‑day work.

Two Lines (Phase 15.75–15.76)
- Rust line (bootstrap): Last Rust VM build. Used only to mint a frozen EXE.
- Frozen line (primary): `hako-frozen-*.exe` executes/compiles Hakorune sources.

Status
- Week‑1 (extern_c MVP): DONE — minimal dynamic FFI via `extern_c "symbol"(args)`.
- Week‑2: Native lib (`libllvm_backend`) to compile MIR JSON → .o, then link a frozen EXE.

Workflow (once Week‑2 lands)
1) Build the native backend lib (cdylib)
2) Use the Rust VM (last build) to run the Hakorune compiler and emit objects via `extern_c` calls
3) Link objects into `bin/hako-frozen-vN.exe`
4) Switch development/testing to the frozen binary

Operational Policy
- Default deny for FFI (strict). Expand by config (ENV/TOML) only.
- The Rust source (parser/*) is considered frozen post vN (guard in CI).
- Periodic refresh is allowed (e.g., yearly) to roll a new v(N+1) frozen EXE.

Toggles & Profiles
- Day‑to‑day: prefer the frozen EXE for tests and local runs.
- Keep a documented escape hatch to flip back to Rust line when needed.

Risks
- Platform linker/ABI variance → mitigate with a thin tested backend lib.
- Scope creep → keep Week‑2 narrowly focused on MIR JSON → .o only.

Related
- docs/reference/language/extern_c.md
- docs/development/roadmap/phases/phase 15.76/MILESTONE.md
- docs/reference/boxes/frozen_v1.md

---

Frozen v1 Box Set & Packaging

Default (recommended for Frozen v1)
- Packaging: static link (single binary). Rationale: reproducibility・配布容易・CI安定。
- Included boxes (minimal core):
  - String, Array, Map
  - Console (print)
  - Time (now_ms)
  - JSON (stringify/min)
  - File[min]（必要最小限の読み書き／将来強化域は別途）

Why static first?
- Single artifact（壊れにくい）: 依存DLL/.so不整合を回避
- Bootstrap期間の観測容易化: “結果が違う時に切り分けが簡単”

Dynamic add‑ons（任意拡張）
- 追加機能は `.so/.dll` プラグインとして後置き可能（VMと共通の仕組み）。
- 推奨プラグイン候補：
  - Regex（非必須・重め。後付け推奨）
  - Crypto（実験／将来域）
  - OS/Path（環境依存差が大きいので後付け）

Switching policies later
- “フル静的”→“コア静的＋拡張動的”の順に段階解凍する方針。
- フラグは `hako_kernel` 側 features で制御（core‑collections/core‑io など）。

---

Quick Recipe — AOT object emission via extern_c

0) Emit MIR JSON from your Hakorune source (Rust VM last build)

```
# Example: emit MIR JSON v0 from a source file (runs no code)
./target/release/hakorune --backend mir \
  --emit-mir-json build/program.mir.json \
  apps/your_app/main.hako
```

1) Build the native backend library

```
cargo build --release -p llvm_backend
```

2) Allow the symbol and point the VM to the library if needed

```
export HAKO_FFI_ALLOW_LIST=llvm_compile_mir_to_object
# Optional: add extra search paths (':' separated)
export HAKO_FFI_LIB_PATHS="$(pwd)/target/release"
```

3) From Hakorune code, call the function

Example (inline):

```
static box Main {
  main() {
    // inputs
    local in;  in  = "build/program.mir.json";
    local out; out = "build/program.o";
    // call native backend (0 on success, -1 on failure)
    local rc; rc = extern_c "llvm_compile_mir_to_object" (in, out);
    if (rc == 0) { print("OK"); } else { print("NG"); }
    return rc;
  }
}
```

4) Link (example — subject to your runtime/linker)

Until the static runtime (`nyrt`) is wired in this repository, this step is environment‑specific. For a simple program that doesn’t require the NyRT, you can link a single object into an executable:

```
clang build/program.o -o build/program
```

If your program requires the runtime, link it against your NyRT archive (once available):

```
clang build/program.o -L path/to/nyrt -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o build/program
```

Notes
- The repository wires the VM to search `libllvm_backend` automatically in `target/release` and `$NYASH_ROOT/target/release`. You can add extra paths via `HAKO_FFI_LIB_PATHS`.
- Keep the allowlist strict. Use `HAKO_FFI_ALLOW_LIST` to opt‑in symbols per build/profile.

---

Frozen v1 Checklist (Ready to mint)
- [ ] extern_c（MIR/VM/Fail‑Fast）に合意済み（docs + smokes）
- [ ] `libs/llvm_backend` がビルド緑で、`.o` 出力が検証済み
- [ ] allowlist は ENV/TOML 経由で拡張可能（既定はDeny）
- [ ] MIR JSON の出力→`.o`→リンク手順がdocs通り再現
- [ ] 凍結EXEに同梱する最小Box（String/Array/Map/Console/Time/JSON/File[min]）を決定
- [ ] 配布/再現のためのタグ付け・ハッシュ記録をドキュメント化

---

Standard Mint Recipe — Freeze a runnable EXE (copy/paste)

0) Build prerequisites

```
cargo build --release                       # build VM (last Rust line)
cargo build --release -p llvm_backend       # build native backend lib
```

1) Emit MIR JSON for your components (parser/mir/vm/main)

```
mkdir -p build/mir

./target/release/hakorune --backend mir \
  --emit-mir-json build/mir/parser.mir.json \
  apps/selfhost/parser.hako

./target/release/hakorune --backend mir \
  --emit-mir-json build/mir/mir_builder.mir.json \
  apps/selfhost/mir_builder.hako

./target/release/hakorune --backend mir \
  --emit-mir-json build/mir/vm.mir.json \
  apps/selfhost/vm.hako

./target/release/hakorune --backend mir \
  --emit-mir-json build/mir/main.mir.json \
  apps/selfhost/main.hako
```

2) Compile each MIR JSON to object via extern_c (wrapper provided)

```
mkdir -p build/obj
tools/aot/emit_object_via_extern_c.sh build/mir/parser.mir.json      build/obj/parser.o
tools/aot/emit_object_via_extern_c.sh build/mir/mir_builder.mir.json build/obj/mir_builder.o
tools/aot/emit_object_via_extern_c.sh build/mir/vm.mir.json          build/obj/vm.o
tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json        build/obj/main.o
```

3) Link into the frozen binary (minimal wrapper)

```
mkdir -p bin
tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 \
  build/obj/parser.o build/obj/mir_builder.o build/obj/vm.o build/obj/main.o
```

If your program requires a static runtime, provide it via `--nyrt`:

```
tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 \
  build/obj/*.o --nyrt /path/to/libnyrt.a
```

4) Verify and tag

```
bin/hako-frozen-v1 --version || true
git tag -a v1.0-frozen -m "frozen toolchain v1 (commit $(git rev-parse --short HEAD))"
```

Optional — emit LLVM IR (.ll) instead of object

```
# Using extern_c helper symbol provided by libllvm_backend
export HAKO_FFI_ALLOW_LIST=llvm_compile_mir_to_ll
tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json build/obj/main.ll   # replace wrapper to call ll function if desired
```

Or call the harness directly:

```
python3 tools/llvmlite_harness.py --in build/mir/main.mir.json --emit-ll build/ir/main.ll
```

---

Doctor — quick end-to-end validation

Prerequisites
- `python3` + `llvmlite`
- `clang` (link step). If absent, the doctor skips linking or uses a tiny C stub path when possible.

Run
```
bash tools/aot/doctor_frozen_v1.sh
```

What it checks
- Finds `hakorune` (VM) and emits MIR JSON from `examples/simple_return.hako`
- Emits an object via `extern_c` → `llvm_compile_mir_to_object`
- Links with `libhako_kernel.a` when available; otherwise uses a tiny C main() stub
- Runs the resulting binary and validates the `Result: <n>` line

Troubleshooting
- `clang: not found`: install clang, or run up to object emission only.
- `llvmlite import error`: `pip install llvmlite` (or enable your Python env)
- `symbol not allowed`: export `HAKO_FFI_ALLOW_LIST=llvm_compile_mir_to_object`

Success example (doctor)
```
[doctor] root: .../hakorune-selfhost
[doctor] llvmlite OK
[doctor] building llvm_backend (cdylib) ...
[doctor] emitting MIR JSON ...
[doctor] emitting object via extern_c ...
[doctor] linking ...
[doctor] running ...
[doctor] run | Result: 0
[doctor] PASS
```

Linking examples
- With static runtime (recommended when available):
```
tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 \
  build/obj/*.o --nyrt crates/hako_kernel/target/release/libhako_kernel.a
```
- Fallback (no NyRT archive): compile a tiny C stub that calls `ny_main()` and link with your objects (the doctor does this automatically when NyRT is unavailable).

Extended mode
- The doctor also builds three extra library objects (Parser/MirBuilder/VM) and relinks to validate multi‑object linking.
- When some helper symbols collide across objects, it uses `-Wl,--allow-multiple-definition` (dev only) to proceed.

---

---

Windows Notes (plan)

Variants
- MSVC toolchain (clang-cl/link.exe): produces `hako_kernel.lib`, link with `/LINK` flags.
- MinGW/Clang (lld): produces `libhako_kernel.a`, link with `clang` and add Windows libs as needed.

MinGW/Clang example
```
# Build VM + native backend first (WSL/MinGW shell)
cargo build --release
cargo build --release -p llvm_backend

# (Optional) Build static runtime (when ready)
cargo build --release -p hako_kernel --no-default-features -F core-runtime,core-collections,core-io

# Emit MIR JSON and .o (same as Linux)
./target/release/hakorune --backend mir --emit-mir-json build/mir/main.mir.json examples/simple_return.hako
tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json build/obj/main.o

# Link (MinGW)
clang build/obj/main.o -o bin/hako-frozen-v1.exe \
  -Wl,--whole-archive crates/hako_kernel/target/release/libhako_kernel.a -Wl,--no-whole-archive \
  -lws2_32 -lbcrypt
```

MSVC/clang-cl example
```
# Library name differs (hako_kernel.lib). Use Developer Prompt or properly set LIB/INCLUDE.
clang-cl /Fe:bin\\hako-frozen-v1.exe build\\obj\\main.obj \
  /link /LIBPATH:crates\\hako_kernel\\target\\release hako_kernel.lib
```

WSL parity
- 本リポのスモークは WSL での動作を前提に整備済みです。Windows ネイティブの EXE 生成は、当面は「WSL で .o 生成 → Windows 側でリンク」という二段運用が簡単です。
- long‑term: hako_kernel を MSVC/MinGW 双方で安定ビルドできるよう順次対応します。
 - Report: build/WINDOWS_LINK_TEST_REPORT.md — end-to-end steps and outputs

Windows object generation
- Method 1 (recommended on Windows):
```
python tools/llvmlite_harness.py --in build/mir/main.mir.json --target windows --out build/obj/main_win.obj
```
- Method 2 (cross from WSL):
```
# produce .ll then cross-compile to .obj
python3 tools/llvmlite_harness.py --in build/mir/main.mir.json --emit-ll build/ir/main.ll
tools/aot/windows/ll_to_obj.sh build/ir/main.ll build/obj/main_win.obj --target x86_64-pc-windows-msvc
```

---

Feature presets (Frozen v1 static)

Goal: include the Box set above in a single static runtime archive.

Build (when `hako_kernel` is available):
```
cargo build --release -p hako_kernel --no-default-features \
  -F core-runtime,core-collections,core-io
# Output: crates/hako_kernel/target/release/libhako_kernel.a
```

Link with the wrapper:
```
tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 \
  build/obj/*.o --nyrt crates/hako_kernel/target/release/libhako_kernel.a
```

Sample success output (minimal program returns 0)
```
Result: 0
```
