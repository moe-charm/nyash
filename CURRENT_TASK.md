# CURRENT_TASK — Status and Next Steps (2025‑10‑16)

This page is a single‑screen snapshot of where we are and what comes next. It replaces scattered daily notes with a concise plan you can act on today.

## Snapshot
- Phase 15.76 (extern_c / Frozen Toolchain): baseline complete
  - extern_c syntax → MIR Extern(Callee) → VM dynamic FFI（deny‑by‑default, allowlist via ENV/TOML）
  - libs/llvm_backend: object emission + LL emission（`llvm_compile_mir_to_object`, `llvm_compile_mir_to_ll`）
  - AOT helpers + Doctor（extended multi‑obj）green on WSL/Linux
- Windows: WSL→Windows link verified end‑to‑end
  - Generate COFF `.obj` from WSL（harness `--target windows`）→ link on Windows（clang）→ run → Result: 0
  - When static runtime is absent, development stubs + tiny C main() stub unblock linking

## Completed (high‑level)
- Language/VM
  - extern_c MVP（parser/AST/MIR lowering/VM dynamic FFI）with allowlist and strict fail‑fast
- Backend & Tools
  - libs/llvm_backend: `llvm_compile_mir_to_ll` added
  - tools/llvmlite_harness.py: `--emit-ll`, `--target windows` support
  - src/llvm_py targets: `windows` target added（COFF .obj）
  - AOT helpers: `link_with_clang.sh`, `emit_ll_via_extern_c.sh`, `doctor_frozen_v1.sh`（extended: multi‑obj linking）
  - Windows helpers: `windows/ll_to_obj.sh`, `windows/link_stub_main.c`, `windows/nyrt_min_stubs_win.S`, batch link wrappers
- Docs
  - Frozen guide（標準レシピ/Doctor/Windowsノート）: `docs/guides/frozen-toolchain.md`
  - Frozen v1 Box セット: `docs/reference/boxes/frozen_v1.md`
  - Windows 実績レポート: `build/WINDOWS_LINK_TEST_REPORT.md`
  - Roadmap 15.77（Polish & Windows Plan）: `docs/development/roadmap/phases/phase-15.77/INDEX.md`

## In‑Flight — Phase 15.77 (Polish & Windows Plan)
- Stabilize static runtime on Windows（MinGW→MSVC）
  - Produce `libhako_kernel.a`/`hako_kernel.lib` reliably; replace dev stubs in examples
  - Record a “static runtime linked” success log and fold it into the guide
- Doctor polish
  - Improve diagnostics（explicit advice when allowlist/lib paths are missing）
  - Extended run: surface exit code vs Result line clearly
- Documentation
  - Add a short “Windows quicklink” snippet to the guide（copy‑paste for both toolchains）

## Prioritized TODOs
- P0 — unblocks next milestones
  - [ ] Windows: build static runtime（hako_kernel）& link sample EXE（Result: 0）
  - [ ] Frozen guide: add “Static runtime（Windows）example” section（copy‑paste）
- P1 — quality of life
  - [ ] Doctor: structured error messages（missing clang/llvmlite/allowlist/lib paths）
  - [ ] Harness: tighter logs for `--target windows` & optional IR dump hint
- P2 — later
  - [ ] CI: build‑only job for `llvm_backend`/harness smoke（opt‑in）
  - [ ] CI: optional Windows cross pipeline doc（no runner）

## Guardrails / Principles
- Fail‑Fast: no silent fallback for FFI/extern; defaults stay strict
- Minimal ENV: config broadens allowlist but never changes default semantics
- Structure first: helpers isolated under `tools/aot/` and `tools/aot/windows/`
- Docs placement: under `docs/guides/`, `docs/reference/`, `docs/development/roadmap/` only（散在禁止）

## How to Reproduce (quick)
- WSL — single obj → run（Linux）
  - `./target/release/hakorune --backend mir --emit-mir-json build/mir/main.mir.json examples/simple_return.hako`
  - `tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json build/obj/main.o`
  - `tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 build/obj/main.o`（NyRT無なら Doctor を参照）
- WSL → Windows（COFF）
  - `.obj`（推奨）: `python3 tools/llvmlite_harness.py --in build/mir/main.mir.json --target windows --out build/obj/main_win.obj`
  - Windows（devスタブ）: `clang link_stub_main.c nyrt_min_stubs_win.S main_win.obj -o test_main.exe`
  - 期待: `Result: 0`

## References
- Guide: `docs/guides/frozen-toolchain.md`
- Report: `build/WINDOWS_LINK_TEST_REPORT.md`
- Box Set: `docs/reference/boxes/frozen_v1.md`
- Roadmap: `docs/development/roadmap/phases/phase-15.77/INDEX.md`

## Parking Lot (tracked but not urgent)
- Plugin‑only build hardening（legacy‑boxes OFF 完全緑化）
- MirCall normalization coverage expansion（安全系のみ）
- Quick profile rollout tuning（HostHandle flags の段階ON）

